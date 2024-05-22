use crate::tests::GitRepo;

#[test_case::test_case("git2"; "with git2 backend")]
#[test_case::test_case("command"; "with command backend")]
fn execute(backend: &str) {
    super::init_logs();

    let root = tempfile::tempdir().unwrap();
    let server = GitRepo::create(root.path().join("server"));
    let first = GitRepo::clone(&server, root.path().join("first"));
    first.commit("Hello World");
    first.push();
    //
    let (stdout, stderr) = first.execute([
        "git-metrics",
        "--root-dir",
        first.path_str(),
        "--backend",
        backend,
        "-vvvvv",
        "add",
        "my-metric",
        "1.0",
    ]);
    assert!(stdout.is_empty());
    assert!(stderr.is_empty());
    //
    let (stdout, stderr) = first.execute([
        "git-metrics",
        "--root-dir",
        first.path_str(),
        "--backend",
        backend,
        "show",
    ]);
    assert_eq!(
        stdout,
        "Metric { name: \"my-metric\", tags: {}, value: 1.0 }\n"
    );
    assert_eq!(stderr, "");
    assert!(stderr.is_empty());
    //
    let (stdout, stderr) = first.execute([
        "git-metrics",
        "--root-dir",
        first.path_str(),
        "--backend",
        backend,
        "push",
    ]);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "");
    //
    let second = GitRepo::clone(&server, root.path().join("second"));
    let (stdout, stderr) = second.execute([
        "git-metrics",
        "--root-dir",
        second.path_str(),
        "--backend",
        backend,
        "pull",
    ]);
    assert_eq!(stdout, "");
    assert_eq!(stderr, "");
    //
    let (stdout, stderr) = second.execute([
        "git-metrics",
        "--root-dir",
        second.path_str(),
        "--backend",
        backend,
        "show",
    ]);
    assert_eq!(
        stdout,
        "Metric { name: \"my-metric\", tags: {}, value: 1.0 }\n"
    );
    assert_eq!(stderr, "");
    assert!(stderr.is_empty());
}
