use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use nuuid::{Rng, Uuid};
use rand_chacha::rand_core::{OsRng, RngCore};
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
            let mut bytes = [0u8; 16];
            OsRng.fill_bytes(&mut bytes);
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
    let mut group = c.benchmark_group("Constructing Strings from UUIDs (to_str)");
    group.throughput(Throughput::Elements(1));

    let uuid = Uuid::new_v4();
    let uuid_ = Uuid_::from_bytes(uuid.to_bytes());

    let mut buf = [0u8; 36];

    group.bench_function("Nuuid::to_str", |b| {
        b.iter(|| {
            uuid.to_str(&mut buf);
        });
        buf = [0u8; 36];
    });

    group.bench_function("Uuid::hyphenated().encode_lower()", |b| {
        b.iter(|| {
            uuid_.hyphenated().encode_lower(&mut buf);
        });
        buf = [0u8; 36];
    });
}

fn variant(c: &mut Criterion) {
    let mut group = c.benchmark_group("UUIDs Variant");
    group.throughput(Throughput::Elements(1));
    let input = Uuid::new_v4();
    let input2 = Uuid_::from_bytes(input.to_bytes());

    group.bench_with_input("Nuuid::variant", &input, |b, u| b.iter(|| u.variant()));
    group.bench_with_input("Uuid::get_variant", &input2, |b, u| {
        b.iter(|| u.get_variant())
    });
    group.finish();
}

fn version(c: &mut Criterion) {
    let mut group = c.benchmark_group("UUIDs Version");
    group.throughput(Throughput::Elements(1));
    let input = Uuid::new_v4();
    let input2 = Uuid_::from_bytes(input.to_bytes());

    group.bench_with_input("Nuuid::version", &input, |b, u| b.iter(|| u.version()));
    group.bench_with_input("Uuid::get_version", &input2, |b, u| {
        b.iter(|| u.get_version())
    });
    group.finish();
}

fn mixed_endian(c: &mut Criterion) {
    let mut group = c.benchmark_group("UUIDs mixed-endian performance");
    group.throughput(Throughput::Elements(1));
    let input = Uuid::new_v4();
    let bytes = input.to_bytes_me();

    group.bench_function("Nuuid::from_bytes_me", |b| {
        b.iter(|| Uuid::from_bytes_me(bytes));
    });

    group.bench_function("Nuuid::from_bytes", |b| {
        b.iter(|| Uuid::from_bytes(bytes));
    });

    group.bench_function("Uuid::from_bytes_le", |b| {
        b.iter(|| Uuid_::from_bytes_le(bytes));
    });

    group.finish();
}

criterion_group!(
    benches, //
    new_v4,
    new_v5,
    from_str,
    to_str,
    variant,
    version,
    mixed_endian
);
criterion_main!(benches);
