use std::sync::Arc;

use loro::{ContainerTrait, LoroMap as LoroMapInner, PeerID};
use pyo3::prelude::*;

use crate::{
    doc::LoroDoc,
    err::PyLoroResult,
    event::{DiffEvent, Subscription},
    value::{ContainerID, LoroValue, ValueOrContainer},
};

use super::Container;

pub fn register_class(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<LoroMap>()?;
    Ok(())
}

#[pyclass(frozen)]
#[derive(Debug, Clone, Default)]
pub struct LoroMap(pub LoroMapInner);

#[pymethods]
impl LoroMap {
    /// Create a new container that is detached from the document.
    ///
    /// The edits on a detached container will not be persisted.
    /// To attach the container to the document, please insert it into an attached container.
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the container is attached to a document.
    #[getter]
    pub fn is_attached(&self) -> bool {
        self.0.is_attached()
    }

    /// Delete a key-value pair from the map.
    pub fn delete(&self, key: &str) -> PyLoroResult<()> {
        self.0.delete(key)?;
        Ok(())
    }

    /// Iterate over the key-value pairs of the map.
    // TODO: why valueOrHandler?
    pub fn for_each(&self, f: Py<PyAny>) {
        Python::attach(|py| {
            self.0.for_each(move |key, value| {
                f.call1(py, (key, ValueOrContainer::from(value))).unwrap();
            })
        });
    }

    /// Insert a key-value pair into the map.
    pub fn insert(&self, key: &str, value: LoroValue) -> PyLoroResult<()> {
        self.0.insert(key, value)?;
        Ok(())
    }

    /// Get the length of the map.
    pub fn __len__(&self) -> usize {
        self.0.len()
    }

    /// Get the ID of the map.
    #[getter]
    pub fn id(&self) -> ContainerID {
        self.0.id().clone().into()
    }

    /// Whether the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the value of the map with the given key.
    pub fn get(&self, key: &str) -> Option<ValueOrContainer> {
        self.0.get(key).map(|v| v.into())
    }

    /// Insert a container with the given type at the given key.
    ///
    /// # Example
    ///
    /// ```
    /// # use loro::{LoroDoc, LoroText, ContainerType, ToJson};
    /// # use serde_json::json;
    /// let doc = LoroDoc::new();
    /// let map = doc.get_map("m");
    /// let text = map.insert_container("t", LoroText::new()).unwrap();
    /// text.insert(0, "12");
    /// text.insert(0, "0");
    /// assert_eq!(doc.get_deep_value().to_json_value(), json!({"m": {"t": "012"}}));
    /// ```
    pub fn insert_container(&self, key: &str, child: Container) -> PyLoroResult<Container> {
        let container = self.0.insert_container(key, loro::Container::from(child))?;
        Ok(container.into())
    }

    /// Get the shallow value of the map.
    ///
    /// It will not convert the state of sub-containers, but represent them as [LoroValue::Container].
    pub fn get_value(&self) -> LoroValue {
        self.0.get_value().into()
    }

    /// Get the deep value of the map.
    ///
    /// It will convert the state of sub-containers into a nested JSON value.
    pub fn get_deep_value(&self) -> LoroValue {
        self.0.get_deep_value().into()
    }

    /// Get or create a container with the given key.
    pub fn get_or_create_container(&self, key: &str, child: Container) -> PyLoroResult<Container> {
        let container = self
            .0
            .get_or_create_container(key, loro::Container::from(child))?;
        Ok(container.into())
    }

    /// Delete all key-value pairs in the map.
    pub fn clear(&self) -> PyLoroResult<()> {
        self.0.clear()?;
        Ok(())
    }

    // TODO: iter
    /// Get the keys of the map.
    pub fn keys(&self) -> Vec<String> {
        self.0.keys().map(|k| k.to_string()).collect()
    }

    /// Get the values of the map.
    pub fn values(&self) -> Vec<ValueOrContainer> {
        self.0.values().map(ValueOrContainer::from).collect()
    }

    /// Get the peer id of the last editor on the given entry
    pub fn get_last_editor(&self, key: &str) -> Option<PeerID> {
        self.0.get_last_editor(key)
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
