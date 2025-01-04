use crate::{err::PyLoroResult, value::ID};
use pyo3::{basic::CompareOp, exceptions::PyNotImplementedError, prelude::*, types::PyType};
use std::fmt::Display;

pub fn register_class(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Frontiers>()?;
    m.add_class::<VersionRange>()?;
    m.add_class::<VersionVector>()?;
    Ok(())
}

#[pyclass(str)]
#[derive(Clone, Default)]
pub struct Frontiers(loro::Frontiers);

impl Display for Frontiers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[pymethods]
impl Frontiers {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    #[classmethod]
    pub fn from_id(_cls: &Bound<'_, PyType>, id: ID) -> Self {
        Self(loro::Frontiers::from(loro::ID::from(id)))
    }

    #[classmethod]
    pub fn from_ids(_cls: &Bound<'_, PyType>, ids: Vec<ID>) -> Self {
        Self(loro::Frontiers::from(
            ids.into_iter().map(loro::ID::from).collect::<Vec<_>>(),
        ))
    }

    pub fn encode(&self) -> Vec<u8> {
        self.0.encode()
    }

    #[classmethod]
    pub fn decode(_cls: &Bound<'_, PyType>, bytes: &[u8]) -> PyLoroResult<Self> {
        let ans = Self(loro::Frontiers::decode(bytes)?);
        Ok(ans)
    }
}

impl From<Frontiers> for loro::Frontiers {
    fn from(value: Frontiers) -> Self {
        value.0
    }
}

impl From<loro::Frontiers> for Frontiers {
    fn from(value: loro::Frontiers) -> Self {
        Self(value)
    }
}

impl From<&Frontiers> for loro::Frontiers {
    fn from(value: &Frontiers) -> Self {
        value.0.clone()
    }
}

#[pyclass(str)]
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct VersionRange(loro::VersionRange);

impl Display for VersionRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl From<VersionRange> for loro::VersionRange {
    fn from(value: VersionRange) -> Self {
        value.0
    }
}

impl From<loro::VersionRange> for VersionRange {
    fn from(value: loro::VersionRange) -> Self {
        Self(value)
    }
}

#[pyclass]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionVector(loro::VersionVector);

#[pymethods]
impl VersionVector {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    fn __richcmp__(&self, other: PyRef<Self>, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.0 == other.0),
            CompareOp::Ne => Ok(self.0 != other.0),
            _ => Err(PyNotImplementedError::new_err(
                "Only == and != comparisons are supported",
            )),
        }
    }
}

impl Default for VersionVector {
    fn default() -> Self {
        Self(loro::VersionVector::new())
    }
}

impl Display for VersionVector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl From<VersionVector> for loro::VersionVector {
    fn from(value: VersionVector) -> Self {
        value.0
    }
}

impl From<loro::VersionVector> for VersionVector {
    fn from(value: loro::VersionVector) -> Self {
        Self(value)
    }
}
