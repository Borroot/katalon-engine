use criterion::{black_box, criterion_group, criterion_main, Criterion};
use katalon;

pub fn addition(c: &mut Criterion) {
    c.bench_function("addition", |b| {
        b.iter(|| katalon::addition(black_box(1), black_box(2)))
    });
}

criterion_group!(benches, addition);
criterion_main!(benches);
