use criterion::{Criterion, criterion_group, criterion_main};
use fastack::Stack;
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

fn fastack(threads: usize) {
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

macro_rules! generate_benchmark {
    ($name: ident, $number: expr) => {
        fn $name(c: &mut Criterion) {
            let mut group = c.benchmark_group("Bravo");
            group.bench_function("Std", |b| b.iter(|| std_mutex_list($number)));
            group.bench_function("Fastack", |b| b.iter(|| fastack($number)));
        }
    };
}

generate_benchmark!(benchmark1, 10);
generate_benchmark!(benchmark2, 50);
generate_benchmark!(benchmark3, 150);

criterion_group! {name = benchmarks; config = Criterion::default(); targets = benchmark1, benchmark2, benchmark3}
criterion_main!(benchmarks);
