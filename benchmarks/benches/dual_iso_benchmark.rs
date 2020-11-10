use criterion::{BenchmarkId, black_box, Criterion, criterion_group, criterion_main};
use rand::prelude::*;

use ::dual_iso::{dual_iso, Graph, GraphBuilder, simple_iso};

criterion_group!(benches, random_graph_benchmarks);
criterion_main!(benches);

fn random_graph_benchmarks(c: &mut Criterion) {
    let n = 42;
    let p = 0.1;

    let graph = random_graph(n, p);
    dbg!(graph.node_count());
    dbg!(graph.relationship_count());

    let pattern = GraphBuilder::new()
        .add_node(0, "fixed")
        .add_node(1, "fixed")
        .add_node(2, "fixed")
        .add_relationship(0, 1)
        .add_relationship(1, 0)
        .add_relationship(1, 2)
        .build();

    c.bench_with_input(
        BenchmarkId::new("simple_iso", format!("random_graph n = {}, p = {}", n, p)),
        &(&graph, &pattern),
        |b, g| b.iter(|| simple_iso_bench(black_box(g))),
    );

    c.bench_with_input(
        BenchmarkId::new("dual_iso", format!("random_graph n = {}, p = {}", n, p)),
        &(&graph, &pattern),
        |b, g| b.iter(|| dual_iso_bench(black_box(g))),
    );
}

fn simple_iso_bench(input: &(&Graph<&str>, &Graph<&str>)) -> usize {
    let (graph, pattern) = input;
    let matches = simple_iso(&graph, &pattern);
    matches.len()
}

fn dual_iso_bench(input: &(&Graph<&str>, &Graph<&str>)) -> usize {
    let (graph, pattern) = input;
    let matches = dual_iso(&graph, &pattern);
    matches.len()
}

fn random_graph(n: usize, p: f64) -> Graph<&'static str> {
    let mut graph_builder = GraphBuilder::new();
    let mut rng = SmallRng::seed_from_u64(1337);

    // generate nodes
    for node_id in 0..n {
        graph_builder.add_node(node_id, "fixed");
    }

    // generate relationships
    for source_id in 0..n {
        for target_id in 0..n {
            if rng.gen_range(0.0, 1.0) < p {
                graph_builder.add_relationship(source_id, target_id);
            }
        }
    }

    graph_builder.build()
}
