use crate::util;

macro_rules! canonicalize_path {
    ($cwd:expr, $path:expr) => {
        util::canonicalize_path($cwd, $path)
    };
}

#[test]
fn test_canonicalize_paths() {
    assert_eq!(canonicalize_path!("/a/b/c", "d"),   "/a/b/c/d");
    assert_eq!(canonicalize_path!("/a/b/c", "./d"), "/a/b/c/d");
    assert_eq!(canonicalize_path!("/a/b/c", "/d"),  "/d");

    assert_eq!(canonicalize_path!("/a/b/c", "d/../e"),   "/a/b/c/e");
    assert_eq!(canonicalize_path!("/a/b/c", "./d/../e"), "/a/b/c/e");
    assert_eq!(canonicalize_path!("/a/b/c", "/d/../e"),  "/e");
}