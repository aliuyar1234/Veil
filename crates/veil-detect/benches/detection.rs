//! Benchmarks for PII detection performance.
//!
//! Run with: `cargo bench -p veil-detect`

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use veil_detect::DetectorRegistry;
use veil_types::{Position, TextSegment};

fn make_segment(content: &str) -> TextSegment {
    TextSegment {
        content: content.to_string().into(),
        position: Position::Text {
            line: 1,
            column: 1,
            byte_offset: 0,
            byte_length: content.len(),
        },
    }
}

fn make_segment_owned(content: String) -> TextSegment {
    let byte_length = content.len();
    TextSegment {
        content: content.into(),
        position: Position::Text {
            line: 1,
            column: 1,
            byte_offset: 0,
            byte_length,
        },
    }
}

fn generate_test_content(lines: usize) -> Vec<TextSegment> {
    let test_lines = vec![
        "Contact John at john.doe@example.com for more information.",
        "Phone: +1-555-123-4567, Fax: +1-555-765-4321",
        "IBAN: DE89370400440532013000 for wire transfers.",
        "Credit card ending in 4242 (4111111111111111)",
        "SSN: 123-45-6789 (do not share)",
        "Please email support@company.org with questions.",
        "German tax ID: 12 345 678 901",
        "Regular text without any PII data here.",
        "More regular text to dilute the density.",
        "Final line with email: test@test.com",
    ];

    (0..lines)
        .map(|i| make_segment(test_lines[i % test_lines.len()]))
        .collect()
}

fn generate_large_text(bytes: usize) -> String {
    let clean_line = "This is a normal line of text without sensitive data.\n";
    let pii_line = "Contact: john.doe@example.com, Phone: +1-555-123-4567\n";

    let mut content = String::with_capacity(bytes + pii_line.len());
    let mut i = 0usize;
    while content.len() < bytes {
        content.push_str(clean_line);
        if i % 50 == 0 {
            content.push_str(pii_line);
        }
        i += 1;
    }
    content.truncate(bytes);
    content
}

fn bench_sequential_detection(c: &mut Criterion) {
    let registry = DetectorRegistry::default();

    let mut group = c.benchmark_group("detection_sequential");

    for size in [10, 100, 1000].iter() {
        let segments = generate_test_content(*size);
        let bytes: usize = segments.iter().map(|s| s.content.len()).sum();

        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_with_input(BenchmarkId::new("segments", size), &segments, |b, segs| {
            b.iter(|| registry.detect_all(black_box(segs)))
        });
    }

    group.finish();
}

fn bench_large_text_segment(c: &mut Criterion) {
    let registry = DetectorRegistry::default();

    let mut group = c.benchmark_group("detection_large_text");

    for bytes in [100_000usize, 1_000_000, 5_000_000].iter() {
        let content = generate_large_text(*bytes);
        let segment = make_segment_owned(content);
        let segments = vec![segment];

        group.throughput(Throughput::Bytes(*bytes as u64));
        group.bench_with_input(BenchmarkId::new("bytes", bytes), &segments, |b, segs| {
            b.iter(|| registry.detect_all(black_box(segs)))
        });
    }

    group.finish();
}

fn bench_many_small_segments(c: &mut Criterion) {
    let registry = DetectorRegistry::default();

    let clean_content = "Regular text without PII.";
    let pii_content = "Contact: john.doe@example.com";

    let mut group = c.benchmark_group("detection_many_small_segments");

    for count in [1_000usize, 10_000].iter() {
        let segments: Vec<TextSegment> = (0..*count)
            .map(|i| {
                make_segment(if i % 50 == 0 {
                    pii_content
                } else {
                    clean_content
                })
            })
            .collect();

        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::new("segments", count), &segments, |b, segs| {
            b.iter(|| registry.detect_all(black_box(segs)))
        });
    }

    group.finish();
}

#[cfg(feature = "parallel")]
fn bench_parallel_detection(c: &mut Criterion) {
    let registry = DetectorRegistry::default();

    let mut group = c.benchmark_group("detection_parallel");

    for size in [10, 100, 1000].iter() {
        let segments = generate_test_content(*size);
        let bytes: usize = segments.iter().map(|s| s.content.len()).sum();

        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_with_input(BenchmarkId::new("segments", size), &segments, |b, segs| {
            b.iter(|| registry.detect_all_parallel(black_box(segs)))
        });
    }

    group.finish();
}

fn bench_dense_pii(c: &mut Criterion) {
    let registry = DetectorRegistry::default();

    // Content with high PII density
    let dense_content = "Email: a@b.com, b@c.org, c@d.net. \
        Phone: +1-111-111-1111, +49-222-222-2222. \
        IBAN: DE89370400440532013000, FR7630006000011234567890189. \
        Cards: 4111111111111111, 5500000000000004.";

    let segments = vec![make_segment(dense_content)];

    c.bench_function("dense_pii_single_segment", |b| {
        b.iter(|| registry.detect_all(black_box(&segments)))
    });
}

fn bench_no_pii(c: &mut Criterion) {
    let registry = DetectorRegistry::default();

    // Content without any PII
    let clean_content = "This is a regular text without any personally identifiable \
        information. It contains normal words, numbers like 42 and 100, \
        and punctuation marks. No emails, phones, or credit cards here.";

    let segments: Vec<TextSegment> = (0..100).map(|_| make_segment(clean_content)).collect();

    c.bench_function("no_pii_100_segments", |b| {
        b.iter(|| registry.detect_all(black_box(&segments)))
    });
}

#[cfg(feature = "parallel")]
criterion_group!(
    benches,
    bench_sequential_detection,
    bench_large_text_segment,
    bench_many_small_segments,
    bench_parallel_detection,
    bench_dense_pii,
    bench_no_pii
);

#[cfg(not(feature = "parallel"))]
criterion_group!(
    benches,
    bench_sequential_detection,
    bench_large_text_segment,
    bench_many_small_segments,
    bench_dense_pii,
    bench_no_pii
);

criterion_main!(benches);
