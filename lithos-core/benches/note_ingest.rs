//! Markdown ingestion benchmarks for the note context.
//!
//! # Safety
//! Benchmark code uses unwrap/expect for simplicity.

#![allow(
    missing_docs,
    clippy::expect_used,
    clippy::disallowed_methods,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    reason = "Criterion benchmarks prefer direct control flow with asserts"
)]

use criterion::{
    Criterion, Throughput, black_box, criterion_group, criterion_main,
};
use lithos_core::{
    config::{
        aggregate::Config,
        raw::RawConfig,
        vault::{VaultId, VaultRoot},
    },
    note::{
        aggregate::{Note, NoteId},
        parser::NoteParser,
    },
};

fn sample_markdown() -> &'static str {
    concat!(
        "# Title\n\n",
        "- [ ] #task Review PR [priority:: 1]\n",
        "- [x] Buy milk\n",
        "- [ ] Call mom\n\n",
        "1. First\n",
        "2. Second\n",
    )
}

fn bench_note_ingest(c: &mut Criterion) {
    let config = Config::build(
        &RawConfig::default(),
        VaultId::new(),
        VaultRoot::try_new(std::path::PathBuf::from("/vault"))
            .expect("valid vault root"),
    )
    .expect("config");
    let markdown = sample_markdown();

    let mut group = c.benchmark_group("note_ingest");
    group.throughput(Throughput::Bytes(markdown.len() as u64));

    group.bench_function("ingest_markdown", |b| {
        b.iter(|| {
            let mut note = Note::new(NoteId::new(), "notes/bench.md")
                .expect("valid note path");
            NoteParser::new(&config)
                .apply(&mut note, black_box(markdown))
                .expect("ingest markdown");
            black_box(note);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_note_ingest);
criterion_main!(benches);
