#![allow(dead_code)]
use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use nuuid::{Rng, Uuid};
use std::str::FromStr;
use uuid_::{Builder, Uuid as Uuid_};

fn new_v4(c: &mut Criterion) {
    let mut group = c.benchmark_group("Generating new_v4 UUIDs");
    group.throughput(Throughput::Elements(1));
    let mut rng = Rng::new();

    // NOTE: The uuid crate uses getrandom directly
    // Nuuid uses it through rand's `OsRng`

    group.bench_function("Nuuid::new_v4", |b| b.iter(Uuid::new_v4));
    group.bench_function("Nuuid::new_v4_rng", |b| {
        b.iter(|| Uuid::new_v4_rng(&mut rng))
    });

    group.bench_function("Uuid::new_v4", |b| b.iter(Uuid_::new_v4));
    // NOTE: Justification for this comparison vs new_v4_rng is that
    // the uuid crate is incapable of doing it as new_v4_rng does.
    group.bench_function("Builder::from_random_bytes", |b| {
        b.iter(|| {
            use rand::RngCore;
            let mut bytes = [0u8; 16];
            rand::rngs::OsRng.fill_bytes(&mut bytes);
            Builder::from_random_bytes(black_box(bytes))
        })
    });
}

fn new_v5(c: &mut Criterion) {
    let mut group = c.benchmark_group("new_v5");
    group.throughput(Throughput::Elements(1));

    group.bench_function("Nuuid::new_v5", |b| {
        b.iter(|| Uuid::new_v5(nuuid::NAMESPACE_DNS, black_box(b"example")))
    });
    group.bench_function("Uuid::new_v5", |b| {
        b.iter(|| Uuid_::new_v5(&Uuid_::NAMESPACE_DNS, black_box(b"example")))
    });
}

fn from_str(c: &mut Criterion) {
    let mut group = c.benchmark_group("Constructing UUIDs from strings");
    group.throughput(Throughput::Elements(1));
    let mut buf = [0; 36];
    let input = Uuid::new_v4();
    let input = input.to_str(&mut buf);

    group.bench_with_input("Nuuid::from_str(upper hex)", input, |b, i| {
        b.iter(|| Uuid::from_str(i))
    });
    group.bench_with_input("Uuid::from_str(upper hex)", input, |b, i| {
        b.iter(|| Uuid_::from_str(i))
    });
    group.finish();
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
                uuid_.hyphenated().encode_lower(buf);
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

criterion_group!(
    benches, //
    new_v4, new_v5, from_str, to_str, inline
);
criterion_main!(benches);
