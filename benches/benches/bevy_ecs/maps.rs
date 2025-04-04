use bevy_ecs::storage::{SparseSet, SparseSetIndex};
use bevy_platform_support::{collections::HashMap, hash::Hashed};
use bevy_utils::PreHashMap;
use core::hint::black_box;
use criterion::*;
use std::{hash::Hash, num::NonZero};

criterion_group!(
    benches,
    mapping_types_lookups_hits::<Small>,
    mapping_types_lookups_hits::<Big>,
    mapping_types_lookups_hits::<BigValue>,
    mapping_types_lookups_hits::<SmallNiche>
);

fn build_dense_hash_map<T: BenchingType>(num: usize) -> (HashMap<T, T::Stored>, Vec<T>) {
    let mut map = HashMap::default();
    let mut valid = Vec::with_capacity(num);
    for i in 0..num {
        let (k, v) = T::from_index(i);
        map.insert(k, v);
        valid.push(k);
    }
    black_box((map, valid))
}

fn build_sparse_hash_map<T: BenchingType>(num: usize) -> (HashMap<T, T::Stored>, Vec<T>) {
    let mut map = HashMap::default();
    let mut valid = Vec::with_capacity(num);
    for i in 0..num {
        let (k, v) = T::from_index(i * i);
        map.insert(k, v);
        valid.push(k);
    }
    black_box((map, valid))
}

fn build_arbitrary_hash_map<T: BenchingType>(num: usize) -> (HashMap<T, T::Stored>, Vec<T>) {
    let mut map = HashMap::default();
    let mut valid = Vec::with_capacity(num);
    for i in 0..(num / 2) {
        let (k, v) = T::from_index(i);
        map.insert(k, v);
        valid.push(k);
        let (k, v) = T::from_index(i * i);
        map.insert(k, v);
        valid.push(k);
    }
    black_box((map, valid))
}

const SIZES: &[usize] = &[64, 512, 1024, 4096];

fn mapping_types_lookups_hits<T: BenchingType>(c: &mut Criterion) {
    for &size in SIZES {
        let test_data_dense = build_dense_hash_map::<T>(size);
        let test_data_sparse = build_sparse_hash_map::<T>(size);
        let test_data_arbitrary = build_arbitrary_hash_map::<T>(size);
        let tests = [
            (test_data_dense.0, test_data_dense.1, "dense"),
            (test_data_sparse.0, test_data_sparse.1, "sparse"),
            (test_data_arbitrary.0, test_data_arbitrary.1, "arbitrary"),
        ];

        for (map, valid, name) in &tests {
            let mut sparse_set_group = c.benchmark_group(format!(
                "map_hits_of_{}_{}_`{}`s",
                size,
                name,
                core::any::type_name::<T>()
                    .split("::")
                    .last()
                    .unwrap_or_else(|| core::any::type_name::<T>())
            ));
            sparse_set_group.warm_up_time(core::time::Duration::from_millis(500));
            sparse_set_group.measurement_time(core::time::Duration::from_secs(4));

            sparse_set_group.bench_function("sparse_set", |x| {
                let map = map_to_sparse_set::<T>(map);
                x.iter(|| {
                    let mut result = T::Stored::default();
                    for &i in valid {
                        let val = map.get(i).copied().unwrap_or_default();
                        T::roll(val, &mut result);
                    }
                    result
                });
            });

            sparse_set_group.bench_function("hashbrown", |x| {
                let map = map_to_hashbrown::<T>(map);
                x.iter(|| {
                    let mut result = T::Stored::default();
                    for i in valid {
                        let val = map.get(i).copied().unwrap_or_default();
                        T::roll(val, &mut result);
                    }
                    result
                });
            });

            sparse_set_group.bench_function("pre-hashmap", |x| {
                let map = map_to_pre_hash::<T>(map);
                x.iter(|| {
                    let mut result = T::Stored::default();
                    for &i in valid {
                        let val = map.get(&Hashed::new(i)).copied().unwrap_or_default();
                        T::roll(val, &mut result);
                    }
                    result
                });
            });
        }
    }
}

fn map_to_sparse_set<T: BenchingType>(map: &HashMap<T, T::Stored>) -> SparseSet<T, T::Stored> {
    let mut set = SparseSet::new();
    for (k, v) in map.iter() {
        set.insert(*k, *v);
    }
    black_box(set)
}

fn map_to_hashbrown<T: BenchingType>(map: &HashMap<T, T::Stored>) -> HashMap<T, T::Stored> {
    black_box(map.clone())
}

fn map_to_pre_hash<T: BenchingType>(map: &HashMap<T, T::Stored>) -> PreHashMap<T, T::Stored> {
    let mut hashmap = PreHashMap::default();
    for (k, v) in map.iter() {
        hashmap.insert(Hashed::new(*k), *v);
    }
    black_box(hashmap)
}

trait BenchingType: Hash + Clone + Copy + PartialEq + Eq + SparseSetIndex + 'static {
    type Stored: Default + Clone + Copy + 'static;
    fn from_index(index: usize) -> (Self, Self::Stored);
    fn roll(val: Self::Stored, into: &mut Self::Stored);
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Small(u32);

impl SparseSetIndex for Small {
    fn sparse_set_index(&self) -> usize {
        self.0 as usize
    }

    fn get_sparse_set_index(value: usize) -> Self {
        Self(value as u32)
    }
}

impl BenchingType for Small {
    type Stored = u32;

    fn from_index(index: usize) -> (Self, Self::Stored) {
        (Self(index as u32), index as u32)
    }

    fn roll(val: Self::Stored, into: &mut Self::Stored) {
        *into += val;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct BigValue(u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
struct BigValueStored {
    x: u128,
    x1: u128,
    x2: u128,
    x3: u128,
    x4: u128,
    x5: u128,
    x6: u128,
    x7: u128,
}

impl SparseSetIndex for BigValue {
    fn sparse_set_index(&self) -> usize {
        self.0 as usize
    }

    fn get_sparse_set_index(value: usize) -> Self {
        Self(value as u32)
    }
}

impl BenchingType for BigValue {
    type Stored = BigValueStored;

    fn from_index(index: usize) -> (Self, Self::Stored) {
        (
            Self(index as u32),
            BigValueStored {
                x: index as u128,
                ..Default::default()
            },
        )
    }

    fn roll(val: Self::Stored, into: &mut Self::Stored) {
        into.x += val.x;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Big(u64);

impl SparseSetIndex for Big {
    fn sparse_set_index(&self) -> usize {
        self.0 as usize
    }

    fn get_sparse_set_index(value: usize) -> Self {
        Self(value as u64)
    }
}

impl BenchingType for Big {
    type Stored = u64;

    fn from_index(index: usize) -> (Self, Self::Stored) {
        (Self(index as u64), index as u64)
    }

    fn roll(val: Self::Stored, into: &mut Self::Stored) {
        *into += val;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct SmallNiche(NonZero<u32>);

impl SparseSetIndex for SmallNiche {
    fn sparse_set_index(&self) -> usize {
        self.0.get() as usize
    }

    fn get_sparse_set_index(value: usize) -> Self {
        Self(NonZero::new(value as u32).unwrap_or(NonZero::<u32>::MIN))
    }
}

impl BenchingType for SmallNiche {
    type Stored = u32;

    fn from_index(index: usize) -> (Self, Self::Stored) {
        (
            Self(NonZero::<u32>::new((index + 1) as u32).unwrap_or(NonZero::<u32>::MIN)),
            index as u32,
        )
    }

    fn roll(val: Self::Stored, into: &mut Self::Stored) {
        *into += val;
    }
}
