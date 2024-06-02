use crate::{assert_failure, assert_success, tests::GitRepo};

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
    first.metrics(["show"], assert_success!("my-metric 1.0\n"));
    first.metrics(["push"], assert_success!());
    //
    second.metrics(["add", "other-metric", "1.0"], assert_success!());
    second.metrics(["push"], assert_failure!("unable to push metrics\n"));
    //
    second.metrics(["show"], assert_success!("other-metric 1.0\n"));
    second.metrics(["pull"], assert_success!());
    second.metrics(
        ["show"],
        assert_success!("my-metric 1.0\nother-metric 1.0\n"),
    );
}
