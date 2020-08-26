// Copyright 2020 TiKV Project Authors. Licensed under Apache-2.0.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn dummy_iter(i: usize) {
    #[minitrace::trace(0u32)]
    fn dummy() {}

    for _ in 0..i - 1 {
        dummy();
    }
}

#[minitrace::trace(0u32)]
fn dummy_rec(i: usize) {
    if i > 1 {
        dummy_rec(i - 1);
    }
}

fn trace_wide_bench(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "trace_wide",
        |b, len| {
            b.iter(|| {
                let _root = minitrace::start_trace(0u32);

                if *len > 1 {
                    dummy_iter(*len);
                }
            });

            minitrace::collect_all();
        },
        vec![1, 10, 100, 1000, 10000],
    );
}

fn trace_deep_bench(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "trace_deep",
        |b, len| {
            b.iter(|| {
                let _root = minitrace::start_trace(0u32);

                if *len > 1 {
                    dummy_rec(*len);
                }
            });

            minitrace::collect_all();
        },
        vec![1, 10, 100, 1000, 10000],
    );
}

fn bench_collect(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "bench_collect",
        |b, len| {
            {
                let _root = minitrace::start_trace(0u32);

                if *len > 1 {
                    dummy_rec(*len);
                }
            }

            b.iter(|| black_box(minitrace::collect_all()));
        },
        vec![1, 10, 100, 1000, 10000],
    );
}

fn trace_future_bench(c: &mut Criterion) {
    use minitrace::future::FutureExt;

    async fn f(i: u32) {
        for i in 0..i - 1 {
            async {}.in_new_span(black_box(i)).await
        }
    }

    c.bench_function_over_inputs(
        "trace_future",
        |b, len| {
            b.iter(|| {
                let _root = minitrace::start_trace(0u32);

                let _ = futures_03::executor::block_on(f(*len).in_new_span(0u32));
            });

            minitrace::collect_all();
        },
        vec![1, 10, 100, 1000, 10000],
    );
}

fn trace_start_context(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "trace_context",
        |b, len| {
            b.iter(|| {
                let _root = minitrace::start_trace(0u32);

                for _ in 0..*len {
                    let _guard = black_box(minitrace::new_async_span());
                }
            });

            minitrace::collect_all();
        },
        vec![1, 10, 100, 1000, 10000],
    );
}

criterion_group!(
    benches,
    trace_wide_bench,
    trace_deep_bench,
    bench_collect,
    trace_start_context,
    trace_future_bench
);
criterion_main!(benches);
