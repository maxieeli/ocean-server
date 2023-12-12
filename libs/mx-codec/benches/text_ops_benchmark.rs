use std::time::Duration;
use criterion::{criterion_group, criterion_main, Criterion};
use rand::{Rng, SeedableRng};
use yrs::Doc;

fn operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("ops/text");
    group.measurement_time(Duration::from_secs(15));
    group.bench_function("mx/insert", |b| {
        let base_text = "test1 test";
        let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(1234);
        let idxs = (0..99)
            .into_iter()
            .map(|_| rng.gen_range(0..base_text.len() as u64))
            .collect::<Vec<_>>();
        b.iter(|| {
            use mx_codec::*;
            let doc = Doc::default();
            let mut text = doc.get_or_create_text("test").unwrap();
            text.insert(0, &base_text).unwrap();
            for idx in &idxs {
                text.insert(*idx, "test").unwrap();
            }
        });
    });

    group.bench_function("wx/remove", |b| {
        let base_text = "test1 test2";
        b.iter(|| {
            use mx_codec::*;
            let doc = Doc::default();
            let mut text = doc.get_or_create_text("test").unwrap();
            text.insert(0, &base_text).unwrap();
            text.insert(0, &base_text).unwrap();
            text.insert(0, &base_text).unwrap();
            for idx in (base_text.len() as u64)..0 {
                text.remove(idx, 1).unwrap();
            }
        })
    });
    group.finish();
}

criterion_group!(benches, operations);
criterion_main!(benches);
