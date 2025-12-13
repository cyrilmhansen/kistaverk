// Performance test to demonstrate the Copy optimization
// Run with: cargo bench --bench number_performance

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use kistaverk_core::features::cas_types::Number;

fn benchmark_number_operations(c: &mut Criterion) {
    c.bench_function("number_arithmetic_without_clones", |b| {
        b.iter(|| {
            let a = Number::from_f64(10.0);
            let b = Number::from_f64(5.0);
            
            // These operations should be fast due to Copy optimization
            let sum = a + b;
            let diff = a - b;
            let product = a * b;
            let quotient = a / b;
            
            black_box((sum, diff, product, quotient));
        })
    });

    c.bench_function("number_cloning", |b| {
        b.iter(|| {
            let a = Number::from_f64(10.0);
            let b = Number::from_f64(5.0);
            
            // Explicit clones (should be optimized away for Fast variant)
            let a_clone = a.clone();
            let b_clone = b.clone();
            
            black_box((a_clone, b_clone));
        })
    });
}

criterion_group!(benches, benchmark_number_operations);
criterion_main!(benches);