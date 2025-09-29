use loro::{ContainerTrait, LoroText as LoroTextInner, PeerID};
use pyo3::{
    exceptions::{PyIndexError, PyTypeError, PyValueError},
    prelude::*,
    types::{PyBytes, PySlice, PyString},
    Bound, PyErr, PyRef,
};
use std::{fmt::Display, sync::Arc};

use crate::{
    doc::LoroDoc,
    err::{PyLoroError, PyLoroResult},
    event::{DiffEvent, Subscription, TextDelta},
    value::{ContainerID, LoroValue, ID},
};

pub fn register_class(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<LoroText>()?;
    m.add_class::<Cursor>()?;
    m.add_class::<Side>()?;
    Ok(())
}

#[pyclass(frozen)]
#[derive(Debug, Clone, Default)]
pub struct LoroText(pub LoroTextInner);

#[pymethods]
impl LoroText {
    /// Create a new container that is detached from the document.
    ///
    /// The edits on a detached container will not be persisted.
    /// To attach the container to the document, please insert it into an attached container.
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the container is attached to a document
    ///
    /// The edits on a detached container will not be persisted.
    /// To attach the container to the document, please insert it into an attached container.
    #[getter]
    pub fn is_attached(&self) -> bool {
        self.0.is_attached()
    }

    pub fn __str__(&self) -> String {
        self.0.to_string()
    }

    pub fn __repr__(&self) -> String {
        format!("LoroText({:?})", self.0.to_string())
    }

    /// Get the [ContainerID]  of the text container.
    #[getter]
    pub fn id(&self) -> ContainerID {
        self.0.id().into()
    }

    // /// Iterate each span(internal storage unit) of the text.
    // ///
    // /// The callback function will be called for each character in the text.
    // /// If the callback returns `false`, the iteration will stop.
    // ///
    // /// Limitation: you cannot access or alter the doc state when iterating.
    // /// If you need to access or alter the doc state, please use `to_string` instead.
    // pub fn iter(&self, callback: impl FnMut(&str) -> bool) {
    //     self.0.iter(callback);
    // }

    /// Insert a string at the given unicode position.
    pub fn insert(&self, pos: usize, s: &str) -> PyLoroResult<()> {
        self.0.insert(pos, s)?;
        Ok(())
    }

    /// Insert a string at the given utf-8 position.
    pub fn insert_utf8(&self, pos: usize, s: &str) -> PyLoroResult<()> {
        self.0.insert_utf8(pos, s)?;
        Ok(())
    }

    /// Delete a range of text at the given unicode position with unicode length.
    pub fn delete(&self, pos: usize, len: usize) -> PyLoroResult<()> {
        self.0.delete(pos, len)?;
        Ok(())
    }

    /// Delete a range of text at the given utf-8 position with utf-8 length.
    pub fn delete_utf8(&self, pos: usize, len: usize) -> PyLoroResult<()> {
        self.0.delete_utf8(pos, len)?;
        Ok(())
    }

    /// Get a string slice at the given Unicode range
    pub fn slice(&self, start_index: usize, end_index: usize) -> PyLoroResult<String> {
        let s = self.0.slice(start_index, end_index)?;
        Ok(s)
    }

    /// Get the characters at given unicode position.
    pub fn char_at(&self, pos: usize) -> PyLoroResult<char> {
        let c = self.0.char_at(pos)?;
        Ok(c)
    }

    /// Delete specified character and insert string at the same position at given unicode position.
    pub fn splice(&self, pos: usize, len: usize, s: &str) -> PyLoroResult<String> {
        let s = self.0.splice(pos, len, s)?;
        Ok(s)
    }

