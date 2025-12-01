use std::sync::Arc;

use loro::{ContainerTrait, LoroList as LoroListInner};
use pyo3::prelude::*;
use pyo3::{
    exceptions::{PyIndexError, PyValueError},
    BoundObject,
};

use crate::container::utils::{py_any_to_loro_values, slice_indices_positions, SliceOrInt};
use crate::{
    doc::LoroDoc,
    err::{PyLoroError, PyLoroResult},
    event::{DiffEvent, Subscription},
    value::{ContainerID, LoroValue, ValueOrContainer, ID},
};

use super::{Container, Cursor, Side};

#[pyclass(frozen, sequence)]
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
    pub fn for_each(&self, f: Py<PyAny>) {
        Python::attach(|py| {
            self.0.for_each(&mut |v| {
                f.call1(py, (ValueOrContainer::from(v),)).unwrap();
            });
        })
    }

    pub fn __getitem__<'py>(
        &self,
        py: Python<'py>,
        index: SliceOrInt<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        match index {
            SliceOrInt::Slice(slice) => {
                let indices = slice.indices(self.0.len() as isize)?;
                let mut i = indices.start;
                let mut list: Vec<ValueOrContainer> = Vec::with_capacity(indices.slicelength);

                for _ in 0..indices.slicelength {
                    list.push(
                        self.0
                            .get(i as usize)
                            .ok_or(PyIndexError::new_err("index out of range"))?
                            .into(),
                    );
                    i += indices.step;
                }
                list.into_pyobject(py)
            }
            SliceOrInt::Int(idx) => {
                let value: ValueOrContainer = self
                    .0
                    .get(idx)
                    .ok_or(PyIndexError::new_err("index out of range"))?
                    .into();
                Ok(value.into_pyobject(py)?.into_any().into_bound())
            }
        }
    }

    pub fn __setitem__<'py>(
        &self,
        index: SliceOrInt<'py>,
        value: Bound<'py, PyAny>,
    ) -> PyLoroResult<()> {
        match index {
            SliceOrInt::Int(idx) => {
                let extracted: LoroValue = value.extract().map_err(PyLoroError::from)?;
                self.0.delete(idx, 1).map_err(PyLoroError::from)?;
                self.0.insert(idx, extracted.0).map_err(PyLoroError::from)?;
                Ok(())
            }
            SliceOrInt::Slice(slice) => {
                let len = self.__len__() as isize;
                let indices = slice.indices(len).map_err(PyLoroError::from)?;

                let values = py_any_to_loro_values(&value).map_err(PyLoroError::from)?;

                if indices.step == 1 {
                    self.0
                        .delete(indices.start as usize, indices.slicelength)
                        .map_err(PyLoroError::from)?;

                    let mut pos = indices.start as usize;
                    for v in values {
                        self.0.insert(pos, v).map_err(PyLoroError::from)?;
                        pos += 1;
                    }
                    Ok(())
                } else {
                    if values.len() != indices.slicelength {
                        return Err(PyValueError::new_err(format!(
                            "attempt to assign sequence of size {} to extended slice of size {}",
                            values.len(),
                            indices.slicelength
                        ))
                        .into());
                    }

                    let positions = slice_indices_positions(&indices);
                    for (pos, v) in positions.into_iter().zip(values.into_iter()) {
                        self.0.delete(pos, 1).map_err(PyLoroError::from)?;
                        self.0.insert(pos, v).map_err(PyLoroError::from)?;
                    }
                    Ok(())
                }
            }
        }
    }

    pub fn __delitem__<'py>(&self, index: SliceOrInt<'py>) -> PyLoroResult<()> {
        match index {
            SliceOrInt::Int(idx) => self.delete(idx, 1),
            SliceOrInt::Slice(slice) => {
                let len = self.__len__() as isize;
                let indices = slice.indices(len).map_err(PyLoroError::from)?;

                if indices.slicelength == 0 {
                    return Ok(());
                }

                if indices.step == 1 {
                    self.delete(indices.start as usize, indices.slicelength)
                } else {
                    let mut positions = slice_indices_positions(&indices);
                    positions.sort_unstable();
                    for pos in positions.into_iter().rev() {
                        self.delete(pos, 1)?;
                    }
                    Ok(())
                }
            }
        }
    }

    /// Get the length of the list.
    #[inline]
    pub fn __len__(&self) -> usize {
        self.0.len()
    }

    /// Whether the list is empty.
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
    pub fn insert_container(&self, pos: usize, child: Container) -> PyLoroResult<Container> {
        let container = self.0.insert_container(pos, loro::Container::from(child))?;
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
