use crate::assert_success;
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
    first.metrics(["add", "my-metric", "1.0"], assert_success!());
    //
    first.metrics(["show"], assert_success!("my-metric 1.00\n"));
    //
    first.metrics(["push"], assert_success!());
    //
    let second = GitRepo::clone(&server, root.path().join("second"));
    second.metrics(["pull"], assert_success!());
    //
    second.metrics(["show"], assert_success!("my-metric 1.00\n"));
    //
    first.commit("second commit");
    first.push();
    first.metrics(["add", "my-metric", "2.0"], assert_success!());
    first.metrics(["add", "other-metric", "42.0"], assert_success!());
    first.metrics(["push"], assert_success!());
    //
    second.pull();
    second.metrics(["pull"], assert_success!());
    second.metrics(["log"], |stdout, stderr, code| {
        let lines: Vec<_> = stdout.trim().split('\n').collect();
        assert_eq!(lines.len(), 5);
        assert!(!lines[0].starts_with(' '));
        similar_asserts::assert_eq!(lines[1], "    my-metric 2.00");
        similar_asserts::assert_eq!(lines[2], "    other-metric 42.00");
        assert!(!lines[3].starts_with(' '));
        similar_asserts::assert_eq!(lines[4], "    my-metric 1.00");
        assert_eq!(stderr, "");
        assert!(code.is_success());
    });
}
