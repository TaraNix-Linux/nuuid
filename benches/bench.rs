use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::str::FromStr;
use uuid::{Rng, Uuid};
use uuid_::Uuid as Uuid_;

fn new_v4(c: &mut Criterion) {
    let mut rng = Rng::new();
    let mut group = c.benchmark_group("new_v4");
    group.throughput(Throughput::Elements(1));
    group.bench_function("Uuid", |b| b.iter(|| Uuid::new_v4_rng(&mut rng)));
    // NOTE: This uses thread_rng, whereas our new_v4 is new each time.
    // So our bench uses new_v4_rng, which is local and seeded once at the start
    // by OsRng, similar to thread_rng.
    group.bench_function("Uuid_", |b| b.iter(Uuid_::new_v4));
}

fn new_v5(c: &mut Criterion) {
    let namespace = Uuid::from_bytes(*Uuid_::NAMESPACE_DNS.as_bytes());
    let name = b"example";
    let input = (namespace, name);
    let mut group = c.benchmark_group("new_v5");
    group.throughput(Throughput::Elements(1));
    group.bench_with_input("Uuid", &input, |b, (namespace, name)| {
        b.iter(|| Uuid::new_v5(*namespace, *name))
    });
    group.bench_with_input("Uuid_", &input, |b, (_, name)| {
        b.iter(|| Uuid_::new_v5(&Uuid_::NAMESPACE_DNS, *name))
    });
}

fn parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("from_str");
    group.throughput(Throughput::Elements(1));
    let mut buf = [0; 36];
    let input = Uuid::new_v4();
    let input = input.to_str(&mut buf);

    group.bench_with_input("Uuid", input, |b, i| b.iter(|| Uuid::from_str(i)));
    group.bench_with_input("Uuid_", input, |b, i| b.iter(|| Uuid_::from_str(i)));
}

criterion_group!(benches, new_v4, new_v5, parse);
criterion_main!(benches);
