use std::borrow::Cow;
use std::cmp::min;
use std::hash::Hash;

use crate::Graph;

pub type NestedVec = Vec<Vec<usize>>;

pub fn simple_iso<T: Eq + Hash>(graph: &Graph<T>, pattern: &Graph<T>) -> NestedVec {
    let mut matches: NestedVec = vec![];
    let mut initial_candidates = init_candidates(graph, pattern);

    simple_simulation(graph, pattern, &mut initial_candidates);
    search(graph, pattern, &mut matches, &initial_candidates, 0);

    matches
}

pub fn dual_iso<T: Eq + Hash>(graph: &Graph<T>, pattern: &Graph<T>) -> NestedVec {
    let mut matches: NestedVec = vec![];
    let mut initial_candidates = init_candidates(graph, pattern);

    dual_simulation(graph, pattern, &mut initial_candidates);
    search(graph, pattern, &mut matches, &initial_candidates, 0);

    matches
}

fn search<T: Eq + Hash>(
    graph: &Graph<T>,
    pattern: &Graph<T>,
    matches: &mut NestedVec,
    candidates: &[Cow<Vec<usize>>],
    depth: usize,
) {
    if depth == pattern.node_count() {
        // found a match
        matches.push(candidates.iter().map(|c| c[0]).collect::<Vec<_>>());
        return;
    }
    for v_g in &*candidates[depth] {
        // check if v_G has matched a previous candidate
        if !candidates[..depth].iter().any(|x| x[0] == *v_g) {
            let mut new_candidates = candidates.to_owned();
            new_candidates[depth] = Cow::Owned(vec![*v_g]);
            if simple_simulation(graph, pattern, &mut new_candidates) {
                search(graph, pattern, matches, &new_candidates, depth + 1);
            }
        }
    }
}

fn init_candidates<'graph, T: Eq + Hash>(
    graph: &'graph Graph<T>,
    pattern: &Graph<T>,
) -> Vec<Cow<'graph, Vec<usize>>> {
    let mut candidates = Vec::with_capacity(pattern.node_count());
    for pattern_node_id in 0..pattern.node_count() {
        candidates.push(Cow::Borrowed(
            graph.nodes_by_label(pattern.node_label(pattern_node_id)),
        ))
    }
    candidates
}

fn simple_simulation<T: Eq + Hash>(
    graph: &Graph<T>,
    pattern: &Graph<T>,
    candidates: &mut Vec<Cow<Vec<usize>>>,
) -> bool {
    let mut is_updated = true;

    while is_updated {
        is_updated = false;
        // for each node u_P in the pattern
        for u_p in 0..pattern.node_count() {
            // for each neighbor of u_P (v_P)
            for v_p in pattern.neighbors(u_p) {
                // updated candidate set for u_P
                let mut u_g_new: Vec<usize> = vec![];
                // for each candidate of u_P (u_G)
                for u_g in &*candidates[u_p] {
                    // check if at least one edge exists in the graph
                    if do_intersect_sorted(graph.neighbors(*u_g), &*candidates[*v_p]) {
                        u_g_new.push(*u_g);
                    } else {
                        is_updated = true;
                    }
                }
                if u_g_new.is_empty() {
                    return false;
                }
                candidates[u_p] = Cow::Owned(u_g_new);
            }
        }
    }
    true
}

fn dual_simulation<T: Eq + Hash>(
    graph: &Graph<T>,
    pattern: &Graph<T>,
    candidates: &mut Vec<Cow<Vec<usize>>>,
) -> bool {
    let mut is_updated = true;

    while is_updated {
        is_updated = false;
        // for each node u_P in the pattern
        for u_p in 0..pattern.node_count() {
            // for each neighbor of u_P (v_P)
            for v_p in pattern.neighbors(u_p) {
                // updated candidate set for v_P
                let mut v_g_new: Vec<usize> = vec![];
                // updated candidate set for u_P
                let mut u_g_new: Vec<usize> = vec![];
                // for each candidate of u_P (u_G)
                for u_g in &*candidates[u_p] {
                    // check if at least one edge exists in the graph
                    let intersect = intersect_sorted(graph.neighbors(*u_g), &*candidates[*v_p]);
                    if !intersect.is_empty() {
                        u_g_new.push(*u_g);
                    } else {
                        // trigger re-eval of candidates if u_Q changed
                        is_updated = true;
                    }
                    union_into_sorted(&mut v_g_new, &*intersect);
                }
                // if there are no candidates for either u_P or v_P
                if u_g_new.is_empty() || v_g_new.is_empty() {
                    return false;
                }

                // trigger re-eval of candidates if v_Q changed
                if v_g_new.len() < (*candidates[*v_p]).len() {
                    is_updated = true;
                }

                candidates[*v_p] = Cow::Owned(intersect_sorted(&*candidates[*v_p], &*v_g_new));
                candidates[u_p] = Cow::Owned(u_g_new);
            }
        }
    }
    true
}

