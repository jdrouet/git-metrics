use crate::assert_success;
use crate::tests::GitRepo;

#[test_case::test_case("git2"; "with git2 backend")]
#[test_case::test_case("command"; "with command backend")]
fn execute(backend: &'static str) {
    super::init_logs();

    let root = tempfile::tempdir().unwrap();
    let server = GitRepo::create(backend, root.path().join("server"));
    let client = GitRepo::clone(&server, root.path().join("client"));
    // set configuration
    let cfg_path = client.path.join(".git-metrics.toml");
    std::fs::write(
        cfg_path,
        r#"[metrics.binary-size]
description = "Binary size"

# max increase of 20%
[[metrics.binary-size.rules]]
type = "max-increase"
ratio = 0.2

[[metrics.binary-size.rules]]
type = "max"
value = 200.0

[metrics.binary-size.subsets.for-darwin]
description = "Binary size for darwin"
matching = { "platform.os" = "darwin" }

[[metrics.binary-size.subsets.for-darwin.rules]]
type = "max"
value = 120.0

[metrics.binary-size.subsets.for-linux]
description = "Binary size for linux"
matching = { "platform.os" = "linux" }

[[metrics.binary-size.subsets.for-linux.rules]]
type = "max"
value = 140.0
"#,
    )
    .unwrap();
    //
    client.commit("First commit");
    client.push();
    //
    client.metrics(["pull"], assert_success!());
    client.metrics(
        ["add", "binary-size", "--tag", "platform.os: linux", "100.0"],
        assert_success!(),
    );
    client.metrics(
        [
            "add",
            "binary-size",
            "--tag",
            "platform.os: darwin",
            "100.0",
        ],
        assert_success!(),
    );
    client.metrics(
        ["add", "binary-size", "--tag", "platform.os: win", "100.0"],
        assert_success!(),
    );
    //
    client.commit("Second commit");
    client.metrics(
        ["add", "binary-size", "--tag", "platform.os: linux", "100.0"],
        assert_success!(),
    );
    client.metrics(
        [
            "add",
            "binary-size",
            "--tag",
            "platform.os: darwin",
            "100.0",
        ],
        assert_success!(),
    );
    client.metrics(
        ["add", "binary-size", "--tag", "platform.os: win", "100.0"],
        assert_success!(),
    );
    client.metrics(["check", "HEAD"], |stdout, stderr, exit| {
        assert_eq!(
            stdout,
            r#"[SUCCESS] binary-size{platform.os="linux"}
[SUCCESS] binary-size{platform.os="darwin"}
[SUCCESS] binary-size{platform.os="win"}
"#
        );
        assert_eq!(stderr, "");
        assert!(exit.is_success());
    });
    //
    client.commit("Third commit");
    client.metrics(
        ["add", "binary-size", "--tag", "platform.os: linux", "100.0"],
        assert_success!(),
    );
    client.metrics(
        [
            "add",
            "binary-size",
            "--tag",
            "platform.os: darwin",
            "150.0",
        ],
        assert_success!(),
    );
    client.metrics(
        ["add", "binary-size", "--tag", "platform.os: win", "130.0"],
        assert_success!(),
    );
    client.metrics(["check", "HEAD"], |stdout, _stderr, exit| {
        assert_eq!(
            stdout,
            "[SUCCESS] binary-size{platform.os=\"linux\"}\n[FAILURE] binary-size{platform.os=\"darwin\"} (2 errors)\n\t- increased of 50.0%, with a limit at 20.0%\n\t- 150 is greater than the max allowed 120\n[FAILURE] binary-size{platform.os=\"win\"} (1 errors)\n\t- increased of 30.0%, with a limit at 20.0%\n"
        );
        assert!(!exit.is_success());
    });
}
