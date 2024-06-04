use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::{NoteRef, RevParse};
use crate::entity::Commit;

#[derive(Clone, Debug, Default)]
pub(crate) struct MockBackend(Rc<MockBackendInner>);

#[derive(Debug, Default)]
pub(crate) struct MockBackendInner {
    commits: Vec<Commit>,
    notes: RefCell<HashMap<String, String>>,
    rev_parses: RefCell<HashMap<String, RevParse>>,
    rev_lists: RefCell<HashMap<String, Vec<String>>>,
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
}

impl super::Backend for MockBackend {
    fn rev_list(&self, range: &str) -> Result<Vec<String>, super::Error> {
        Ok(self
            .0
            .rev_lists
            .borrow()
            .get(range)
            .cloned()
            .unwrap_or_default())
    }

    fn rev_parse(&self, range: &str) -> Result<super::RevParse, super::Error> {
        self.0
            .rev_parses
            .borrow()
            .get(range)
            .cloned()
            .ok_or_else(|| {
                super::Error::new(
                    "invalid range for rev_parse",
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        range.to_string(),
                    )),
                )
            })
    }

    fn list_notes(&self, _note_ref: &NoteRef) -> Result<Vec<super::Note>, super::Error> {
        todo!()
    }

    fn remove_note(&self, target: &str, note_ref: &NoteRef) -> Result<(), super::Error> {
        let key = format!("{target}/{note_ref}");
        self.0.notes.borrow_mut().remove(&key);
        Ok(())
    }

    fn pull(&self, _remote: &str, _local_ref: &NoteRef) -> Result<(), super::Error> {
        todo!()
    }

    fn push(&self, _remote: &str, _local_ref: &NoteRef) -> Result<(), super::Error> {
        todo!()
    }

    fn read_note<T: serde::de::DeserializeOwned>(
        &self,
        target: &str,
        note_ref: &NoteRef,
    ) -> Result<Option<T>, super::Error> {
        let key = format!("{target}/{note_ref}");
        if let Some(value) = self.0.notes.borrow().get(&key) {
            let value: T = toml::from_str(value).map_err(|error| super::Error {
                message: "unable to deserialize",
                source: Box::new(error),
            })?;
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
    ) -> Result<(), super::Error> {
        let key = format!("{target}/{note_ref}");
        let value = toml::to_string_pretty(&value).map_err(|error| super::Error {
            message: "unable to serialize",
            source: Box::new(error),
        })?;
        self.0.notes.borrow_mut().insert(key, value);
        Ok(())
    }

    fn get_commits(&self, _range: &str) -> Result<Vec<crate::entity::Commit>, super::Error> {
        Ok(self.0.commits.clone())
    }
}
