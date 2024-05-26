use std::{path::PathBuf, process::Command};

mod conflict_different;
mod simple_use_case;

fn init_logs() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .try_init();
}

struct GitRepo {
    backend: &'static str,
    path: PathBuf,
    path_str: String,
}

impl GitRepo {
    fn create(backend: &'static str, path: PathBuf) -> Self {
        let path_str = path.to_string_lossy().to_string();
        let output = Command::new("git")
            .arg("init")
            .arg("--bare")
            .arg(path.as_path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "stderr: {:?}",
            String::from_utf8_lossy(&output.stderr)
        );
        Self {
            backend,
            path,
            path_str,
        }
    }

    fn clone(server: &GitRepo, path: PathBuf) -> Self {
        let path_str = path.to_string_lossy().to_string();
        let output = Command::new("git")
            .arg("clone")
            .arg(server.path.as_path())
            .arg(path.as_path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "stderr: {:?}",
            String::from_utf8_lossy(&output.stderr)
        );
        Self {
            backend: server.backend,
            path,
            path_str,
        }
    }

    fn path_str(&self) -> &str {
        self.path_str.as_str()
    }

    fn commit(&self, message: &str) {
        let output = Command::new("git")
            .current_dir(self.path.as_path())
            .arg("commit")
            .arg("--allow-empty")
            .arg("-m")
            .arg(message)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "stderr: {:?}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn pull(&self) {
        let output = Command::new("git")
            .current_dir(self.path.as_path())
            .arg("pull")
            .output()
            .unwrap();
        assert!(output.status.success());
    }

    fn push(&self) {
        let output = Command::new("git")
            .current_dir(self.path.as_path())
            .arg("push")
            .output()
            .unwrap();
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("main -> main"),
            "stderr: {:?}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            output.status.success(),
            "stderr: {:?}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn metrics<'a, I, F>(&'a self, iter: I, callback: F)
    where
        I: IntoIterator<Item = &'a str>,
        F: FnOnce(String, String, crate::ExitCode),
    {
        use clap::Parser;

        let mut args = vec![
            "git-metrics",
            "--root-dir",
            self.path_str(),
            "--backend",
            self.backend,
        ];
        args.extend(iter);

        let mut stdout = Vec::<u8>::new();
        let mut stderr = Vec::<u8>::new();
        let result = crate::Args::parse_from(args).execute(&mut stdout, &mut stderr);

        let stdout = String::from_utf8_lossy(&stdout).to_string();
        let stderr = String::from_utf8_lossy(&stderr).to_string();

        callback(stdout, stderr, result);
    }
}

#[macro_export]
macro_rules! assert_success {
    () => {
        |stdout, stderr, code| {
            assert_eq!(stdout, "", "unexpected stdout");
            assert_eq!(stderr, "", "unexpected stderr");
            assert!(code.is_success());
        }
    };
    ($output:expr) => {
        |stdout, stderr, code| {
            assert_eq!(stdout, $output, "unexpected stdout");
            assert_eq!(stderr, "", "unexpected stderr");
            assert!(code.is_success());
        }
    };
}
#[macro_export]
macro_rules! assert_failure {
    () => {
        |stdout, stderr, code| {
            assert_eq!(stdout, "", "unexpected stdout");
            assert_eq!(stderr, "", "unexpected stderr");
            assert!(!code.is_success());
        }
    };
    ($output:expr) => {
        |stdout, stderr, code| {
            assert_eq!(stdout, "", "unexpected stdout");
            assert_eq!(stderr, $output, "unexpected stderr");
            assert!(!code.is_success());
        }
    };
}
