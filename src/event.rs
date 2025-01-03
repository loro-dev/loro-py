use crate::value::{ContainerID, LoroValue, TreeID, TreeParentId, ValueOrContainer};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use pyo3_stub_gen::derive::*;
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
    Ok(())
}

#[derive(Debug, IntoPyObject)]
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

#[gen_stub_pyclass_enum]
/// The kind of the event trigger.
#[pyclass(eq, eq_int)]
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

#[derive(Debug, Clone, IntoPyObject)]
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

#[derive(Debug, Clone, IntoPyObject)]
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

#[gen_stub_pyclass_enum]
#[derive(Debug, Clone, IntoPyObject, FromPyObject)]
pub enum Index {
    Key(String),
    Seq(u32),
    Node(TreeID),
}

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Index::Key(key) => write!(f, "Key(key='{}')", key),
            Index::Seq(index) => write!(f, "Seq(index={})", index),
            Index::Node(target) => write!(f, "Node(target={})", target),
        }
    }
}

#[derive(Debug, Clone, IntoPyObject)]
pub enum Diff {
    /// A list diff.
    List(Vec<ListDiffItem>),
    /// A text diff.
    Text(Vec<TextDelta>),
    /// A map diff.
    Map(MapDelta),
    /// A tree diff.
    Tree(TreeDiff),
    /// A counter diff.
    Counter(f64),
    /// An unknown diff.
    Unknown(()),
}

impl fmt::Display for Diff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Diff::List(diff) => write!(
                f,
                "List([{}])",
                diff.iter()
                    .map(|d| format!("{}", d))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Diff::Text(diff) => write!(
                f,
                "Text([{}])",
                diff.iter()
                    .map(|d| format!("{}", d))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Diff::Map(diff) => write!(f, "Map({})", diff),
            Diff::Tree(diff) => write!(f, "Tree({})", diff),
            Diff::Counter(diff) => write!(f, "Counter({})", diff),
            Diff::Unknown(()) => write!(f, "Unknown()"),
        }
    }
}

#[gen_stub_pyclass_enum]
#[derive(Debug, Clone, IntoPyObject, FromPyObject)]
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

#[gen_stub_pyclass_enum]
#[pyclass]
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

#[gen_stub_pyclass]
#[pyclass(str, get_all, set_all)]
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

#[gen_stub_pyclass]
#[pyclass(str, get_all, set_all)]
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

#[gen_stub_pyclass]
#[pyclass(str, get_all, set_all)]
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

#[gen_stub_pyclass_enum]
#[pyclass(str, get_all, set_all)]
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

#[gen_stub_pyclass]
#[pyclass(frozen)]
pub struct Subscription(pub(crate) Mutex<Option<loro::Subscription>>);

#[gen_stub_pymethods]
#[pymethods]
impl Subscription {
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
