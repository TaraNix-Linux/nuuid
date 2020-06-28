use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::str::FromStr;
use uuid::Uuid;
use uuid_::Uuid as Uuid_;

fn new(c: &mut Criterion) {
    let mut group = c.benchmark_group("new_v4");
    group.throughput(Throughput::Elements(1));
    group.bench_function("Uuid", |b| b.iter(Uuid::new_v4));
    group.bench_function("Uuid_", |b| b.iter(Uuid_::new_v4));
}

fn parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("from_str");
    group.throughput(Throughput::Elements(1));
    let mut buf = [0; 36];
    let input = Uuid::new_v4();
    let input = input.to_string(&mut buf);

    group.bench_with_input("Uuid", input, |b, i| b.iter(|| Uuid::from_str(i)));
    group.bench_with_input("Uuid_", input, |b, i| b.iter(|| Uuid_::from_str(i)));
}

criterion_group!(benches, new, parse);
criterion_main!(benches);
