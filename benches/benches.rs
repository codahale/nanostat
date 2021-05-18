extern crate criterion;
extern crate nanostat;

use criterion::{criterion_group, criterion_main, Criterion};

use nanostat::Summary;

fn summarize(c: &mut Criterion) {
    let v = vec![0.0; 1000];
    c.bench_function("summarize", move |b| {
        b.iter(|| v.iter().collect::<Summary>())
    });
}

fn compare(c: &mut Criterion) {
    let s1: Summary = vec![0.0; 10].iter().collect();
    let s2: Summary = vec![0.1; 10].iter().collect();

    c.bench_function("compare", move |b| {
        b.iter(|| s1.compare(&s2, nanostat::Confidence::P98))
    });
}

criterion_group!(benches, summarize, compare);
criterion_main!(benches);
