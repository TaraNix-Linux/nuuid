use criterion::{criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use nuuid::{Rng, Uuid};
use std::str::FromStr;
use uuid_::Uuid as Uuid_;

fn new_v4(c: &mut Criterion) {
    let mut rng = Rng::new();
    let mut group = c.benchmark_group("new_v4");
    group.throughput(Throughput::Elements(1));
    group.bench_function("Nuuid", |b| b.iter(|| Uuid::new_v4_rng(&mut rng)));
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
    group.bench_with_input("Nuuid", &input, |b, (namespace, name)| {
        b.iter(|| Uuid::new_v5(*namespace, *name))
    });
    group.bench_with_input("Uuid_", &input, |b, (_, name)| {
        b.iter(|| Uuid_::new_v5(&Uuid_::NAMESPACE_DNS, *name))
    });
}

fn from_str(c: &mut Criterion) {
    let mut group = c.benchmark_group("from_str");
    group.throughput(Throughput::Elements(1));
    let mut buf = [0; 36];
    let input = Uuid::new_v4();
    let input = input.to_str(&mut buf);

    group.bench_with_input("Nuuid", input, |b, i| b.iter(|| Uuid::from_str(i)));
    group.bench_with_input("Uuid_", input, |b, i| b.iter(|| Uuid_::from_str(i)));
}

fn to_str(c: &mut Criterion) {
    let mut group = c.benchmark_group("to_str");
    group.throughput(Throughput::Elements(1));

    let uuid = Uuid::new_v4();
    let uuid_ = Uuid_::from_bytes(uuid.to_bytes());

    group.bench_function("Nuuid", |b| {
        b.iter_batched_ref(
            || [0; 36],
            |buf| {
                uuid.to_str(buf);
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("Uuid_", |b| {
        b.iter_batched_ref(
            || [0; 36],
            |buf| {
                uuid_.to_hyphenated().encode_lower(buf);
            },
            BatchSize::SmallInput,
        )
    });
}

#[allow(unused_variables)]
fn inline(c: &mut Criterion) {
    let mut group = c.benchmark_group("inline");
    let uuid = Uuid::new_v4();
    let nil = Uuid::nil();
    group.bench_function("Debug", |b| b.iter(|| format!("{:?}", uuid)));
    // group.bench_function("UpperHex", |b| b.iter(|| format!("{:X}", uuid)));
    // group.bench_function("LowerHex", |b| b.iter(|| format!("{:x}", uuid)));
    // group.bench_function("variant", |b| b.iter(|| uuid.variant()));
    // group.bench_function("version", |b| b.iter(|| uuid.version()));
    // group.bench_function("non-nil", |b| b.iter(|| uuid.is_nil()));
    // group.bench_function("nil", |b| b.iter(|| nil.is_nil()));
    // group.bench_function("to_bytes", |b| b.iter(|| nil.to_bytes()));
    // group.bench_function("to_bytes_me", |b| b.iter(|| nil.to_bytes_me()));
    // group.bench_function("to_str", |b| {
    //     b.iter_batched_ref(
    //         || [0; 36],
    //         |buf| {
    //             nil.to_str(buf);
    //         },
    //         BatchSize::SmallInput,
    //     )
    // });
    // group.bench_function("to_urn", |b| {
    //     b.iter_batched_ref(
    //         || [0; 45],
    //         |buf| {
    //             nil.to_urn(buf);
    //         },
    //         BatchSize::SmallInput,
    //     )
    // });
    // group.bench_function("to_urn_upper", |b| {
    //     b.iter_batched_ref(
    //         || [0; 45],
    //         |buf| {
    //             nil.to_urn_upper(buf);
    //         },
    //         BatchSize::SmallInput,
    //     )
    // });
    // group.bench_function("to_str_upper", |b| {
    //     b.iter_batched_ref(
    //         || [0; 36],
    //         |buf| {
    //             nil.to_str_upper(buf);
    //         },
    //         BatchSize::SmallInput,
    //     )
    // });
}

criterion_group!(benches, new_v4, new_v5, from_str, to_str, inline);
criterion_main!(benches);
