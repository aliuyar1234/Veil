//! Benchmarks for document parsing operations.
//!
//! Run with: `cargo bench -p veil-parsers`

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use veil_parsers::{parse_bytes, FileFormat, ParseOptions};

fn generate_text_content(lines: usize) -> Vec<u8> {
    let sample_lines = vec![
        "This is a sample line of text for benchmarking purposes.",
        "It contains various words, numbers like 12345, and punctuation.",
        "Email addresses like test@example.com might appear here.",
        "Phone numbers: +1-555-123-4567 are also common in text.",
        "Regular sentences without any special data are most common.",
    ];

    let content: String = (0..lines)
        .map(|i| sample_lines[i % sample_lines.len()])
        .collect::<Vec<_>>()
        .join("\n");

    content.into_bytes()
}

fn generate_csv_content(rows: usize) -> Vec<u8> {
    let mut content = String::from("name,email,phone,address\n");

    for i in 0..rows {
        content.push_str(&format!(
            "User{},user{}@example.com,+1-555-{:03}-{:04},\"123 Main St, City\"\n",
            i,
            i,
            i % 1000,
            i % 10000
        ));
    }

    content.into_bytes()
}

fn generate_json_content(items: usize) -> Vec<u8> {
    let mut content = String::from("[\n");

    for i in 0..items {
        if i > 0 {
            content.push_str(",\n");
        }
        content.push_str(&format!(
            r#"  {{"id": {}, "name": "User{}", "email": "user{}@example.com", "active": true}}"#,
            i, i, i
        ));
    }

    content.push_str("\n]");
    content.into_bytes()
}

fn bench_text_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_text");

    for lines in [100, 1000, 10000].iter() {
        let content = generate_text_content(*lines);
        let options = ParseOptions {
            format: Some(FileFormat::Text),
            ..Default::default()
        };

        group.throughput(Throughput::Bytes(content.len() as u64));
        group.bench_with_input(BenchmarkId::new("lines", lines), &content, |b, data| {
            b.iter(|| parse_bytes(black_box(data), black_box(&options)))
        });
    }

    group.finish();
}

fn bench_csv_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_csv");

    for rows in [100, 1000, 10000].iter() {
        let content = generate_csv_content(*rows);
        let options = ParseOptions {
            format: Some(FileFormat::Csv),
            ..Default::default()
        };

        group.throughput(Throughput::Bytes(content.len() as u64));
        group.bench_with_input(BenchmarkId::new("rows", rows), &content, |b, data| {
            b.iter(|| parse_bytes(black_box(data), black_box(&options)))
        });
    }

    group.finish();
}

fn bench_json_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_json");

    for items in [100, 1000, 5000].iter() {
        let content = generate_json_content(*items);
        let options = ParseOptions {
            format: Some(FileFormat::Json),
            ..Default::default()
        };

        group.throughput(Throughput::Bytes(content.len() as u64));
        group.bench_with_input(BenchmarkId::new("items", items), &content, |b, data| {
            b.iter(|| parse_bytes(black_box(data), black_box(&options)))
        });
    }

    group.finish();
}

fn bench_format_detection(c: &mut Criterion) {
    let text = generate_text_content(100);
    let csv = generate_csv_content(100);
    let json = generate_json_content(100);

    c.bench_function("detect_format_text", |b| {
        b.iter(|| veil_parsers::detect_format(black_box(&text), black_box(Some("test.txt"))))
    });

    c.bench_function("detect_format_csv", |b| {
        b.iter(|| veil_parsers::detect_format(black_box(&csv), black_box(Some("test.csv"))))
    });

    c.bench_function("detect_format_json", |b| {
        b.iter(|| veil_parsers::detect_format(black_box(&json), black_box(Some("test.json"))))
    });
}

criterion_group!(
    benches,
    bench_text_parsing,
    bench_csv_parsing,
    bench_json_parsing,
    bench_format_detection
);

criterion_main!(benches);
