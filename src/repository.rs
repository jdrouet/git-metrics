use crate::metric::Metric;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Note {
    metrics: Vec<Metric>,
}

pub(crate) trait Repository {
    fn get_metrics(&self, target: &str) -> Result<Vec<Metric>, String>;
    fn set_metrics(&self, target: &str, metrics: &[Metric]) -> Result<(), String>;
    fn add_metric(&self, target: &str, metric: Metric) -> Result<(), String> {
        let mut metrics = self.get_metrics(target)?;
        metrics.push(metric);
        self.set_metrics(target, &metrics)
    }
}

const NOTES_REF: Option<&str> = Some("refs/notes/metrics");

pub(crate) struct GitRepository {
    repo: git2::Repository,
}

impl GitRepository {
    pub fn from_env() -> Result<Self, String> {
        let repo = git2::Repository::open_from_env()
            .map_err(|err| format!("unable to open repository: {err:?}"))?;
        Ok(GitRepository { repo })
    }
}

impl Repository for GitRepository {
    fn get_metrics(&self, _target: &str) -> Result<Vec<Metric>, String> {
        // let head = self
        //     .repo
        //     .revparse(target)
        //     .map_err(|err| format!("couldn't get ref for {target:?}: {err:?}"))?;

        let Ok(notes) = self.repo.notes(NOTES_REF) else {
            return Ok(Default::default());
        };

        let mut result = Vec::default();

        for note in notes
            .filter_map(|note| note.ok())
            .filter_map(|(_note_id, annotation_id)| {
                self.repo.find_note(NOTES_REF, annotation_id).ok()
            })
            .filter_map(|note| {
                note.message()
                    .and_then(|msg| toml::from_str::<Note>(msg).ok())
            })
        {
            result.extend(note.metrics);
        }

        Ok(result)
    }

    fn set_metrics(&self, target: &str, metrics: &[Metric]) -> Result<(), String> {
        Ok(())
    }
}