    /// Whether the text container is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn __len__(&self) -> usize {
        self.len_unicode()
    }

    pub fn __getitem__<'py>(
        &self,
        py: Python<'py>,
        key: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if let Ok(index) = key.extract::<isize>() {
            let len = self.len_unicode() as isize;
            let mut idx = index;
            if idx < 0 {
                idx += len;
            }
            if idx < 0 || idx >= len {
                return Err(PyIndexError::new_err("string index out of range"));
            }
            let ch = self.char_at(idx as usize).map_err(PyErr::from)?;
            let mut buf = [0u8; 4];
            let as_str = ch.encode_utf8(&mut buf);
            Ok(PyString::new(py, as_str).into_any())
        } else if let Ok(slice) = key.downcast::<PySlice>() {
            let text = self.0.to_string();
            let py_str = PyString::new(py, &text);
            py_str.get_item(slice)
        } else {
            Err(PyTypeError::new_err(
                "text indices must be integers or slices",
            ))
        }
    }

    pub fn __contains__(&self, item: Bound<'_, PyAny>) -> PyResult<bool> {
        let text = self.0.to_string();
        if let Ok(substr) = item.extract::<&str>() {
            Ok(text.contains(substr))
        } else if let Ok(other) = item.extract::<PyRef<LoroText>>() {
            Ok(text.contains(&other.0.to_string()))
        } else {
            Ok(false)
        }
    }

    pub fn __add__(&self, other: Bound<'_, PyAny>) -> PyResult<String> {
        let mut result = self.0.to_string();
        if let Ok(substr) = other.extract::<&str>() {
            result.push_str(substr);
            Ok(result)
        } else if let Ok(other_text) = other.extract::<PyRef<LoroText>>() {
            result.push_str(&other_text.0.to_string());
            Ok(result)
        } else {
            Err(PyTypeError::new_err("can only concatenate str or LoroText"))
        }
    }

    pub fn __radd__(&self, other: Bound<'_, PyAny>) -> PyResult<String> {
        if let Ok(prefix) = other.extract::<&str>() {
            Ok(format!("{}{}", prefix, self.0.to_string()))
        } else if let Ok(other_text) = other.extract::<PyRef<LoroText>>() {
            Ok(format!(
                "{}{}",
                other_text.0.to_string(),
                self.0.to_string()
            ))
        } else {
            Err(PyTypeError::new_err("can only concatenate str or LoroText"))
        }
    }

    /// Get the length of the text container in UTF-8.
    #[getter]
    pub fn len_utf8(&self) -> usize {
        self.0.len_utf8()
    }

    /// Get the length of the text container in Unicode.
    #[getter]
    pub fn len_unicode(&self) -> usize {
        self.0.len_unicode()
    }

    /// Get the length of the text container in UTF-16.
    #[getter]
    pub fn len_utf16(&self) -> usize {
        self.0.len_utf16()
    }

    /// Update the current text based on the provided text.
    ///
    /// It will calculate the minimal difference and apply it to the current text.
    /// It uses Myers' diff algorithm to compute the optimal difference.
    ///
    /// This could take a long time for large texts (e.g. > 50_000 characters).
    /// In that case, you should use `updateByLine` instead.
    ///
    /// # Example
    /// ```rust
    /// use loro::LoroDoc;
    ///
    /// let doc = LoroDoc::new();
    /// let text = doc.get_text("text");
    /// text.insert(0, "Hello").unwrap();
    /// text.update("Hello World", Default::default()).unwrap();
    /// assert_eq!(text.to_string(), "Hello World");
    /// ```
    #[pyo3(signature = (text, use_refined_diff=true, timeout_ms=None))]
    pub fn update(
        &self,
        text: &str,
        use_refined_diff: bool,
        timeout_ms: Option<f64>,
    ) -> PyResult<()> {
        self.0
            .update(
                text,
                loro::UpdateOptions {
                    timeout_ms,
                    use_refined_diff,
                },
            )
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(())
    }

    /// Update the current text based on the provided text.
    ///
    /// This update calculation is line-based, which will be more efficient but less precise.
    #[pyo3(signature = (text, use_refined_diff=true, timeout_ms=None))]
    pub fn update_by_line(
        &self,
        text: &str,
        use_refined_diff: bool,
        timeout_ms: Option<f64>,
    ) -> PyResult<()> {
        self.0
            .update_by_line(
                text,
                loro::UpdateOptions {
                    timeout_ms,
                    use_refined_diff,
                },
            )
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(())
    }

    /// Apply a [delta](https://quilljs.com/docs/delta/) to the text container.
    pub fn apply_delta(&self, delta: Vec<TextDelta>) -> PyLoroResult<()> {
        self.0
            .apply_delta(&delta.iter().map(|x| x.into()).collect::<Vec<_>>())?;
        Ok(())
    }

    /// Mark a range of text with a key-value pair.
    ///
    /// You can use it to create a highlight, make a range of text bold, or add a link to a range of text.
    ///
    /// You can specify the `expand` option to set the behavior when inserting text at the boundary of the range.
    ///
    /// - `after`(default): when inserting text right after the given range, the mark will be expanded to include the inserted text
    /// - `before`: when inserting text right before the given range, the mark will be expanded to include the inserted text
    /// - `none`: the mark will not be expanded to include the inserted text at the boundaries
    /// - `both`: when inserting text either right before or right after the given range, the mark will be expanded to include the inserted text
    ///
    /// *You should make sure that a key is always associated with the same expand type.*
    ///
    /// Note: this is not suitable for unmergeable annotations like comments.
    pub fn mark(&self, start: usize, end: usize, key: &str, value: LoroValue) -> PyLoroResult<()> {
        self.0.mark(start..end, key, value)?;
        Ok(())
    }

    /// Unmark a range of text with a key and a value.
    ///
    /// You can use it to remove highlights, bolds or links
    ///
    /// You can specify the `expand` option to set the behavior when inserting text at the boundary of the range.
    ///
    /// **Note: You should specify the same expand type as when you mark the text.**
    ///
    /// - `after`(default): when inserting text right after the given range, the mark will be expanded to include the inserted text
    /// - `before`: when inserting text right before the given range, the mark will be expanded to include the inserted text
    /// - `none`: the mark will not be expanded to include the inserted text at the boundaries
    /// - `both`: when inserting text either right before or right after the given range, the mark will be expanded to include the inserted text
    ///
    /// *You should make sure that a key is always associated with the same expand type.*
    ///
    /// Note: you cannot delete unmergeable annotations like comments by this method.
    pub fn unmark(&self, start: usize, end: usize, key: &str) -> PyLoroResult<()> {
        self.0.unmark(start..end, key)?;
        Ok(())
    }

    /// Get the rich text value of the text container.
    ///
    /// # Example
    /// ```
    /// # use loro::{LoroDoc, ToJson, ExpandType};
    /// # use serde_json::json;
    ///
    /// let doc = LoroDoc::new();
    /// let text = doc.get_text("text");
    /// text.insert(0, "Hello world!").unwrap();
    /// text.mark(0..5, "bold", true).unwrap();
    /// assert_eq!(
    ///     text.get_richtext_value().to_json_value(),
    ///     json!([
    ///         { "insert": "Hello", "attributes": {"bold": true} },
    ///         { "insert": " world!" },
    ///     ])
    /// );
    /// text.unmark(3..5, "bold").unwrap();
    /// assert_eq!(
    ///     text.get_richtext_value().to_json_value(),
    ///     json!([
    ///         { "insert": "Hel", "attributes": {"bold": true} },
    ///         { "insert": "lo world!" },
    ///    ])
    /// );
    /// ```
    pub fn get_richtext_value(&self) -> LoroValue {
        self.0.get_richtext_value().into()
    }

    /// Get the text in [Delta](https://quilljs.com/docs/delta/) format.
    pub fn to_delta(&self) -> Vec<TextDelta> {
        self.0.to_delta().iter().map(|x| x.into()).collect()
    }

    /// Get the text content of the text container.
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    /// Get the cursor at the given position in the given Unicode position.
    ///
    /// Using "index" to denote cursor positions can be unstable, as positions may
    /// shift with document edits. To reliably represent a position or range within
    /// a document, it is more effective to leverage the unique ID of each item/character
    /// in a List CRDT or Text CRDT.
    ///
    /// Loro optimizes State metadata by not storing the IDs of deleted elements. This
    /// approach complicates tracking cursors since they rely on these IDs. The solution
    /// recalculates position by replaying relevant history to update stable positions
    /// accurately. To minimize the performance impact of history replay, the system
    /// updates cursor info to reference only the IDs of currently present elements,
    /// thereby reducing the need for replay.
    ///
    /// # Example
    ///
    /// ```
    /// # use loro::{LoroDoc, ToJson};
    /// let doc = LoroDoc::new();
    /// let text = &doc.get_text("text");
    /// text.insert(0, "01234").unwrap();
    /// let pos = text.get_cursor(5, Default::default()).unwrap();
    /// assert_eq!(doc.get_cursor_pos(&pos).unwrap().current.pos, 5);
    /// text.insert(0, "01234").unwrap();
    /// assert_eq!(doc.get_cursor_pos(&pos).unwrap().current.pos, 10);
    /// text.delete(0, 10).unwrap();
    /// assert_eq!(doc.get_cursor_pos(&pos).unwrap().current.pos, 0);
    /// text.insert(0, "01234").unwrap();
    /// assert_eq!(doc.get_cursor_pos(&pos).unwrap().current.pos, 5);
    /// ```
    pub fn get_cursor(&self, pos: usize, side: Side) -> Option<Cursor> {
        self.0.get_cursor(pos, side.into()).map(|x| x.into())
    }

    /// Whether the text container is deleted.
    pub fn is_deleted(&self) -> bool {
        self.0.is_deleted()
    }

    /// Push a string to the end of the text container.
    pub fn push_str(&self, s: &str) -> PyLoroResult<()> {
        self.0.push_str(s)?;
        Ok(())
    }

    /// Get the editor of the text at the given position.
    pub fn get_editor_at_unicode_pos(&self, pos: usize) -> Option<PeerID> {
        self.0.get_editor_at_unicode_pos(pos)
    }

    pub fn doc(&self) -> Option<LoroDoc> {
        self.0.doc().map(|doc| doc.into())
    }

    /// Subscribe the events of a container.
    ///
    /// The callback will be invoked when the container is changed.
    /// Returns a subscription that can be used to unsubscribe.
    ///
    /// The events will be emitted after a transaction is committed. A transaction is committed when:
    ///
    /// - `doc.commit()` is called.
    /// - `doc.export(mode)` is called.
    /// - `doc.import(data)` is called.
    /// - `doc.checkout(version)` is called.
    pub fn subscribe(&self, callback: Py<PyAny>) -> Option<Subscription> {
        let subscription = self.0.subscribe(Arc::new(move |e| {
            Python::attach(|py| {
                callback.call1(py, (DiffEvent::from(e),)).unwrap();
            });
        }));
        subscription.map(|s| s.into())
    }
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    Left = -1,
    Middle = 0,
    Right = 1,
}

#[pyclass(str, frozen)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cursor(pub loro::cursor::Cursor);

impl Display for Cursor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cursor(id={:?}, container={:?}, side={:?})",
            self.0.id, self.0.container, self.0.side,
        )
    }
}

#[pymethods]
impl Cursor {
    #[getter]
    pub fn id(&self) -> Option<ID> {
        self.0.id.map(|x| x.into())
    }

    /// The target position is at the left, middle, or right of the given id.
    ///
    /// Side info can help to model the selection
    #[getter]
    pub fn side(&self) -> Side {
        self.0.side.into()
    }

    #[getter]
    pub fn container(&self) -> ContainerID {
        self.0.container.clone().into()
    }

    pub fn encode(&self) -> Vec<u8> {
        self.0.encode()
    }

    #[staticmethod]
    pub fn decode(bytes: Bound<'_, PyBytes>) -> PyLoroResult<Self> {
        let cursor = loro::cursor::Cursor::decode(bytes.as_bytes())
            .map_err(|e| PyLoroError::Error(e.to_string()))?;
        Ok(Self(cursor))
    }
}
