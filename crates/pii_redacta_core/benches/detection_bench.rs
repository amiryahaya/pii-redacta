//! Detection benchmarks
//!
//! Sprint 2: Pattern-Based Detection Engine

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pii_redacta_core::detection::PatternDetector;

fn bench_email_detection(c: &mut Criterion) {
    let detector = PatternDetector::new();
    let text = "Contact john@example.com or jane@test.org for details";

    c.bench_function("detect_email", |b| {
        b.iter(|| detector.detect_email(black_box(text)));
    });
}

fn bench_nric_detection(c: &mut Criterion) {
    let detector = PatternDetector::new();
    let text = "IC: 850101-14-5123 and 900202-13-4122";

    c.bench_function("detect_nric", |b| {
        b.iter(|| detector.detect_nric(black_box(text)));
    });
}

fn bench_phone_detection(c: &mut Criterion) {
    let detector = PatternDetector::new();
    let text = "Call 012-3456789 or +60 12-345 6789";

    c.bench_function("detect_phone", |b| {
        b.iter(|| detector.detect_phone(black_box(text)));
    });
}

fn bench_credit_card_detection(c: &mut Criterion) {
    let detector = PatternDetector::new();
    let text = "Card: 4532-1234-5678-9012 or 4532123456789012";

    c.bench_function("detect_credit_card", |b| {
        b.iter(|| detector.detect_credit_card(black_box(text)));
    });
}

fn bench_full_detection(c: &mut Criterion) {
    let detector = PatternDetector::new();
    let text = "Email: a@b.com, IC: 850101-14-5123, Phone: 012-3456789, Card: 4532-1234-5678-9012";

    c.bench_function("detect_all", |b| {
        b.iter(|| detector.detect_all(black_box(text)));
    });
}

criterion_group!(
    benches,
    bench_email_detection,
    bench_nric_detection,
    bench_phone_detection,
    bench_credit_card_detection,
    bench_full_detection
);
criterion_main!(benches);
