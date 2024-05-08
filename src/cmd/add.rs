/// Add a metric related to the target
#[derive(clap::Parser, Debug, Default)]
pub(crate) struct CommandAdd {
    /// Name of the metric
    name: String,
    /// Context given to the metric
    #[clap(short, long)]
    context: Vec<String>,
    /// Value of the metric
    value: f64,
}

impl super::Executor for CommandAdd {
    fn execute<Repo: crate::repository::Repository, Out: std::io::Write, Err: std::io::Write>(
        self,
        repo: Repo,
        _stdout: Out,
        mut stderr: Err,
    ) -> std::io::Result<()> {
        let metric = crate::metric::Metric {
            name: self.name,
            context: self
                .context
                .into_iter()
                .filter_map(|item| {
                    item.split_once(':')
                        .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
                })
                .filter(|(key, _)| !key.is_empty())
                .collect(),
            value: self.value,
        };
        if let Err(err) = repo.add_metric("HEAD", metric) {
            stderr.write_all(err.as_bytes())?;
        }
        Ok(())
    }
}
