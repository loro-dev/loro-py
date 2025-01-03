use loro::{Counter, PeerID};
use pyo3::{exceptions::PyValueError, prelude::*, BoundObject};
use pyo3_stub_gen::derive::*;
use std::fmt::Display;

use crate::{
    container::Container,
    convert::{loro_value_to_pyobject, pyobject_to_loro_value},
};

pub fn register_class(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ID>()?;
    m.add_class::<ContainerType>()?;
    m.add_class::<ContainerID>()?;
    m.add_class::<Ordering>()?;
    m.add_class::<TreeID>()?;
    Ok(())
}

#[gen_stub_pyclass]
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

#[gen_stub_pymethods]
#[pymethods]
impl ID {
    #[new]
    pub fn new(peer: u64, counter: i32) -> Self {
        Self { peer, counter }
    }
}

#[gen_stub_pyclass_enum]
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

#[gen_stub_pyclass_enum]
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

#[gen_stub_pyclass_enum]
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

#[gen_stub_pyclass]
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

#[gen_stub_pyclass_enum]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TreeParentId {
    Node(TreeID),
    Root,
    Deleted,
    Unexist,
}

impl<'py> FromPyObject<'py> for TreeParentId {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        if ob.is_instance_of::<TreeID>() {
            Ok(TreeParentId::Node(ob.extract::<TreeID>()?))
        } else if ob.is_none() {
            Ok(TreeParentId::Root)
        } else {
            Err(PyValueError::new_err("Invalid tree parent id"))
        }
    }
}

impl<'py> IntoPyObject<'py> for TreeParentId {
    type Target = PyAny;

    type Output = Bound<'py, PyAny>;

    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let ans = match self {
            TreeParentId::Node(id) => id.into_pyobject(py)?.into_any().into_bound(),
            TreeParentId::Root => py.None().into_pyobject(py)?.into_any().into_bound(),
            TreeParentId::Deleted | TreeParentId::Unexist => {
                return Err(PyValueError::new_err("Invalid tree parent id"))
            }
        };
        Ok(ans)
    }
}

#[gen_stub_pyclass_enum]
#[derive(Debug, Clone, FromPyObject, IntoPyObject)]
pub enum ValueOrContainer {
    Value(LoroValue),
    Container(Container),
}

impl Display for ValueOrContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[gen_stub_pyclass]
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
