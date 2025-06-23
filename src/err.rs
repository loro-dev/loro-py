use loro::{CannotFindRelativePosition, ChangeTravelError, LoroEncodeError};
use pyo3::{exceptions::PyBaseException, PyErr};

pub enum PyLoroError {
    LoroError(loro::LoroError),
    CannotFindRelativePosition(CannotFindRelativePosition),
    LoroEncodeError(LoroEncodeError),
    ChangeTravelError(ChangeTravelError),
    PyError(PyErr),
    Error(String),
}

pub type PyLoroResult<T> = Result<T, PyLoroError>;

impl From<loro::LoroError> for PyLoroError {
    fn from(other: loro::LoroError) -> Self {
        Self::LoroError(other)
    }
}

impl From<CannotFindRelativePosition> for PyLoroError {
    fn from(other: CannotFindRelativePosition) -> Self {
        Self::CannotFindRelativePosition(other)
    }
}

impl From<LoroEncodeError> for PyLoroError {
    fn from(other: LoroEncodeError) -> Self {
        Self::LoroEncodeError(other)
    }
}

impl From<ChangeTravelError> for PyLoroError {
    fn from(other: ChangeTravelError) -> Self {
        Self::ChangeTravelError(other)
    }
}

impl From<PyLoroError> for PyErr {
    fn from(value: PyLoroError) -> Self {
        match value {
            PyLoroError::LoroError(e) => PyBaseException::new_err(e.to_string()),
            PyLoroError::CannotFindRelativePosition(e) => PyBaseException::new_err(e.to_string()),
            PyLoroError::LoroEncodeError(e) => PyBaseException::new_err(e.to_string()),
            PyLoroError::ChangeTravelError(e) => PyBaseException::new_err(e.to_string()),
            PyLoroError::PyError(e) => e,
            PyLoroError::Error(e) => PyBaseException::new_err(e),
        }
    }
}

impl From<PyErr> for PyLoroError {
    fn from(value: PyErr) -> Self {
        Self::PyError(value)
    }
}
