use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

use crate::{
    container::Cursor,
    doc::{AbsolutePosition, CounterSpan, LoroDoc},
    err::PyLoroResult,
    event::DiffEvent,
    value::LoroValue,
};

pub fn register_class(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<UndoManager>()?;
    m.add_class::<UndoOrRedo>()?;
    Ok(())
}

#[gen_stub_pyclass]
#[pyclass]
pub struct UndoManager(loro::UndoManager);

#[gen_stub_pymethods]
#[pymethods]
impl UndoManager {
    /// Create a new UndoManager.
    #[new]
    pub fn new(doc: &LoroDoc) -> Self {
        Self(loro::UndoManager::new(&doc.doc))
    }

    /// Undo the last change made by the peer.
    pub fn undo(&mut self, doc: &LoroDoc) -> PyLoroResult<bool> {
        Ok(self.0.undo(&doc.doc)?)
    }

    /// Redo the last change made by the peer.
    pub fn redo(&mut self, doc: &LoroDoc) -> PyLoroResult<bool> {
        Ok(self.0.redo(&doc.doc)?)
    }

    /// Record a new checkpoint.
    pub fn record_new_checkpoint(&mut self, doc: &LoroDoc) -> PyLoroResult<()> {
        Ok(self.0.record_new_checkpoint(&doc.doc)?)
    }

    /// Whether the undo manager can undo.
    pub fn can_undo(&self) -> bool {
        self.0.can_undo()
    }

    /// Whether the undo manager can redo.
    pub fn can_redo(&self) -> bool {
        self.0.can_redo()
    }

    /// If a local event's origin matches the given prefix, it will not be recorded in the
    /// undo stack.
    pub fn add_exclude_origin_prefix(&mut self, prefix: &str) {
        self.0.add_exclude_origin_prefix(prefix)
    }

    /// Set the maximum number of undo steps. The default value is 100.
    pub fn set_max_undo_steps(&mut self, size: usize) {
        self.0.set_max_undo_steps(size)
    }

    /// Set the merge interval in ms. The default value is 0, which means no merge.
    pub fn set_merge_interval(&mut self, interval: i64) {
        self.0.set_merge_interval(interval)
    }

    /// Set the listener for push events.
    /// The listener will be called when a new undo/redo item is pushed into the stack.
    #[pyo3(signature = (on_push=None))]
    pub fn set_on_push(&mut self, on_push: Option<PyObject>) {
        if let Some(on_push) = on_push {
            self.0
                .set_on_push(Some(Box::new(move |undo_or_redo, span, event| {
                    Python::with_gil(|py| {
                        let meta = on_push
                            .call1(
                                py,
                                (
                                    UndoOrRedo::from(undo_or_redo),
                                    CounterSpan::from(span),
                                    event.map(|x| DiffEvent::from(loro::event::DiffEvent::from(x))),
                                ),
                            )
                            .unwrap()
                            .extract::<UndoItemMeta>(py)
                            .unwrap();
                        loro::undo::UndoItemMeta::from(meta)
                    })
                })));
        } else {
            self.0.set_on_push(None);
        }
    }

    /// Set the listener for pop events.
    /// The listener will be called when an undo/redo item is popped from the stack.
    #[pyo3(signature = (on_pop=None))]
    pub fn set_on_pop(&mut self, on_pop: Option<PyObject>) {
        if let Some(on_pop) = on_pop {
            self.0
                .set_on_pop(Some(Box::new(move |undo_or_redo, span, meta| {
                    Python::with_gil(|py| {
                        on_pop
                            .call1(
                                py,
                                (
                                    UndoOrRedo::from(undo_or_redo),
                                    CounterSpan::from(span),
                                    UndoItemMeta::from(meta),
                                ),
                            )
                            .unwrap();
                    })
                })));
        } else {
            self.0.set_on_pop(None);
        }
    }

    /// Clear the undo stack and the redo stack
    pub fn clear(&self) {
        self.0.clear();
    }
}

#[gen_stub_pyclass_enum]
#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UndoOrRedo {
    Undo,
    Redo,
}

#[derive(Debug, Clone, IntoPyObject, FromPyObject)]
pub struct UndoItemMeta {
    pub value: LoroValue,
    pub cursors: Vec<CursorWithPos>,
}

#[derive(Debug, Clone, IntoPyObject, FromPyObject)]
pub struct CursorWithPos {
    pub cursor: Cursor,
    pub pos: AbsolutePosition,
}
