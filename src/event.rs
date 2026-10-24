use crate::value::{ContainerID, LoroValue, TreeID, TreeParentId, ValueOrContainer};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;

pub fn register_class(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Subscription>()?;
    m.add_class::<EventTriggerKind>()?;
    m.add_class::<ListDiffItem>()?;
    m.add_class::<MapDelta>()?;
    m.add_class::<TreeDiff>()?;
    m.add_class::<TreeDiffItem>()?;
    m.add_class::<TreeExternalDiff>()?;
    m.add_class::<DiffEvent>()?;
    m.add_class::<TextDelta>()?;
    m.add_class::<PathItem>()?;
    m.add_class::<ContainerDiff>()?;
    m.add_class::<Index>()?;
    m.add_class::<Diff>()?;
    m.add_class::<DiffBatch>()?;
    Ok(())
}

#[pyclass(get_all, str)]
#[derive(Debug)]
pub struct DiffEvent {
    /// How the event is triggered.
    pub triggered_by: EventTriggerKind,
    /// The origin of the event.
    pub origin: String,
    /// The current receiver of the event.
    pub current_target: Option<ContainerID>,
    /// The diffs of the event.
    pub events: Vec<ContainerDiff>,
}

impl fmt::Display for DiffEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DiffEvent(triggered_by={}, origin='{}', current_target={}, events={})",
            self.triggered_by,
            self.origin,
            self.current_target
                .as_ref()
                .map_or("None".to_string(), |v| format!("{}", v)),
            self.events
                .iter()
                .map(|e| format!("{}", e))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

/// The kind of the event trigger.
#[pyclass(eq, eq_int, from_py_object)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventTriggerKind {
    /// The event is triggered by a local transaction.
    Local,
    /// The event is triggered by importing
    Import,
    /// The event is triggered by checkout
    Checkout,
}

impl fmt::Display for EventTriggerKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventTriggerKind::Local => write!(f, "Local"),
            EventTriggerKind::Import => write!(f, "Import"),
            EventTriggerKind::Checkout => write!(f, "Checkout"),
        }
    }
}

#[pyclass(get_all, str, from_py_object)]
#[derive(Debug, Clone)]
pub struct PathItem {
    pub container: ContainerID,
    pub index: Index,
}

impl fmt::Display for PathItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PathItem(container={}, index={})",
            self.container, self.index
        )
    }
}

#[pyclass(get_all, str, from_py_object)]
#[derive(Debug, Clone)]
/// A diff of a container.
pub struct ContainerDiff {
    /// The target container id of the diff.
    pub target: ContainerID,
    /// The path of the diff.
    pub path: Vec<PathItem>,
    /// Whether the diff is from unknown container.
    pub is_unknown: bool,
    /// The diff
    pub diff: Diff,
}

impl fmt::Display for ContainerDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ContainerDiff(target={}, path=[{}], is_unknown={}, diff={})",
            self.target,
            self.path
                .iter()
                .map(|p| format!("{}", p))
                .collect::<Vec<_>>()
                .join(", "),
            self.is_unknown,
            self.diff
        )
    }
}

#[pyclass(str, get_all, from_py_object)]
#[derive(Debug, Clone)]
pub enum Index {
    Key { key: String },
    Seq { index: u32 },
    Node { target: TreeID },
}

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Index::Key { key } => write!(f, "Key(key='{}')", key),
            Index::Seq { index } => write!(f, "Seq(index={})", index),
            Index::Node { target } => write!(f, "Node(target={})", target),
        }
    }
}

#[pyclass(get_all, from_py_object)]
#[derive(Debug, Clone)]
pub enum Diff {
    /// A list diff.
    List { diff: Vec<ListDiffItem> },
    /// A text diff.
    Text { diff: Vec<TextDelta> },
    /// A map diff.
    Map { diff: MapDelta },
    /// A tree diff.
    Tree { diff: TreeDiff },
    /// A counter diff.
    Counter { diff: f64 },
    /// An unknown diff.
    Unknown {},
}

