use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use memlink_msdk::prelude::*;
use std::hint::black_box;

fn create_test_context() -> CallContext<'static> {
    let arena = Box::leak(Box::new(unsafe {
        let buf = vec![0u8; 8192].into_boxed_slice();
        let ptr = Box::into_raw(buf) as *mut u8;
        Arena::new(ptr, 8192)
    }));
    CallContext::new(arena, 0.0, 0, 0, None, None)
}

fn bench_function_call(c: &mut Criterion) {
    let mut group = c.benchmark_group("function_call");
    group.throughput(Throughput::Elements(1));
    let ctx = create_test_context();

    group.bench_function("echo_empty", |b| {
        b.iter(|| {
            let _ = black_box(echo(&ctx, String::new()));
        });
    });

    group.bench_function("echo_small", |b| {
        b.iter(|| {
            let _ = black_box(echo(&ctx, "hello".to_string()));
        });
    });

    group.finish();
}

fn bench_arena(c: &mut Criterion) {
    let mut group = c.benchmark_group("arena");
    group.throughput(Throughput::Elements(1));
    let ctx = create_test_context();

    group.bench_function("alloc_u64", |b| {
        b.iter(|| {
            let _ = black_box(ctx.arena().alloc::<u64>());
        });
    });

    group.bench_function("alloc_with", |b| {
        b.iter(|| {
            let _ = black_box(ctx.arena().alloc_with(42u64));
        });
    });

    group.finish();
}

fn echo(_ctx: &CallContext, input: String) -> Result<String> {
    Ok(input)
}

criterion_group!(benches, bench_function_call, bench_arena);
criterion_main!(benches);
