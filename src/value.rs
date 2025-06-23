use loro::{Counter, PeerID};
use pyo3::prelude::*;
use std::fmt::Display;

use crate::{
    container::Container,
    convert::{loro_value_to_pyobject, pyobject_to_loro_value},
};

pub fn register_class(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ID>()?;
    m.add_class::<IdLp>()?;
    m.add_class::<ContainerType>()?;
    m.add_class::<ContainerID>()?;
    m.add_class::<Ordering>()?;
    m.add_class::<TreeID>()?;
    m.add_class::<ValueOrContainer>()?;
    Ok(())
}

#[pyclass(eq, str, get_all, set_all)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ID {
    pub peer: u64,
    pub counter: i32,
}

impl Display for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[pymethods]
impl ID {
    #[new]
    pub fn new(peer: u64, counter: i32) -> Self {
        Self { peer, counter }
    }
}

#[pyclass(eq, str, get_all, set_all)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IdLp {
    pub peer: u64,
    pub lamport: i32,
}

impl Display for IdLp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[pyclass(eq, hash, str)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContainerType {
    Text {},
    Map {},
    List {},
    MovableList {},
    Tree {},
    Counter {},
    Unknown { kind: u8 },
}

impl Display for ContainerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[pyclass(eq, str, hash)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContainerID {
    Root {
        name: String,
        container_type: ContainerType,
    },
    Normal {
        peer: u64,
        counter: i32,
        container_type: ContainerType,
    },
}

impl Display for ContainerID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[pyclass(eq, str, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ordering {
    Less,
    Equal,
    Greater,
}

impl Display for Ordering {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<std::cmp::Ordering> for Ordering {
    fn from(value: std::cmp::Ordering) -> Self {
        match value {
            std::cmp::Ordering::Less => Ordering::Less,
            std::cmp::Ordering::Equal => Ordering::Equal,
            std::cmp::Ordering::Greater => Ordering::Greater,
        }
    }
}

#[pyclass(eq, str, get_all, set_all)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TreeID {
    pub peer: PeerID,
    pub counter: Counter,
}

impl Display for TreeID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[pymethods]
impl TreeID {
    #[new]
    pub fn new(peer: PeerID, counter: Counter) -> Self {
        Self { peer, counter }
    }
}

pub type TreeParentId = Option<TreeID>;

#[pyclass]
#[derive(Debug, Clone)]
pub enum ValueOrContainer {
    Value { value: LoroValue },
    Container { container: Container },
}

impl Display for ValueOrContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[pymethods]
impl ValueOrContainer {
    #[staticmethod]
    #[pyo3(signature = (value=None))]
    pub fn is_value(value: Option<&ValueOrContainer>) -> bool {
        if value.is_none() {
            return false;
        }

        matches!(value.unwrap(), ValueOrContainer::Value { .. })
    }

    #[staticmethod]
    #[pyo3(signature = (value=None))]
    pub fn is_container(value: Option<&ValueOrContainer>) -> bool {
        if value.is_none() {
            return false;
        }

        matches!(value.unwrap(), ValueOrContainer::Container { .. })
    }
}

#[derive(Debug, Clone)]
pub struct LoroValue(pub(crate) loro::LoroValue);

impl<'py> FromPyObject<'py> for LoroValue {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let value = pyobject_to_loro_value(ob)?;
        Ok(Self(value))
    }
}

impl<'py> IntoPyObject<'py> for LoroValue {
    type Target = PyAny;

    type Output = Bound<'py, PyAny>;

    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        loro_value_to_pyobject(py, self)
    }
}
