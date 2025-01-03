use crate::{err::PyLoroResult, value::ContainerID};
use loro::LoroCounter as LoroCounterInner;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

#[gen_stub_pyclass]
#[pyclass(frozen)]
#[derive(Debug, Clone, Default)]
pub struct LoroCounter(pub LoroCounterInner);

#[gen_stub_pymethods]
#[pymethods]
impl LoroCounter {
    /// Create a new Counter.
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    /// Return container id of the Counter.
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
}