fn do_intersect_sorted(left: &[usize], right: &[usize]) -> bool {
    let mut i = 0;
    let mut j = 0;
    while i < left.len() && j < right.len() {
        if left[i] < right[j] {
            i += 1;
        } else if left[i] > right[j] {
            j += 1;
        } else {
            return true;
        }
    }
    return false;
}

fn intersect_sorted(left: &[usize], right: &[usize]) -> Vec<usize> {
    let mut intersect = Vec::new();
    intersect.resize(min(left.len(), right.len()), 0);

    let mut count = 0;
    let mut i = 0;
    let mut j = 0;
    let m = left.len();
    let n = right.len();
    let mut prev = usize::max_value();

    while i < m && j < n {
        if left[i] < right[j] {
            i += 1;
        } else if left[i] > right[j] {
            j += 1;
        } else {
            if left[i] != prev {
                prev = intersect[count];
                intersect[count] = left[i];
                count += 1;
            }
            i += 1;
            j += 1;
        }
    }

    intersect.truncate(count);
    intersect
}

fn union_into_sorted(left: &mut Vec<usize>, right: &[usize]) {
    let mut i = 0;
    let mut j = 0;
    let m = left.len();
    let n = right.len();

    while i < m && j < n {
        if left[i] < right[j] {
            i += 1;
        } else if left[i] > right[j] {
            left.insert(i, right[j]);
            j += 1;
        } else {
            i += 1;
            j += 1;
        }
    }

    while j < n {
        left.push(right[j]);
        j += 1;
    }
}

#[cfg(test)]
mod tests {
    use crate::GraphBuilder;

    use super::*;

    #[test]
    fn paper_match() {
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

        let matches = simple_iso(&graph, &pattern);
        assert_eq!(vec![vec![2, 6, 7]], matches);
        let matches = dual_iso(&graph, &pattern);
        assert_eq!(vec![vec![2, 6, 7]], matches)
    }

    #[test]
    fn test_intersect_sorted() {
        let a = vec![0, 1, 2, 3, 4];
        let b = vec![2, 3, 4, 5, 6];

        assert_eq!(vec![2, 3, 4], intersect_sorted(&a, &b));

        let a = vec![0, 1, 2, 3, 4];
        let b = vec![0, 1, 2, 3, 4];

        assert_eq!(vec![0, 1, 2, 3, 4], intersect_sorted(&a, &b));

        let a = vec![0];
        let b = vec![4];

        let expected: Vec<usize> = vec![];
        assert_eq!(expected, intersect_sorted(&a, &b))
    }

    #[test]
    fn test_do_intersect_sorted() {
        let a = vec![0, 1, 2, 3, 4];
        let b = vec![2, 3, 4, 5, 6];

        assert!(do_intersect_sorted(&a, &b));

        let a = vec![0, 1, 2, 3, 4];
        let b = vec![0, 1, 2, 3, 4];

        assert!(do_intersect_sorted(&a, &b));

        let a = vec![0];
        let b = vec![4];

        assert!(!do_intersect_sorted(&a, &b));
    }

    #[test]
    fn test_union_into_sorted() {
        let mut a = vec![0, 1, 2, 3, 4];
        let b = vec![2, 3, 4, 5, 6];
        union_into_sorted(&mut a, &b);
        assert_eq!(vec![0, 1, 2, 3, 4, 5, 6], a);

        let mut a = vec![2, 3, 4, 5, 6];
        let b = vec![0, 1, 2, 3, 4];
        union_into_sorted(&mut a, &b);
        assert_eq!(vec![0, 1, 2, 3, 4, 5, 6], a);

        let mut a = vec![0, 1, 2, 3, 4];
        let b = vec![0, 1, 2, 3, 4];

        union_into_sorted(&mut a, &b);
        assert_eq!(vec![0, 1, 2, 3, 4], a);

        let mut a = vec![0];
        let b = vec![4];

        union_into_sorted(&mut a, &b);
        assert_eq!(vec![0, 4], a);
    }
}
