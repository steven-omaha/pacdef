use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn get_all_installed_packages(c: &mut Criterion) {
    c.bench_function("get_all_installed_packages", |b| {
        b.iter(|| {
            black_box(pacdef::backend::get_all_installed_packages());
        })
    });
}

fn get_explicitly_installed_packages(c: &mut Criterion) {
    c.bench_function("get_explicitly_installed_packages", |b| {
        b.iter(|| {
            black_box(pacdef::backend::get_explicitly_installed_packages());
        })
    });
}

criterion_group!(
    benches,
    get_all_installed_packages,
    get_explicitly_installed_packages
);
criterion_main!(benches);
