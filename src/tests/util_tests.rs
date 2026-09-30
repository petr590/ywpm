use crate::util::canonicalize_path;

#[test]
fn test_canonicalize_paths() {
    assert_eq!(canonicalize_path("/a/b/c", "d"),   "/a/b/c/d");
    assert_eq!(canonicalize_path("/a/b/c", "./d"), "/a/b/c/d");
    assert_eq!(canonicalize_path("/a/b/c", "/d"),  "/d");

    assert_eq!(canonicalize_path("/a/b/c", "d/../e"),   "/a/b/c/e");
    assert_eq!(canonicalize_path("/a/b/c", "./d/../e"), "/a/b/c/e");
    assert_eq!(canonicalize_path("/a/b/c", "/d/../e"),  "/e");
}