use loro::LoroUnknown as LoroUnknownInner;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

use crate::value::ContainerID;

#[gen_stub_pyclass]
#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct LoroUnknown(pub LoroUnknownInner);

#[gen_stub_pymethods]
#[pymethods]
impl LoroUnknown {
    /// Get the container id.
    #[getter]
    pub fn id(&self) -> ContainerID {
        self.0.id().into()
    }
}
