use std::hash::Hash;

use crate::Graph;

pub type NestedVec = Vec<Vec<usize>>;

pub fn dual_iso<T: Eq + Hash>(graph: &Graph<T>, pattern: &Graph<T>) -> NestedVec {
    let initial_candidates = init_candidates(graph, pattern);
    todo!()
}

fn init_candidates<'graph, T: Eq + Hash>(
    graph: &'graph Graph<T>,
    pattern: &Graph<T>,
) -> Vec<&'graph Vec<usize>> {
    let mut candidates = Vec::with_capacity(pattern.node_count());
    for pattern_node_id in 0..pattern.node_count() {
        candidates.push(graph.nodes_by_label(pattern.node_label(pattern_node_id)))
    }
    candidates
}

#[cfg(test)]
mod tests {
    use crate::GraphBuilder;

    use super::*;

    #[test]
    fn match1() {
        let graph = GraphBuilder::new()
            .add_node(0, "b")
            .add_node(1, "a")
            .add_node(2, "a")
            .add_node(3, "c")
            .add_node(4, "b")
            .add_node(5, "a")
            .add_node(6, "b")
            .add_node(7, "c")
            .add_node(8, "b")
            .add_relationship(0, 1)
            .add_relationship(0, 3)
            .add_relationship(1, 6)
            .add_relationship(2, 6)
            .add_relationship(4, 1)
            .add_relationship(4, 3)
            .add_relationship(5, 4)
            .add_relationship(6, 2)
            .add_relationship(6, 5)
            .add_relationship(6, 7)
            .add_relationship(8, 5)
            .build();

        let pattern = GraphBuilder::new()
            .add_node(0, "a")
            .add_node(1, "b")
            .add_node(2, "c")
            .add_relationship(0, 1)
            .add_relationship(1, 0)
            .add_relationship(1, 2)
            .build();

        let matches = dual_iso(&graph, &pattern);

        assert_eq!(vec![vec![2, 6, 7]], matches)
    }
}
