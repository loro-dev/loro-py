use loro::{ContainerTrait, LoroUnknown as LoroUnknownInner};
use pyo3::prelude::*;

use crate::{doc::LoroDoc, value::ContainerID};

#[pyclass(frozen, from_py_object)]
#[derive(Debug, Clone)]
pub struct LoroUnknown(pub LoroUnknownInner);

#[pymethods]
impl LoroUnknown {
    /// Get the container id.
    #[getter]
    pub fn id(&self) -> ContainerID {
        self.0.id().into()
    }

    pub fn doc(&self) -> Option<LoroDoc> {
        self.0.doc().map(|doc| doc.into())
    }
}
