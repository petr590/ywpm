use std::{borrow::Cow, path::Path};

use criterion::{Criterion, criterion_group, criterion_main};
use path_absolutize::Absolutize;

pub fn canonicalize_path_1(cwd: &str, path: String) -> String {
    Path::new(&path)
        .absolutize_from(cwd)
        .to_string_lossy()
        .into_owned()
}

pub fn canonicalize_path_2(cwd: &str, path: String) -> String {
    match Path::new(&path).absolutize_from(cwd) {
        Cow::Borrowed(_) => path,

        Cow::Owned(path_buf) => path_buf
            .to_string_lossy()
            .into_owned(),
    }
}

fn generate_random_paths() -> Vec<String> {
    let chars = ('0'..'9')
            .chain('a'..'z')
            .chain('A'..'Z')
            .collect::<String>();

    std::iter::from_fn(|| {
        Some(std::iter::from_fn(|| {
            if rand::random_range(0..10) < 1 {
                Some('/')
            } else {
                Some(chars.as_bytes()[rand::random_range(0..chars.len())] as char)
            }
        })
        .take(rand::random_range(0..100))
        .collect())
    })
    .take(100)
    .collect()
}

fn bench_canonicalize(criterion: &mut Criterion) {
    let cwd = "/foo/bar";
    let paths = generate_random_paths();

    println!("PATHS: {paths:?}");

    let mut group = criterion.benchmark_group("UnixStream Performance");

    group.sample_size(1000);

    group.bench_function("canonicalize_path_1", |bencher| {
        bencher.iter(|| {
            for path in &paths {
                canonicalize_path_1(cwd, path.clone());
            }
        });
    });

    group.bench_function("canonicalize_path_2", |bencher| {
        bencher.iter(|| {
            for path in &paths {
                canonicalize_path_2(cwd, path.clone());
            }
        });
    });
}

criterion_group!(benches, bench_canonicalize);
criterion_main!(benches);