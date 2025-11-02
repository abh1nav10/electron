use criterion::{Criterion, criterion_group, criterion_main};
use ruby::Stack;
use std::collections::LinkedList as StdLinkedList;
use std::sync::Mutex;

fn std_mutex_list(threads: usize) {
    let new = &Mutex::new(StdLinkedList::new());
    std::thread::scope(|s| {
        for i in 0..threads {
            s.spawn(move || {
                new.lock().unwrap().push_front(i);
            });
        }
        for _ in 0..threads {
            s.spawn(move || {
                new.lock().unwrap().pop_front();
            });
        }
    });
}

fn ruby(threads: usize) {
    let new = &Stack::new();
    std::thread::scope(|s| {
        for i in 0..threads {
            s.spawn(move || {
                let _ = new.insert(i);
            });
        }
        for _ in 0..threads {
            s.spawn(move || {
                let _ = new.delete();
            });
        }
    });
}

fn benchmark1(c: &mut Criterion) {
    let mut group = c.benchmark_group("Bravo");
    group.bench_function("Std", |b| b.iter(|| std_mutex_list(10)));
    group.bench_function("Ruby", |b| b.iter(|| ruby(10)));
    group.finish();
}
fn benchmark2(c: &mut Criterion) {
    let mut group = c.benchmark_group("Bravo");
    group.bench_function("Std", |b| b.iter(|| std_mutex_list(50)));
    group.bench_function("Ruby", |b| b.iter(|| ruby(50)));
    group.finish();
}
fn benchmark3(c: &mut Criterion) {
    let mut group = c.benchmark_group("Bravo");
    group.bench_function("Std", |b| b.iter(|| std_mutex_list(150)));
    group.bench_function("Ruby", |b| b.iter(|| ruby(150)));
    group.finish();
}

criterion_group! {name = benchmarks; config = Criterion::default(); targets = benchmark1, benchmark2, benchmark3}
criterion_main!(benchmarks);
