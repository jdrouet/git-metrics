use crate::metric::Metric;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Note {
    metrics: Vec<Metric>,
}

pub(crate) trait Repository {
    fn get_metrics(&self, target: &str) -> Result<Vec<Metric>, String>;
    fn set_metrics(&self, target: &str, metrics: Vec<Metric>) -> Result<(), String>;
    fn add_metric(&self, target: &str, metric: Metric) -> Result<(), String> {
        let mut metrics = self.get_metrics(target)?;
        metrics.push(metric);
        self.set_metrics(target, metrics)
    }
}

const NOTES_REF: Option<&str> = Some("refs/notes/metrics");

pub(crate) struct GitRepository {
    repo: git2::Repository,
}

impl GitRepository {
    pub(crate) fn from_env() -> Result<Self, String> {
        let repo = git2::Repository::open_from_env()
            .map_err(|err| format!("unable to open repository: {err:?}"))?;
        Ok(GitRepository { repo })
    }

    fn revision_id(&self, target: &str) -> Result<git2::Oid, String> {
        self.repo
            .revparse_single(target)
            .map(|rev| rev.id())
            .map_err(|err| format!("couldn't get ref for {target:?}: {err:?}"))
    }

    fn signature(&self) -> Result<git2::Signature, String> {
        self.repo
            .signature()
            .map_err(|err| format!("unable to read signature: {err:?}"))
    }
}

impl Repository for GitRepository {
    fn get_metrics(&self, target: &str) -> Result<Vec<Metric>, String> {
        let rev_id = self.revision_id(target)?;

        let Ok(note) = self.repo.find_note(NOTES_REF, rev_id) else {
            return Ok(Default::default());
        };

        match note.message().map(|msg| toml::from_str::<Note>(msg)) {
            Some(Ok(res)) => Ok(res.metrics),
            Some(Err(err)) => Err(format!("unable to decode content: {err:?}")),
            None => Ok(Default::default()),
        }
    }

    fn set_metrics(&self, target: &str, metrics: Vec<Metric>) -> Result<(), String> {
        let head_id = self.revision_id(target)?;
        let sig = self.signature()?;

        let note = toml::to_string_pretty(&Note { metrics })
            .map_err(|err| format!("unable to serialize metrics: {err:?}"))?;
        self.repo
            .note(&sig, &sig, NOTES_REF, head_id, &note, true)
            .map_err(|err| format!("unable to persist metrics: {err:?}"))?;

        Ok(())
    }
}
