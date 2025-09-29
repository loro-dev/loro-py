use std::sync::Arc;

use crate::{
    doc::LoroDoc,
    err::PyLoroResult,
    event::{DiffEvent, Subscription},
    value::ContainerID,
};
use loro::{ContainerTrait, LoroCounter as LoroCounterInner};
use pyo3::{exceptions::PyTypeError, prelude::*, Bound, PyRef};

impl LoroCounter {
    fn coerce_to_f64(other: &Bound<'_, PyAny>) -> PyLoroResult<f64> {
        if let Ok(value) = other.extract::<f64>() {
            Ok(value)
        } else if let Ok(counter) = other.extract::<PyRef<LoroCounter>>() {
            Ok(counter.get_value())
        } else {
            Err(PyTypeError::new_err("expected a number or LoroCounter").into())
        }
    }
}

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
    pub fn increment(&self, py: Python, value: Py<PyAny>) -> PyLoroResult<()> {
        self.0.increment(value.extract::<f64>(py)?)?;
        Ok(())
    }

    /// Decrement the counter by the given value.
    pub fn decrement(&self, py: Python, value: Py<PyAny>) -> PyLoroResult<()> {
        self.0.decrement(value.extract::<f64>(py)?)?;
        Ok(())
    }

    /// Get the current value of the counter.
    #[getter]
    #[pyo3(name = "value")]
    pub fn get_value(&self) -> f64 {
        self.0.get_value()
    }

    pub fn __float__(&self) -> f64 {
        self.0.get_value()
    }

    pub fn __int__(&self) -> PyLoroResult<i64> {
        let value = self.0.get_value();
        if !value.is_finite() {
            return Err(PyTypeError::new_err("cannot convert non-finite counter to int").into());
        }
        if value < i64::MIN as f64 || value > i64::MAX as f64 {
            return Err(PyTypeError::new_err("counter value out of range for int").into());
        }
        Ok(value.trunc() as i64)
    }

    pub fn __add__(&self, other: Bound<'_, PyAny>) -> PyLoroResult<f64> {
        let delta = Self::coerce_to_f64(&other)?;
        Ok(self.0.get_value() + delta)
    }

    pub fn __radd__(&self, other: Bound<'_, PyAny>) -> PyLoroResult<f64> {
        self.__add__(other)
    }

    pub fn __sub__(&self, other: Bound<'_, PyAny>) -> PyLoroResult<f64> {
        let delta = Self::coerce_to_f64(&other)?;
        Ok(self.0.get_value() - delta)
    }

    pub fn __rsub__(&self, other: Bound<'_, PyAny>) -> PyLoroResult<f64> {
        let delta = Self::coerce_to_f64(&other)?;
        Ok(delta - self.0.get_value())
    }

    pub fn __neg__(&self) -> f64 {
        -self.0.get_value()
    }

    pub fn __abs__(&self) -> f64 {
        self.0.get_value().abs()
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
