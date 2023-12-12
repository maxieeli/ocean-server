use std::time::Duration;
use criterion::{criterion_group, criterion_main, Criterion};
use rand::{Rng, SeedableRng};
use yrs::Doc;

fn operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("ops/array");
    group.measurement_time(Duration::from_secs(15));
    group.bench_function("mx/insert", |b| {
        let base_text = "test1 test2";
        let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(1234);
        let idxs = (0..99)
            .into_iter()
            .map(|_| rng.gen_range(0..base_text.len() as u64))
            .collect::<Vec<_>>();
        b.iter(|| {
            use mx_codec::*;
            let doc = Doc::default();
            let mut array = doc.get_or_create_array("test").unwrap();
            for c in base_text.chars() {
                array.push(c.to_string()).unwrap();
            }
            for idx in &idxs {
                array.insert(*idx, "test").unwrap();
            }
        });
    });

    group.bench_function("mx/insert range", |b| {
        let base_text = "test1 test2";
        let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(1234);
        let idxs = (0..99)
            .into_iter()
            .map(|_| rng.gen_range(0..base_text.len() as u64))
            .collect::<Vec<_>>();
        b.iter(|| {
            use mx_codec::*;
            let doc = Doc::default();
            let mut array = doc.get_or_create_array("test").unwrap();
            for c in base_text.chars() {
                array.push(c.to_string()).unwrap();
            }
            for idx in &idxs {
                array.insert(*idx, "test1").unwrap();
                array.insert(idx + 1, "test2").unwrap();
            }
        });
    });
    group.bench_function("mx/remove", |b| {
        let base_text = "test1 test2";
        b.iter(|| {
            use mx_codec::*;
            let doc = Doc::default();
            let mut array = doc.get_or_create_array("test").unwrap();
            for c in base_text.chars() {
                array.push(c.to_string()).unwrap();
            }
            for idx in (base_text.len() as u64)..0 {
                array.remove(idx, 1).unwrap();
            }
        });
    });
    group.finish();
}

criterion_group!(benches, operations);
criterion_main!(benches);
