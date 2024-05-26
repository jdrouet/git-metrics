use crate::tests::GitRepo;

#[test_case::test_case("git2"; "with git2 backend")]
#[test_case::test_case("command"; "with command backend")]
fn execute(backend: &'static str) {
    super::init_logs();

    let root = tempfile::tempdir().unwrap();
    let server = GitRepo::create(backend, root.path().join("server"));
    let first = GitRepo::clone(&server, root.path().join("first"));
    first.commit("Hello World");
    first.push();
    //
    let second = GitRepo::clone(&server, root.path().join("second"));
    //
    second.metrics(["pull"], |stdout, stderr, code| {
        assert_eq!(stdout, "");
        assert_eq!(stderr, "");
        assert!(code.is_success());
    });
    //
    first.metrics(["pull"], |stdout, stderr, code| {
        assert_eq!(stdout, "");
        assert_eq!(stderr, "");
        assert!(code.is_success());
    });
    first.metrics(["add", "my-metric", "1.0"], |stdout, stderr, code| {
        assert_eq!(stdout, "");
        assert_eq!(stderr, "");
        assert!(code.is_success());
    });
    first.metrics(["show"], |stdout, stderr, code| {
        assert_eq!(stdout, "my-metric{} = 1.0\n");
        assert_eq!(stderr, "");
        assert!(code.is_success());
    });
    first.metrics(["push"], |stdout, stderr, code| {
        assert_eq!(stdout, "");
        assert_eq!(stderr, "");
        assert!(code.is_success());
    });
    //
    second.metrics(["add", "other-metric", "1.0"], |stdout, stderr, code| {
        assert_eq!(stdout, "");
        assert_eq!(stderr, "");
        assert!(code.is_success());
    });
    second.metrics(["push"], |stdout, stderr, code| {
        assert_eq!(stdout, "");
        assert_ne!(stderr, "");
        assert!(!code.is_success());
    });
    //
    second.metrics(["show"], |stdout, stderr, code| {
        assert_eq!(stdout, "other-metric{} = 1.0\n");
        assert_eq!(stderr, "");
        assert!(code.is_success());
    });
    second.metrics(["pull"], |stdout, stderr, code| {
        assert_eq!(stdout, "");
        assert_eq!(stderr, "");
        assert!(code.is_success());
    });
    second.metrics(["show"], |stdout, stderr, code| {
        assert_eq!(stdout, "my-metric{} = 1.0\nother-metric{} = 1.0\n");
        assert_eq!(stderr, "");
        assert!(code.is_success());
    })
}
