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
    let second = GitRepo::clone(&server, root.path().join("second"));
    //
    second.metrics(["pull"], assert_success!());
    //
    first.metrics(["pull"], assert_success!());
    first.metrics(["add", "my-metric", "1.0"], assert_success!());
    first.metrics(["show"], assert_success!("my-metric 1.00\n"));
    first.metrics(["push"], assert_success!());
    //
    second.metrics(["add", "other-metric", "1.0"], assert_success!());
    second.metrics(["push"], |stdout, stderr, code| {
        assert_eq!(stdout, "", "unexpected stdout");
        assert!(stderr.starts_with("unable to push metrics"), "{stderr}");
        assert!(!code.is_success());
    });
    //
    second.metrics(["show"], assert_success!("other-metric 1.00\n"));
    second.metrics(["pull"], assert_success!());
    second.metrics(
        ["show"],
        assert_success!("my-metric 1.00\nother-metric 1.00\n"),
    );
}