impl fmt::Display for Diff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Diff::List { diff } => write!(
                f,
                "List([{}])",
                diff.iter()
                    .map(|d| format!("{}", d))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Diff::Text { diff } => write!(
                f,
                "Text([{}])",
                diff.iter()
                    .map(|d| format!("{}", d))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Diff::Map { diff } => write!(f, "Map({})", diff),
            Diff::Tree { diff } => write!(f, "Tree({})", diff),
            Diff::Counter { diff } => write!(f, "Counter({})", diff),
            Diff::Unknown {} => write!(f, "Unknown"),
        }
    }
}

#[pyclass(str, get_all, from_py_object)]
#[derive(Debug, Clone)]
pub enum TextDelta {
    Retain {
        retain: usize,
        attributes: Option<HashMap<String, LoroValue>>,
    },
    Insert {
        insert: String,
        attributes: Option<HashMap<String, LoroValue>>,
    },
    Delete {
        delete: usize,
    },
}

impl fmt::Display for TextDelta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextDelta::Retain { retain, attributes } => {
                write!(
                    f,
                    "Retain(retain={}, attributes={})",
                    retain,
                    attributes.as_ref().map_or("None".to_string(), |a| format!(
                        "{{{}}}",
                        a.iter()
                            .map(|(k, v)| format!("'{}': {:?}", k, v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))
                )
            }
            TextDelta::Insert { insert, attributes } => {
                write!(
                    f,
                    "Insert(insert='{}', attributes={})",
                    insert,
                    attributes.as_ref().map_or("None".to_string(), |a| format!(
                        "{{{}}}",
                        a.iter()
                            .map(|(k, v)| format!("'{}': {:?}", k, v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))
                )
            }
            TextDelta::Delete { delete } => {
                write!(f, "Delete(delete={})", delete)
            }
        }
    }
}

#[pyclass(str, get_all, from_py_object)]
#[derive(Debug, Clone)]
pub enum ListDiffItem {
    /// Insert a new element into the list.
    Insert {
        /// The new elements to insert.
        insert: Vec<ValueOrContainer>,
        /// Whether the new elements are created by moving
        is_move: bool,
    },
    /// Delete n elements from the list at the current index.
    Delete {
        /// The number of elements to delete.
        delete: u32,
    },
    /// Retain n elements in the list.
    ///
    /// This is used to keep the current index unchanged.
    Retain {
        /// The number of elements to retain.
        retain: u32,
    },
}

impl fmt::Display for ListDiffItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ListDiffItem::Insert { insert, is_move } => {
                write!(
                    f,
                    "Insert(insert=[{}], is_move={})",
                    insert
                        .iter()
                        .map(|v| format!("{}", v))
                        .collect::<Vec<_>>()
                        .join(", "),
                    is_move
                )
            }
            ListDiffItem::Delete { delete } => {
                write!(f, "Delete(delete={})", delete)
            }
            ListDiffItem::Retain { retain } => {
                write!(f, "Retain(retain={})", retain)
            }
        }
    }
}

#[pyclass(str, get_all, from_py_object)]
#[derive(Debug, Clone)]
pub struct MapDelta {
    /// All the updated keys and their new values.
    pub updated: HashMap<String, Option<ValueOrContainer>>,
}

impl fmt::Display for MapDelta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MapDelta(updated={{{}}})",
            self.updated
                .iter()
                .map(|(k, v)| format!(
                    "'{}': {}",
                    k,
                    v.as_ref().map_or("None".to_string(), |v| format!("{}", v))
                ))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[pyclass(str, get_all, from_py_object)]
#[derive(Debug, Clone)]
pub struct TreeDiff {
    pub diff: Vec<TreeDiffItem>,
}

impl fmt::Display for TreeDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TreeDiff(diff=[{}])",
            self.diff
                .iter()
                .map(|d| format!("{}", d))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[pyclass(str, get_all, from_py_object)]
