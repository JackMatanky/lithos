use criterion::{Criterion, black_box, criterion_group, criterion_main};
use lithos_domain::models::note::{Note, Tag};

fn bench_note_creation(c: &mut Criterion) {
    c.bench_function("note_creation", |b| {
        b.iter(|| {
            let note =
                Note::new(black_box("bench/test.md".to_string())).unwrap();
            black_box(note);
        });
    });
}

fn bench_tag_parsing(c: &mut Criterion) {
    c.bench_function("tag_parsing", |b| {
        b.iter(|| {
            let tag = Tag::parse(black_box("#work/project")).unwrap();
            black_box(tag);
        });
    });
}

criterion_group!(benches, bench_note_creation, bench_tag_parsing);
criterion_main!(benches);
