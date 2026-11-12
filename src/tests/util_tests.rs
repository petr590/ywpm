use std::borrow::Cow;
use std::env;
use std::path::{Path, PathBuf};

use path_absolutize::Absolutize;

use crate::util;

#[test]
fn test_absolutize_from() {
    assert_eq!(Path::new("d")  .absolutize_from("/a/b/c").into_owned(), PathBuf::from("/a/b/c/d"));
    assert_eq!(Path::new("./d").absolutize_from("/a/b/c").into_owned(), PathBuf::from("/a/b/c/d"));
    assert_eq!(Path::new("/d") .absolutize_from("/a/b/c").into_owned(), PathBuf::from("/d"));

    assert_eq!(Path::new("d/../e")  .absolutize_from("/a/b/c").into_owned(), PathBuf::from("/a/b/c/e"));
    assert_eq!(Path::new("./d/../e").absolutize_from("/a/b/c").into_owned(), PathBuf::from("/a/b/c/e"));
    assert_eq!(Path::new("/d/../e") .absolutize_from("/a/b/c").into_owned(), PathBuf::from("/e"));
}

#[test]
fn test_add_home_if_neccecery() {
    assert_eq!(
        util::replace_tilde_with_home(Path::new("~/a/b/c")),
        Cow::Owned::<'_, Path>(env::home_dir().unwrap().join("a/b/c"))
    );

    assert_eq!(
        util::replace_tilde_with_home(Path::new("/a/b/c")),
        Cow::Borrowed(Path::new("/a/b/c"))
    );

    assert_eq!(
        util::replace_tilde_with_home(Path::new("./a/b/c")),
        Cow::Borrowed(Path::new("./a/b/c"))
    );
}