#[derive(Debug, Clone)]
pub struct TreeDiffItem {
    pub target: TreeID,
    pub action: TreeExternalDiff,
}

impl fmt::Display for TreeDiffItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TreeDiffItem(target={}, action={})",
            self.target, self.action
        )
    }
}

#[pyclass(str, get_all, from_py_object)]
#[derive(Debug, Clone)]
pub enum TreeExternalDiff {
    Create {
        parent: TreeParentId,
        index: u32,
        fractional_index: String,
    },
    Move {
        parent: TreeParentId,
        index: u32,
        fractional_index: String,
        old_parent: TreeParentId,
        old_index: u32,
    },
    Delete {
        old_parent: TreeParentId,
        old_index: u32,
    },
}

impl fmt::Display for TreeExternalDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TreeExternalDiff::Create {
                parent,
                index,
                fractional_index,
            } => {
                write!(
                    f,
                    "Create(parent={:?}, index={}, fractional_index='{}')",
                    parent, index, fractional_index
                )
            }
            TreeExternalDiff::Move {
                parent,
                index,
                fractional_index,
                old_parent,
                old_index,
            } => {
                write!(
                    f,
                    "Move(parent={:?}, index={}, fractional_index='{}', old_parent={:?}, old_index={})",
                    parent, index, fractional_index, old_parent, old_index
                )
            }
            TreeExternalDiff::Delete {
                old_parent,
                old_index,
            } => {
                write!(
                    f,
                    "Delete(old_parent={:?}, old_index={})",
                    old_parent, old_index
                )
            }
        }
    }
}

#[pyclass(frozen)]
pub struct Subscription(pub(crate) Mutex<Option<loro::Subscription>>);

#[pymethods]
impl Subscription {
    pub fn detach(&self) {
        let s = self.0.lock().unwrap().take();
        if let Some(s) = s {
            s.detach();
        }
    }

    pub fn unsubscribe(&self) {
        let s = self.0.lock().unwrap().take();
        if let Some(s) = s {
            s.unsubscribe();
        }
    }

    #[pyo3(signature = (*_args, **_kwargs))]
    pub fn __call__(
        &self,
        _py: Python<'_>,
        _args: &Bound<'_, PyTuple>,
        _kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        if let Ok(mut subscription) = self.0.lock() {
            if let Some(subscription) = std::mem::take(&mut *subscription) {
                subscription.unsubscribe();
            }
        }
        Ok(())
    }
}

#[pyclass(str, from_py_object)]
#[derive(Debug, Clone, Default)]
pub struct DiffBatch(loro::event::DiffBatch);

impl fmt::Display for DiffBatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DiffBatch({:?})", self.0)
    }
}

#[pymethods]
impl DiffBatch {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a new event to the batch.
    ///
    /// If the cid already exists in the batch, return Err
    pub fn push(&mut self, cid: ContainerID, diff: Diff) -> Option<Diff> {
        if let Err(diff) = self.0.push(cid.into(), diff.into()) {
            Some((&diff).into())
        } else {
            None
        }
    }

    // TODO: use iterator
    /// Returns an iterator over the diffs in this batch, in the order they were added.
    ///
    /// The iterator yields tuples of `(&ContainerID, &Diff)` where:
    /// - `ContainerID` is the ID of the container that was modified
    /// - `Diff` contains the actual changes made to that container
    ///
    /// The order of the diffs is preserved from when they were originally added to the batch.
    pub fn get_diff(&self) -> Vec<(ContainerID, Diff)> {
        self.0
            .iter()
            .map(|(cid, diff)| (cid.into(), diff.into()))
            .collect()
    }
}

impl From<DiffBatch> for loro::event::DiffBatch {
    fn from(value: DiffBatch) -> Self {
        value.0
    }
}

impl From<loro::event::DiffBatch> for DiffBatch {
    fn from(value: loro::event::DiffBatch) -> Self {
        Self(value)
    }
}
