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
    first.metrics(["add", "my-metric", "1.0"], super::no_output);
    //
    first.metrics(["show"], |stdout, stderr, code| {
        assert!(code.is_success());
        assert_eq!(
            stdout,
            "Metric { name: \"my-metric\", tags: {}, value: 1.0 }\n"
        );
        assert_eq!(stderr, "");
        assert!(stderr.is_empty());
    });
    //
    first.metrics(["push"], super::no_output);
    //
    let second = GitRepo::clone(&server, root.path().join("second"));
    second.metrics(["pull"], super::no_output);
    //
    second.metrics(["show"], |stdout, stderr, code| {
        assert!(code.is_success());
        assert_eq!(
            stdout,
            "Metric { name: \"my-metric\", tags: {}, value: 1.0 }\n"
        );
        assert_eq!(stderr, "");
        assert!(stderr.is_empty());
    });
}
