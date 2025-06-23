use std::sync::Arc;

use crate::{
    doc::LoroDoc,
    err::PyLoroResult,
    event::{DiffEvent, Subscription},
    value::ContainerID,
};
use loro::{ContainerTrait, LoroCounter as LoroCounterInner};
use pyo3::prelude::*;

#[pyclass(frozen)]
#[derive(Debug, Clone, Default)]
pub struct LoroCounter(pub LoroCounterInner);

#[pymethods]
impl LoroCounter {
    /// Create a new Counter.
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    /// Return container id of the Counter.
    #[getter]
    pub fn id(&self) -> ContainerID {
        self.0.id().into()
    }

    /// Increment the counter by the given value.
    pub fn increment(&self, py: Python, value: PyObject) -> PyLoroResult<()> {
        self.0.increment(value.extract::<f64>(py)?)?;
        Ok(())
    }

    /// Decrement the counter by the given value.
    pub fn decrement(&self, py: Python, value: PyObject) -> PyLoroResult<()> {
        self.0.decrement(value.extract::<f64>(py)?)?;
        Ok(())
    }

    /// Get the current value of the counter.
    #[getter]
    #[pyo3(name = "value")]
    pub fn get_value(&self) -> f64 {
        self.0.get_value()
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
    pub fn subscribe(&self, callback: PyObject) -> Option<Subscription> {
        let subscription = self.0.subscribe(Arc::new(move |e| {
            Python::with_gil(|py| {
                callback.call1(py, (DiffEvent::from(e),)).unwrap();
            });
        }));
        subscription.map(|s| s.into())
    }
}
