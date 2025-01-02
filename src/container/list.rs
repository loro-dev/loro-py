use loro::LoroList as LoroListInner;
use pyo3::prelude::*;

use crate::{
    err::PyLoroResult,
    value::{ContainerID, LoroValue, ValueOrContainer, ID},
};

use super::{Container, Cursor, Side};

#[pyclass(frozen)]
#[derive(Debug, Clone, Default)]
pub struct LoroList(pub LoroListInner);

#[pymethods]
impl LoroList {
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

    /// Insert a value at the given position.
    pub fn insert(&self, pos: usize, v: LoroValue) -> PyLoroResult<()> {
        self.0.insert(pos, &v)?;
        Ok(())
    }

    /// Delete values at the given position.
    #[inline]
    pub fn delete(&self, pos: usize, len: usize) -> PyLoroResult<()> {
        self.0.delete(pos, len)?;
        Ok(())
    }

    /// Get the value at the given position.
    #[inline]
    pub fn get(&self, index: usize) -> Option<ValueOrContainer> {
        self.0.get(index).map(ValueOrContainer::from)
    }

    /// Get the deep value of the container.
    #[inline]
    pub fn get_deep_value(&self) -> LoroValue {
        self.0.get_deep_value().into()
    }

    /// Get the shallow value of the container.
    ///
    /// This does not convert the state of sub-containers; instead, it represents them as [LoroValue::Container].
    #[inline]
    pub fn get_value(&self) -> LoroValue {
        self.0.get_value().into()
    }

    /// Get the ID of the container.
    #[getter]
    #[inline]
    pub fn id(&self) -> ContainerID {
        self.0.id().into()
    }

    /// Pop the last element of the list.
    #[inline]
    pub fn pop(&self) -> PyLoroResult<Option<LoroValue>> {
        let ans = self.0.pop()?.map(LoroValue::from);
        Ok(ans)
    }

    /// Push a value to the list.
    #[inline]
    pub fn push(&self, v: LoroValue) -> PyLoroResult<()> {
        self.0.push(&v)?;
        Ok(())
    }

    /// Push a container to the list.
    #[inline]
    pub fn push_container(&self, child: Container) -> PyLoroResult<Container> {
        let container = self.0.push_container(loro::Container::from(child))?;
        Ok(container.into())
    }

    /// Iterate over the elements of the list.
    pub fn for_each(&self, f: PyObject) {
        Python::with_gil(|py| {
            self.0.for_each(&mut |v| {
                f.call1(py, (ValueOrContainer::from(v),)).unwrap();
            });
        })
    }

    /// Get the length of the list.
    #[inline]
    pub fn __len__(&self) -> usize {
        self.0.len()
    }

    /// Whether the list is empty.
    #[getter]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Insert a container with the given type at the given index.
    ///
    /// # Example
    ///
    /// ```
    /// # use loro::{LoroDoc, ContainerType, LoroText, ToJson};
    /// # use serde_json::json;
    /// let doc = LoroDoc::new();
    /// let list = doc.get_list("m");
    /// let text = list.insert_container(0, LoroText::new()).unwrap();
    /// text.insert(0, "12");
    /// text.insert(0, "0");
    /// assert_eq!(doc.get_deep_value().to_json_value(), json!({"m": ["012"]}));
    /// ```
    #[inline]
    pub fn insert_container(
        &self,
        py: Python,
        pos: usize,
        child: PyObject,
    ) -> PyLoroResult<Container> {
        let container = self
            .0
            .insert_container(pos, loro::Container::from(child.extract::<Container>(py)?))?;
        Ok(container.into())
    }

    /// Get the cursor at the given position.
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
    /// use loro::LoroDoc;
    /// use loro_internal::cursor::Side;
    ///
    /// let doc = LoroDoc::new();
    /// let list = doc.get_list("list");
    /// list.insert(0, 0).unwrap();
    /// let cursor = list.get_cursor(0, Side::Middle).unwrap();
    /// assert_eq!(doc.get_cursor_pos(&cursor).unwrap().current.pos, 0);
    /// list.insert(0, 0).unwrap();
    /// assert_eq!(doc.get_cursor_pos(&cursor).unwrap().current.pos, 1);
    /// list.insert(0, 0).unwrap();
    /// list.insert(0, 0).unwrap();
    /// assert_eq!(doc.get_cursor_pos(&cursor).unwrap().current.pos, 3);
    /// list.insert(4, 0).unwrap();
    /// assert_eq!(doc.get_cursor_pos(&cursor).unwrap().current.pos, 3);
    /// ```
    pub fn get_cursor(&self, pos: usize, side: Side) -> Option<Cursor> {
        self.0.get_cursor(pos, side.into()).map(Cursor::from)
    }

    /// Converts the LoroList to a Vec of LoroValue.
    ///
    /// This method unwraps the internal Arc and clones the data if necessary,
    /// returning a Vec containing all the elements of the LoroList as LoroValue.
    ///
    /// # Returns
    ///
    /// A Vec<LoroValue> containing all elements of the LoroList.
    ///
    /// # Example
    ///
    /// ```
    /// use loro::{LoroDoc, LoroValue};
    ///
    /// let doc = LoroDoc::new();
    /// let list = doc.get_list("my_list");
    /// list.insert(0, 1).unwrap();
    /// list.insert(1, "hello").unwrap();
    /// list.insert(2, true).unwrap();
    ///
    /// let vec = list.to_vec();
    /// ```
    pub fn to_vec(&self) -> Vec<LoroValue> {
        self.0.to_vec().into_iter().map(LoroValue::from).collect()
    }

    /// Delete all elements in the list.
    pub fn clear(&self) -> PyLoroResult<()> {
        self.0.clear()?;
        Ok(())
    }

    /// Get the ID of the list item at the given position.
    pub fn get_id_at(&self, pos: usize) -> Option<ID> {
        self.0.get_id_at(pos).map(ID::from)
    }
}
