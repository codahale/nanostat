extern crate criterion;
extern crate nanostat;

use criterion::{criterion_group, criterion_main, Criterion};

fn summarize(c: &mut Criterion) {
    let v = vec![0.0; 1000];
    c.bench_function("summarize", move |b| b.iter(|| nanostat::Summary::of(&v)));
}

fn compare(c: &mut Criterion) {
    let s1 = nanostat::Summary::of(&vec![0.0; 10]);
    let s2 = nanostat::Summary::of(&vec![0.1; 10]);

    c.bench_function("compare", move |b| {
        b.iter(|| s1.compare(&s2, nanostat::Confidence::P98))
    });
}

criterion_group!(benches, summarize, compare);
criterion_main!(benches);
