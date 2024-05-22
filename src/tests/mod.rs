use std::{ffi::OsString, path::PathBuf, process::Command};

mod simple_use_case;

fn init_logs() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .try_init();
}

struct GitRepo {
    path: PathBuf,
    path_str: String,
}

impl GitRepo {
    fn create(path: PathBuf) -> Self {
        let path_str = path.to_string_lossy().to_string();
        let output = Command::new("git")
            .arg("init")
            .arg("--bare")
            .arg(path.as_path())
            .output()
            .unwrap();
        assert!(output.status.success());
        Self { path, path_str }
    }

    fn clone(server: &GitRepo, path: PathBuf) -> Self {
        let path_str = path.to_string_lossy().to_string();
        let output = Command::new("git")
            .arg("clone")
            .arg(server.path.as_path())
            .arg(path.as_path())
            .output()
            .unwrap();
        assert!(output.status.success());
        Self { path, path_str }
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
        assert!(output.status.success());
    }

    fn push(&self) {
        let output = Command::new("git")
            .current_dir(self.path.as_path())
            .arg("push")
            .output()
            .unwrap();
        assert!(String::from_utf8_lossy(&output.stderr).contains("main -> main"));
        assert!(output.status.success());
    }

    fn execute<I, T>(&self, itr: I) -> (String, String)
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        use clap::Parser;

        let mut stdout = Vec::<u8>::new();
        let mut stderr = Vec::<u8>::new();
        crate::Args::parse_from(itr)
            .execute(&mut stdout, &mut stderr)
            .unwrap();

        tracing::info!("output bytes: {}/{}", stdout.len(), stderr.len());

        (
            String::from_utf8_lossy(&stdout).to_string(),
            String::from_utf8_lossy(&stderr).to_string(),
        )
    }
}
