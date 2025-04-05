use bevy_ecs::prelude::*;
use core::hint::black_box;
use criterion::*;

criterion_group!(benches, entity_allocation);

fn entity_allocation(c: &mut Criterion) {
    for count in [128, 1024] {
        bench_allocator(World::new, c, count, "entity_allocator_fresh");

        bench_allocator(
            || {
                let mut world = World::new();
                // SAFETY: We're just freeing them
                let entiteis = unsafe { world.entities_mut() };
                let reserved = entiteis.reserve_entities(count).collect::<Vec<_>>();
                entiteis.flush_as_invalid();
                for entity in reserved {
                    entiteis.free(entity);
                }
                world
            },
            c,
            count,
            "entity_allocator_batched_and_freed",
        );
    }
}

fn bench_allocator(
    mut constructor: impl FnMut() -> World,
    c: &mut Criterion,
    count: u32,
    name: &str,
) {
    let mut group = c.benchmark_group(format!("{name}_{count}"));
    group.warm_up_time(core::time::Duration::from_millis(500));
    group.measurement_time(core::time::Duration::from_secs(4));

    group.bench_function("allocate", |bencher| {
        bencher.iter(|| {
            let mut world = constructor();
            // SAFETY: It's a benchmark.
            let entities = black_box(unsafe { world.entities_mut() });
            for _ in 0..count {
                black_box(entities.alloc());
            }
        });
    });
    group.bench_function("allocate_with_flush", |bencher| {
        bencher.iter(|| {
            let mut world = constructor();
            // SAFETY: It's a benchmark.
            let entities = black_box(unsafe { world.entities_mut() });
            for _ in 0..count {
                entities.flush_as_invalid();
                black_box(entities.alloc());
            }
        });
    });
    group.bench_function("reserve", |bencher| {
        bencher.iter(|| {
            let mut world = constructor();
            // SAFETY: It's a benchmark.
            let entities = black_box(unsafe { world.entities_mut() });
            for _ in 0..count {
                black_box(entities.reserve_entity());
            }
        });
    });
    group.bench_function("reserve_batch", |bencher| {
        bencher.iter(|| {
            let mut world = constructor();
            // SAFETY: It's a benchmark.
            let entities = black_box(unsafe { world.entities_mut() });
            black_box(entities.reserve_entities(count));
        });
    });
    group.bench_function("reserve_batch_and_flush", |bencher| {
        bencher.iter(|| {
            let mut world = constructor();
            // SAFETY: It's a benchmark.
            let entities = black_box(unsafe { world.entities_mut() });
            black_box(entities.reserve_entities(count));
            entities.flush_as_invalid();
        });
    });
}
