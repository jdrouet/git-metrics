use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

use super::{NoteRef, RevParse};
use crate::entity::config::Config;
use crate::entity::git::Commit;

#[derive(Debug)]
pub(crate) struct Error {
    message: &'static str,
}

impl Error {
    fn new(message: &'static str) -> Self {
        Self { message }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for Error {}

impl crate::error::DetailedError for Error {
    fn details(&self) -> Option<String> {
        None
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct MockBackend(Rc<MockBackendInner>);

#[derive(Debug)]
pub(crate) struct MockBackendInner {
    temp_dir: tempfile::TempDir,
    commits: Vec<Commit>,
    notes: RefCell<HashMap<String, String>>,
    rev_parses: RefCell<HashMap<String, RevParse>>,
    rev_lists: RefCell<HashMap<String, Vec<String>>>,
}

impl Default for MockBackendInner {
    fn default() -> Self {
        Self {
            temp_dir: tempfile::tempdir().unwrap(),
            commits: Default::default(),
            notes: Default::default(),
            rev_parses: Default::default(),
            rev_lists: Default::default(),
        }
    }
}

impl MockBackend {
    pub(crate) fn get_note(&self, target: &str, note_ref: NoteRef) -> Option<String> {
        let key = format!("{target}/{note_ref}");
        self.0.notes.borrow().get(&key).map(String::from)
    }

    pub(crate) fn set_note(&self, target: &str, note_ref: NoteRef, value: impl Into<String>) {
        let key = format!("{target}/{note_ref}");
        self.0.notes.borrow_mut().insert(key, value.into());
    }

    pub(crate) fn set_rev_list<H: Into<String>>(
        &self,
        target: impl Into<String>,
        items: impl IntoIterator<Item = H>,
    ) {
        self.0.rev_lists.borrow_mut().insert(
            target.into(),
            items.into_iter().map(Into::into).collect::<Vec<String>>(),
        );
    }

    pub(crate) fn set_rev_parse(&self, target: impl Into<String>, item: RevParse) {
        self.0.rev_parses.borrow_mut().insert(target.into(), item);
    }

    pub(crate) fn set_config(&self, input: &str) {
        let file = self.0.temp_dir.path().join(".git-metrics.toml");
        std::fs::write(file, input).unwrap();
    }

    pub(crate) fn get_config(&self) -> Config {
        let file = self.0.temp_dir.path().join(".git-metrics.toml");
        Config::from_path(&file).unwrap()
    }
}

impl super::Backend for MockBackend {
    type Err = Error;

    fn rev_list(&self, range: &str) -> Result<Vec<String>, Self::Err> {
        Ok(self
            .0
            .rev_lists
            .borrow()
            .get(range)
            .cloned()
            .unwrap_or_default())
    }

    fn rev_parse(&self, range: &str) -> Result<super::RevParse, Self::Err> {
        self.0
            .rev_parses
            .borrow()
            .get(range)
            .cloned()
            .ok_or_else(|| Error::new("invalid range for rev_parse"))
    }

    fn list_notes(&self, _note_ref: &NoteRef) -> Result<Vec<super::Note>, Self::Err> {
        todo!()
    }

    fn remove_note(&self, target: &str, note_ref: &NoteRef) -> Result<(), Self::Err> {
        let key = format!("{target}/{note_ref}");
        self.0.notes.borrow_mut().remove(&key);
        Ok(())
    }

    fn pull(&self, _remote: &str, _local_ref: &NoteRef) -> Result<(), Self::Err> {
        todo!()
    }

    fn push(&self, _remote: &str, _local_ref: &NoteRef) -> Result<(), Self::Err> {
        todo!()
    }

    fn read_note<T: serde::de::DeserializeOwned>(
        &self,
        target: &str,
        note_ref: &NoteRef,
    ) -> Result<Option<T>, Self::Err> {
        let key = format!("{target}/{note_ref}");
        if let Some(value) = self.0.notes.borrow().get(&key) {
            let value: T =
                toml::from_str(value).map_err(|_| Error::new("unable to deserialize"))?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn write_note<T: serde::Serialize>(
        &self,
        target: &str,
        note_ref: &NoteRef,
        value: &T,
    ) -> Result<(), Self::Err> {
        let key = format!("{target}/{note_ref}");
        let value =
            toml::to_string_pretty(&value).map_err(|_| Error::new("unable to serialize"))?;
        self.0.notes.borrow_mut().insert(key, value);
        Ok(())
    }

    fn get_commits(&self, _range: &str) -> Result<Vec<crate::entity::git::Commit>, Self::Err> {
        Ok(self.0.commits.clone())
    }

    fn root_path(&self) -> Result<std::path::PathBuf, Self::Err> {
        Ok(self.0.temp_dir.path().to_path_buf())
    }
}
