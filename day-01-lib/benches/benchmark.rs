use criterion::{criterion_group, criterion_main, Criterion};

const INPUT: &str = include_str!("../../day-01/input.txt");

fn criterion_benchmark(c: &mut Criterion) {
    let elves = day_01_lib::parse_input(INPUT).unwrap();
    c.bench_function("parse_input", |b| {
        b.iter(|| day_01_lib::parse_input(INPUT).unwrap())
    });
    c.bench_function("parse_input_fancy", |b| {
        b.iter(|| day_01_lib::parse_input_fancy(INPUT).unwrap())
    });
    c.bench_function("find_max_calories", |b| {
        b.iter(|| day_01_lib::find_max_calories(&elves))
    });
    c.bench_function("find_max_calories_fancy", |b| {
        b.iter(|| day_01_lib::find_max_calories_fancy(&elves))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
