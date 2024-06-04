use crate::assert_success;
use crate::tests::GitRepo;

#[test]
fn execute() {
    super::init_logs();

    let root = tempfile::tempdir().unwrap();
    let server = GitRepo::create("git2", root.path().join("server"));
    let cli = GitRepo::clone(&server, root.path().join("first"));
    // HEAD~4
    cli.commit("0001");
    cli.push();
    cli.metrics(
        ["add", "binary_size", "--tag", "build.os: linux", "1000"],
        assert_success!(),
    );
    cli.metrics(
        ["add", "binary_size", "--tag", "build.os: windows", "2000"],
        assert_success!(),
    );
    cli.metrics(["push"], assert_success!());
    // HEAD~3
    cli.commit("0002");
    cli.push();
    cli.metrics(
        ["add", "binary_size", "--tag", "build.os: linux", "1000"],
        assert_success!(),
    );
    cli.metrics(
        ["add", "binary_size", "--tag", "build.os: windows", "2000"],
        assert_success!(),
    );
    cli.metrics(
        ["add", "binary_size", "--tag", "build.os: macos", "3000"],
        assert_success!(),
    );
    cli.metrics(["push"], assert_success!());
    // HEAD~2
    cli.commit("0003");
    cli.push();
    cli.metrics(
        ["add", "binary_size", "--tag", "build.os: linux", "1500"],
        assert_success!(),
    );
    cli.metrics(
        ["add", "binary_size", "--tag", "build.os: windows", "2500"],
        assert_success!(),
    );
    cli.metrics(
        ["add", "binary_size", "--tag", "build.os: macos", "3500"],
        assert_success!(),
    );
    // HEAD~1
    cli.commit("0004");
    cli.push();
    cli.metrics(
        ["add", "binary_size", "--tag", "build.os: linux", "1500"],
        assert_success!(),
    );
    cli.metrics(
        ["add", "binary_size", "--tag", "build.os: windows", "2500"],
        assert_success!(),
    );
    cli.metrics(
        ["add", "binary_size", "--tag", "build.os: macos", "4000"],
        assert_success!(),
    );
    // HEAD
    cli.commit("0005");
    cli.push();
    cli.metrics(
        ["add", "binary_size", "--tag", "build.os: linux", "1000"],
        assert_success!(),
    );
    cli.metrics(
        ["add", "binary_size", "--tag", "build.os: windows", "2000"],
        assert_success!(),
    );
    cli.metrics(
        ["add", "binary_size", "--tag", "build.os: macos", "3000"],
        assert_success!(),
    );
    //
    assert_eq!(
        cli.metrics_exec("command", ["diff"]),
        cli.metrics_exec("git2", ["diff"]),
    );
    assert_eq!(
        cli.metrics_exec("command", ["diff", "HEAD~2"]),
        cli.metrics_exec("git2", ["diff", "HEAD~2"]),
    );
    assert_eq!(
        cli.metrics_exec("command", ["diff", "HEAD~3..HEAD~1"]),
        cli.metrics_exec("git2", ["diff", "HEAD~3..HEAD~1"]),
    );
    //
    cli.metrics(["diff"], |stdout, stderr, code| {
        assert_eq!(
            stdout,
            r#"- binary_size{build.os="linux"} 1500.0
+ binary_size{build.os="linux"} 1000.0 (-33.33 %)
- binary_size{build.os="windows"} 2500.0
+ binary_size{build.os="windows"} 2000.0 (-20.00 %)
- binary_size{build.os="macos"} 4000.0
+ binary_size{build.os="macos"} 3000.0 (-25.00 %)
"#
        );
        assert_eq!(stderr, "");
        assert!(code.is_success());
    });
    cli.metrics(["diff", "HEAD~3..HEAD~1"], |stdout, stderr, code| {
        assert_eq!(
            stdout,
            r#"- binary_size{build.os="linux"} 1000.0
+ binary_size{build.os="linux"} 1500.0 (+50.00 %)
- binary_size{build.os="windows"} 2000.0
+ binary_size{build.os="windows"} 2500.0 (+25.00 %)
- binary_size{build.os="macos"} 3000.0
+ binary_size{build.os="macos"} 4000.0 (+33.33 %)
"#
        );
        assert_eq!(stderr, "");
        assert!(code.is_success());
    });
}
