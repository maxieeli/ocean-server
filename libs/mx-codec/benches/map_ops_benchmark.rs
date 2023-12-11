use std::time::Duration;
use criterion::{criterion_group, criterion_main, Criterion};

fn operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("ops/map");
    group.measurement_time(Duration::from_secs(15));

    group.bench_function("mx/insert", |b| {
        let base_text = "test1 test2"
            .split(" ")
            .collect::<Vec<_>>();
        b.iter(|| {
            use mx_codec::*;
            let doc = Doc::default();
            let mut map = doc.get_or_create_map("test").unwrap();
            for (idx, key) in base_text.iter().enumerate() {
                map.insert(key.to_string(), idx).unwrap();
            }
            for key in &base_text {
                map.remove(key);
            }
        });
    });
    group.finish();
}

criterion_group!(benches, operations);
criterion_main!(benches);
