use crate::value::LoroValue as PyLoroValue;
use loro::LoroValue as CoreLoroValue;
use pyo3::{
    exceptions::PyTypeError,
    types::{PyAnyMethods, PySequence, PySequenceMethods, PySlice, PySliceIndices},
    Bound, FromPyObject, PyAny, PyResult,
};

#[derive(FromPyObject)]
pub enum SliceOrInt<'py> {
    Slice(Bound<'py, PySlice>),
    Int(usize),
}

pub fn py_any_to_loro_values(obj: &Bound<'_, PyAny>) -> PyResult<Vec<CoreLoroValue>> {
    let sequence = obj
        .downcast::<PySequence>()
        .map_err(|_| PyTypeError::new_err("can only assign an iterable to a slice"))?;

    let length = sequence.len()?;
    let mut values = Vec::with_capacity(length);
    for idx in 0..length {
        let element = sequence.get_item(idx)?;
        let extracted: PyLoroValue = element.extract()?;
        values.push(extracted.0);
    }
    Ok(values)
}

pub fn slice_indices_positions(indices: &PySliceIndices) -> Vec<usize> {
    let mut positions = Vec::with_capacity(indices.slicelength);
    let mut current = indices.start;
    for _ in 0..indices.slicelength {
        positions.push(current as usize);
        current += indices.step;
    }
    positions
}
