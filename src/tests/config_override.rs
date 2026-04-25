use crate::tests::GitRepo;

fn setup(backend: &'static str, root: &std::path::Path) -> GitRepo {
    let server = GitRepo::create(backend, root.join("server"));
    let client = GitRepo::clone(&server, root.join("client"));
    client.commit("first commit");
    client.push();
    client
        .metrics_exec(backend, ["pull"])
        .expect("initial pull");
    client
        .metrics_exec(backend, ["add", "binary-size", "100.0"])
        .expect("add on first commit");
    client.commit("second commit");
    client
        .metrics_exec(backend, ["add", "binary-size", "100.0"])
        .expect("add on second commit");
    client
}

#[test_case::test_case("git2"; "with git2 backend")]
#[test_case::test_case("command"; "with command backend")]
fn override_overrides_repo_config(backend: &'static str) {
    super::init_logs();

    let root = tempfile::tempdir().unwrap();

    let client = setup(backend, root.path());

    let repo_cfg = client.path.join(".git-metrics.toml");
    std::fs::write(
        &repo_cfg,
        r#"[metrics.binary-size]
[[metrics.binary-size.rules]]
type = "max"
value = 1000.0
"#,
    )
    .unwrap();

    let alt_cfg = root.path().join("strict.toml");
    std::fs::write(
        &alt_cfg,
        r#"[metrics.binary-size]
[[metrics.binary-size.rules]]
type = "max"
value = 10.0
"#,
    )
    .unwrap();

    client
        .metrics_exec(backend, ["check", "HEAD"])
        .expect("repo config allows up to 1000");

    let alt_cfg_str = alt_cfg.to_string_lossy().to_string();
    client
        .metrics_exec(backend, ["--config", alt_cfg_str.as_str(), "check", "HEAD"])
        .expect_err("strict override caps at 10");
}

#[test_case::test_case("git2"; "with git2 backend")]
#[test_case::test_case("command"; "with command backend")]
fn override_applies_when_repo_has_no_config(backend: &'static str) {
    super::init_logs();

    let root = tempfile::tempdir().unwrap();
    let client = setup(backend, root.path());

    let alt_cfg = root.path().join("alt.toml");
    std::fs::write(
        &alt_cfg,
        r#"[metrics.binary-size]
[[metrics.binary-size.rules]]
type = "max"
value = 10.0
"#,
    )
    .unwrap();

    client
        .metrics_exec(backend, ["check", "HEAD"])
        .expect("no repo config, no rules to break");

    let alt_cfg_str = alt_cfg.to_string_lossy().to_string();
    client
        .metrics_exec(backend, ["--config", alt_cfg_str.as_str(), "check", "HEAD"])
        .expect_err("override should make the rule fail");
}

#[test_case::test_case("git2"; "with git2 backend")]
#[test_case::test_case("command"; "with command backend")]
fn missing_override_path_errors_clearly(backend: &'static str) {
    super::init_logs();

    let root = tempfile::tempdir().unwrap();
    let client = setup(backend, root.path());

    let missing = root.path().join("does-not-exist.toml");
    let missing_str = missing.to_string_lossy().to_string();

    let err = client
        .metrics_exec(backend, ["--config", missing_str.as_str(), "check", "HEAD"])
        .expect_err("missing override path should fail");
    assert!(
        err.to_lowercase().contains("no such file")
            || err.to_lowercase().contains("not found")
            || err.to_lowercase().contains("cannot find"),
        "expected a missing-file error, got: {err}"
    );
}

#[test_case::test_case("git2"; "with git2 backend")]
#[test_case::test_case("command"; "with command backend")]
fn override_ignored_by_commands_that_dont_read_config(backend: &'static str) {
    super::init_logs();

    let root = tempfile::tempdir().unwrap();
    let server = GitRepo::create(backend, root.path().join("server"));
    let client = GitRepo::clone(&server, root.path().join("client"));
    client.commit("first commit");
    client.push();
    client.metrics_exec(backend, ["pull"]).unwrap();

    let missing = root.path().join("does-not-exist.toml");
    let missing_str = missing.to_string_lossy().to_string();

    client
        .metrics_exec(
            backend,
            ["--config", missing_str.as_str(), "add", "anything", "1.0"],
        )
        .expect("add should not read the config file");
}

#[test_case::test_case("git2"; "with git2 backend")]
#[test_case::test_case("command"; "with command backend")]
fn init_writes_to_override_path(backend: &'static str) {
    super::init_logs();

    let root = tempfile::tempdir().unwrap();
    let server = GitRepo::create(backend, root.path().join("server"));
    let client = GitRepo::clone(&server, root.path().join("client"));
    client.commit("first commit");
    client.push();

    let target = root.path().join("custom.toml");
    let target_str = target.to_string_lossy().to_string();

    client
        .metrics_exec(backend, ["--config", target_str.as_str(), "init"])
        .unwrap();

    assert!(target.exists(), "init should have created {target_str}");
    assert!(
        !client.path.join(".git-metrics.toml").exists(),
        "init with --config should not touch the default location"
    );
}
