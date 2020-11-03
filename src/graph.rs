#![allow(dead_code)]

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

pub struct Graph<T> {
    node_count: usize,
    relationship_count: usize,
    node_labels: HashMap<usize, Rc<T>>,
    label_idx: HashMap<Rc<T>, Vec<usize>>,
    offsets: Vec<usize>,
    lists: Vec<usize>,
}

impl<T> Graph<T>
where
    T: Eq + Hash,
{
    pub fn node_count(&self) -> usize {
        self.node_count
    }

    pub fn relationship_count(&self) -> usize {
        self.relationship_count
    }

    pub fn node_label(&self, node_id: usize) -> &T {
        self.validate_node_id(node_id);
        self.node_labels.get(&node_id).unwrap()
    }

    pub fn nodes_by_label(&self, label: &T) -> &Vec<usize> {
        self.label_idx.get(label).unwrap()
    }

    pub fn degree(&self, node_id: usize) -> usize {
        self.validate_node_id(node_id);
        let offset = self.offsets[node_id];
        self.lists[offset]
    }

    pub fn neighbors(&self, node_id: usize) -> &[usize] {
        self.validate_node_id(node_id);
        let offset = self.offsets[node_id];
        let degree = self.lists[offset];
        &self.lists[offset + 1..offset + 1 + degree]
    }

    fn validate_node_id(&self, node_id: usize) {
        if node_id >= self.node_count {
            panic!(
                "Node id {} must be within range [0..{}).",
                node_id, self.node_count
            )
        }
    }
}

#[derive(Default)]
pub struct GraphBuilder<T> {
    node_count: usize,
    relationship_count: usize,
    node_labels: HashMap<usize, Rc<T>>,
    adjacency_lists: HashMap<usize, Vec<usize>>,
}

impl<T> GraphBuilder<T>
where
    T: Eq + Hash,
{
    pub fn new() -> Self {
        GraphBuilder {
            node_count: 0,
            relationship_count: 0,
            node_labels: HashMap::new(),
            adjacency_lists: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node_id: usize, node_label: T) -> &mut Self {
        if node_id > self.node_count {
            panic!(
                "Next node id should be within range [0..{}], but was {}.",
                self.node_count, node_id
            )
        }
        if let Entry::Vacant(o) = self.node_labels.entry(node_id) {
            o.insert(Rc::new(node_label));
            self.node_count += 1;
        }
        self
    }

    pub fn add_relationship(&mut self, start_node: usize, end_node: usize) -> &mut Self {
        if !self.node_labels.contains_key(&start_node) {
            panic!("Start node {} has not been added yet.", start_node);
        }
        if !self.node_labels.contains_key(&end_node) {
            panic!("End node {} has not been added yet.", end_node);
        }
        self.adjacency_lists
            .entry(start_node)
            .or_insert_with(Vec::new)
            .push(end_node);
        self.relationship_count += 1;
        self
    }

    pub fn build(&mut self) -> Graph<T> {
        // initialize with 0
        let mut offsets = vec![0; self.node_count];
        // position at offset 0 stores the 0-degree
        let mut lists = vec![0];

        let adjacency_lists = std::mem::take(&mut self.adjacency_lists);
        for (node_id, mut list) in adjacency_lists {
            let degree = list.len();
            list.sort_unstable();
            offsets[node_id] = lists.len();

            // try to avoid too much resizing, but might have no effect in the end
            lists.reserve(degree + 1);
            lists.push(degree);
            lists.extend(list);
        }

        // Build label index
        let mut label_idx = HashMap::new();
        for (node_id, label) in self.node_labels.iter() {
            label_idx
                .entry(Rc::clone(label))
                .or_insert_with(Vec::new)
                .push(*node_id);
        }

        Graph {
            node_count: self.node_count,
            relationship_count: self.relationship_count,
            node_labels: std::mem::take(&mut self.node_labels),
            label_idx,
            offsets,
            lists,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_count() {
        let graph = GraphBuilder::new()
            .add_node(0, "foo")
            .add_node(1, "bar")
            .build();
        assert_eq!(2, graph.node_count())
    }

    #[test]
    #[should_panic(expected = "Next node id should be within range [0..1], but was 2.")]
    fn test_add_invalid_node() {
        let _ = GraphBuilder::new()
            .add_node(0, "foo")
            .add_node(2, "bar")
            .build();
    }

    #[test]
    fn test_relationship_count() {
        let graph = GraphBuilder::new()
            .add_node(0, "foo")
            .add_node(1, "bar")
            .add_relationship(0, 1)
            .build();
        assert_eq!(1, graph.relationship_count())
    }

    #[test]
    #[should_panic(expected = "Start node 0 has not been added yet")]
    fn test_add_relationship_for_invalid_start_node() {
        let _ = GraphBuilder::<&str>::new().add_relationship(0, 1).build();
    }

    #[test]
    #[should_panic(expected = "End node 1 has not been added yet")]
    fn test_add_relationship_for_invalid_end_node() {
        let _ = GraphBuilder::new()
            .add_node(0, "foo")
            .add_relationship(0, 1)
            .build();
    }

    #[test]
    fn test_node_label() {
        let graph = GraphBuilder::new()
            .add_node(0, "foo")
            .add_node(1, "bar")
            .build();
        assert_eq!("foo", *graph.node_label(0));
        assert_eq!("bar", *graph.node_label(1))
    }

    #[test]
    #[should_panic(expected = "Node id 1 must be within range [0..1).")]
    fn test_node_label_for_invalid_node() {
        let graph = GraphBuilder::new().add_node(0, "foo").build();
        let _ = *graph.node_label(1);
    }

    #[test]
    fn test_degree() {
        let graph = GraphBuilder::new()
            .add_node(0, "foo")
            .add_node(1, "bar")
            .add_node(2, "baz")
            .add_relationship(0, 1)
            .add_relationship(0, 2)
            .add_relationship(1, 2)
            .build();

        assert_eq!(2, graph.degree(0));
        assert_eq!(1, graph.degree(1));
        assert_eq!(0, graph.degree(2));
    }

    #[test]
    #[should_panic(expected = "Node id 1 must be within range [0..1).")]
    fn test_degree_for_invalid_node() {
        let graph = GraphBuilder::new().add_node(0, "foo").build();
        let _ = graph.degree(1);
    }

    #[test]
    fn test_neighbors() {
        let graph = GraphBuilder::new()
            .add_node(0, "foo")
            .add_node(1, "bar")
            .add_node(2, "baz")
            .add_node(3, "boo")
            .add_relationship(0, 2)
            .add_relationship(0, 1)
            .add_relationship(0, 0)
            .add_relationship(0, 3)
            .add_relationship(1, 2)
            .build();

        let empty: &[usize; 0] = &[];

        assert_eq!(&[0, 1, 2, 3], graph.neighbors(0));
        assert_eq!(&[2], graph.neighbors(1));
        assert_eq!(empty, graph.neighbors(2))
    }
}
