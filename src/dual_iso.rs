use std::borrow::Cow;
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

    simple_simulation(graph, pattern, &mut initial_candidates);
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
                    let mut found_relationship = false;
                    // for each candidate of v_P (v_G)
                    // TODO: efficient intersect between graph.neighbors(u_g) and candidates(v_p)
                    for v_g in &*candidates[*v_p] {
                        if graph.neighbors(*u_g).binary_search(v_g).is_ok() {
                            found_relationship = true;
                            break;
                        }
                    }
                    if found_relationship {
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

        let matches = simple_iso(&graph, &pattern);

        assert_eq!(vec![vec![2, 6, 7]], matches)
    }
}
