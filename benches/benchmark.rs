use mandelbrot::{Region, generate};
use criterion::{criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let region = Region {
        img_w: 100,
        img_h: 100,
        real_min: -2.0,
        real_max: 1.0,
        im_min: -1.5,
        im_max: 1.5,
    };
    c.bench_function("generate 100", |b| b.iter(|| generate(&region, 1000, || {})));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
