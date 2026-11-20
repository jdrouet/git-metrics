use std::process::Command;

use crate::assert_failure;
use crate::tests::GitRepo;

#[test_case::test_case(&["log"]; "log")]
#[test_case::test_case(&["show"]; "show")]
#[test_case::test_case(&["add", "my-metric", "1.0"]; "add")]
fn should_explain_missing_commits(cmd: &[&str]) {
    super::init_logs();

    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("repo");
    let output = Command::new("git").arg("init").arg(&path).output().unwrap();
    assert!(output.status.success());

    let repo = GitRepo {
        backend: "git2",
        path_str: path.to_string_lossy().to_string(),
        path,
    };
    repo.metrics(
        cmd.iter().copied(),
        assert_failure!("your current branch does not have any commits yet\n"),
    );
}
