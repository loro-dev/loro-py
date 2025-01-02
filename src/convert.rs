use std::{borrow::Cow, collections::HashMap, sync::Mutex};

use fxhash::FxHashMap;
use pyo3::{
    exceptions::PyTypeError,
    prelude::*,
    types::{PyBool, PyBytes, PyDict, PyList, PyMapping, PyString},
    BoundObject, PyResult,
};

use crate::{
    container::{
        Container, Cursor, LoroCounter, LoroList, LoroMap, LoroMovableList, LoroText, LoroTree,
        LoroUnknown, Side, TreeNode, UpdateOptions,
    },
    doc::{
        AbsolutePosition, ChangeMeta, CounterSpan, EncodedBlobMode, ExportMode, IdSpan,
        ImportBlobMetadata, PosQueryResult,
    },
    event::{
        ContainerDiff, Diff, DiffEvent, EventTriggerKind, Index, ListDiffItem, MapDelta, PathItem,
        Subscription, TextDelta, TreeDiff, TreeDiffItem, TreeExternalDiff,
    },
    value::{ContainerID, ContainerType, LoroValue, TreeID, TreeParentId, ValueOrContainer, ID},
};

impl From<ID> for loro::ID {
    fn from(value: ID) -> Self {
        Self {
            peer: value.peer,
            counter: value.counter,
        }
    }
}

impl From<loro::ID> for ID {
    fn from(value: loro::ID) -> Self {
        Self {
            peer: value.peer,
            counter: value.counter,
        }
    }
}

pub fn pyobject_to_container_id(
    obj: &Bound<'_, PyAny>,
    ty: ContainerType,
) -> PyResult<loro::ContainerID> {
    if let Ok(value) = obj.downcast::<PyString>() {
        return Ok(loro::ContainerID::new_root(value.to_str()?, ty.into()));
    }
    if let Ok(value) = obj.downcast::<ContainerID>() {
        return Ok(loro::ContainerID::from(value.get()));
    }

    Err(PyTypeError::new_err("Invalid ContainerID"))
}

pub fn pyobject_to_loro_value(obj: &Bound<'_, PyAny>) -> PyResult<loro::LoroValue> {
    if obj.is_none() {
        return Ok(loro::LoroValue::Null);
    }

    if let Ok(value) = obj.extract::<bool>() {
        return Ok(loro::LoroValue::Bool(value));
    }
    if let Ok(value) = obj.extract::<i64>() {
        return Ok(loro::LoroValue::I64(value));
    }
    if let Ok(value) = obj.extract::<f64>() {
        return Ok(loro::LoroValue::Double(value));
    }
    if let Ok(value) = obj.downcast::<PyBytes>() {
        return Ok(loro::LoroValue::Binary(loro::LoroBinaryValue::from(
            value.as_bytes().to_vec(),
        )));
    }
    if let Ok(value) = obj.downcast::<PyString>() {
        return Ok(loro::LoroValue::String(loro::LoroStringValue::from(
            value.to_string(),
        )));
    }
    if let Ok(value) = obj.downcast::<PyList>() {
        let mut list = Vec::with_capacity(value.len());
        for item in value.iter() {
            list.push(pyobject_to_loro_value(&item)?);
        }
        return Ok(loro::LoroValue::List(loro::LoroListValue::from(list)));
    }
    if let Ok(value) = obj.downcast::<PyDict>() {
        let mut map = FxHashMap::default();
        for (key, value) in value.iter() {
            if key.downcast::<PyString>().is_ok() {
                map.insert(key.to_string(), pyobject_to_loro_value(&value)?);
            } else {
                return Err(PyTypeError::new_err(
                    "only dict with string keys is supported for converting to LoroValue",
                ));
            }
        }
        return Ok(loro::LoroValue::Map(loro::LoroMapValue::from(map)));
    }
    if let Ok(value) = obj.downcast::<PyMapping>() {
        let mut map = FxHashMap::default();
        for key in value.keys()? {
            if key.downcast::<PyString>().is_ok() {
                map.insert(
                    key.to_string(),
                    pyobject_to_loro_value(&value.get_item(key).unwrap())?,
                );
            } else {
                return Err(PyTypeError::new_err(
                    "only dict with string keys is supported for converting to LoroValue",
                ));
            }
        }
        return Ok(loro::LoroValue::Map(loro::LoroMapValue::from(map)));
    }
    if let Ok(value) = obj.downcast::<ContainerID>() {
        return Ok(loro::LoroValue::Container(value.get().clone().into()));
    }
    Err(PyTypeError::new_err("Invalid LoroValue"))
}

pub fn loro_value_to_pyobject(py: Python, value: LoroValue) -> PyResult<Bound<'_, PyAny>> {
    match value.0 {
        loro::LoroValue::Null => Ok(py.None().into_pyobject(py)?.into_any().into_bound()),
        loro::LoroValue::Bool(b) => Ok(PyBool::new(py, b)
            .into_pyobject(py)?
            .into_any()
            .into_bound()),
        loro::LoroValue::Double(f) => Ok(f.into_pyobject(py)?.into_any().into_bound()),
        loro::LoroValue::I64(i) => Ok(i.into_pyobject(py)?.into_any().into_bound()),
        loro::LoroValue::String(s) => Ok(s.to_string().into_pyobject(py)?.into_any().into_bound()),
        loro::LoroValue::Binary(b) => Ok(b.as_slice().into_pyobject(py)?.into_any().into_bound()),
        loro::LoroValue::List(l) => {
            let list = l
                .iter()
                .map(|v| LoroValue(v.clone()).into_pyobject(py))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(list.into_pyobject(py)?.into_any().into_bound())
        }
        loro::LoroValue::Map(m) => {
            let dict = m
                .iter()
                .map(|(k, v)| Ok((k, LoroValue(v.clone()).into_pyobject(py)?)))
                .collect::<Result<FxHashMap<_, _>, PyErr>>()?;
            Ok(dict.into_pyobject(py)?.into_any().into_bound())
        }
        loro::LoroValue::Container(c) => Ok(ContainerID::from(&c)
            .into_pyobject(py)?
            .into_any()
            .into_bound()),
    }
}

impl From<ContainerType> for loro::ContainerType {
    fn from(value: ContainerType) -> loro::ContainerType {
        match value {
            ContainerType::Text {} => loro::ContainerType::Text,
            ContainerType::Map {} => loro::ContainerType::Map,
            ContainerType::List {} => loro::ContainerType::List,
            ContainerType::MovableList {} => loro::ContainerType::MovableList,
            ContainerType::Tree {} => loro::ContainerType::Tree,
            ContainerType::Counter {} => loro::ContainerType::Counter,
            ContainerType::Unknown { kind } => loro::ContainerType::Unknown(kind),
        }
    }
}
impl From<loro::ContainerType> for ContainerType {
    fn from(value: loro::ContainerType) -> ContainerType {
        match value {
            loro::ContainerType::Text => ContainerType::Text {},
            loro::ContainerType::Map => ContainerType::Map {},
            loro::ContainerType::List => ContainerType::List {},
            loro::ContainerType::MovableList => ContainerType::MovableList {},
            loro::ContainerType::Tree => ContainerType::Tree {},
            loro::ContainerType::Counter => ContainerType::Counter {},
            loro::ContainerType::Unknown(kind) => ContainerType::Unknown { kind },
        }
    }
}
impl From<ContainerID> for loro::ContainerID {
    fn from(value: ContainerID) -> loro::ContainerID {
        match value {
            ContainerID::Root {
                name,
                container_type,
            } => loro::ContainerID::Root {
                name: name.into(),
                container_type: container_type.into(),
            },
            ContainerID::Normal {
                peer,
                counter,
                container_type,
            } => loro::ContainerID::Normal {
                peer,
                counter,
                container_type: container_type.into(),
            },
        }
    }
}

impl From<&ContainerID> for loro::ContainerID {
    fn from(value: &ContainerID) -> loro::ContainerID {
        match value {
            ContainerID::Root {
                name,
                container_type,
            } => loro::ContainerID::Root {
                name: name.clone().into(),
                container_type: (*container_type).into(),
            },
            ContainerID::Normal {
                peer,
                counter,
                container_type,
            } => loro::ContainerID::Normal {
                peer: *peer,
                counter: *counter,
                container_type: (*container_type).into(),
            },
        }
    }
}

impl From<loro::ContainerID> for ContainerID {
    fn from(value: loro::ContainerID) -> ContainerID {
        match value {
            loro::ContainerID::Root {
                name,
                container_type,
            } => ContainerID::Root {
                name: name.to_string(),
                container_type: container_type.into(),
            },
            loro::ContainerID::Normal {
                peer,
                counter,
                container_type,
            } => ContainerID::Normal {
                peer,
                counter,
                container_type: container_type.into(),
            },
        }
    }
}

impl From<&loro::ContainerID> for ContainerID {
    fn from(value: &loro::ContainerID) -> ContainerID {
        match value {
            loro::ContainerID::Root {
                name,
                container_type,
            } => ContainerID::Root {
                name: name.to_string(),
                container_type: (*container_type).into(),
            },
            loro::ContainerID::Normal {
                peer,
                counter,
                container_type,
            } => ContainerID::Normal {
                peer: *peer,
                counter: *counter,
                container_type: (*container_type).into(),
            },
        }
    }
}

impl From<TreeID> for loro::TreeID {
    fn from(value: TreeID) -> Self {
        Self {
            peer: value.peer,
            counter: value.counter,
        }
    }
}

impl From<loro::TreeID> for TreeID {
    fn from(value: loro::TreeID) -> Self {
        Self {
            peer: value.peer,
            counter: value.counter,
        }
    }
}

impl From<TreeParentId> for loro::TreeParentId {
    fn from(value: TreeParentId) -> Self {
        match value {
            TreeParentId::Node(id) => loro::TreeParentId::Node(id.into()),
            TreeParentId::Root {} => loro::TreeParentId::Root,
            TreeParentId::Deleted {} => loro::TreeParentId::Deleted,
            TreeParentId::Unexist {} => loro::TreeParentId::Unexist,
        }
    }
}

impl From<loro::TreeParentId> for TreeParentId {
    fn from(value: loro::TreeParentId) -> Self {
        match value {
            loro::TreeParentId::Node(id) => TreeParentId::Node(id.into()),
            loro::TreeParentId::Root => TreeParentId::Root {},
            loro::TreeParentId::Deleted => TreeParentId::Deleted {},
            loro::TreeParentId::Unexist => TreeParentId::Unexist {},
        }
    }
}

impl<'a> From<&loro::event::ContainerDiff<'a>> for ContainerDiff {
    fn from(value: &loro::event::ContainerDiff<'a>) -> Self {
        Self {
            target: value.target.into(),
            path: value
                .path
                .iter()
                .map(|(id, index)| PathItem {
                    container: id.into(),
                    index: index.into(),
                })
                .collect(),
            is_unknown: value.is_unknown,
            diff: (&value.diff).into(),
        }
    }
}

impl From<&loro::Index> for Index {
    fn from(value: &loro::Index) -> Self {
        match value {
            loro::Index::Key(key) => Index::Key {
                key: key.to_string(),
            },
            loro::Index::Seq(index) => Index::Seq {
                index: *index as u32,
            },
            loro::Index::Node(target) => Index::Node {
                target: (*target).into(),
            },
        }
    }
}

impl From<Index> for loro::Index {
    fn from(value: Index) -> loro::Index {
        match value {
            Index::Key { key } => loro::Index::Key(key.into()),
            Index::Seq { index } => loro::Index::Seq(index as usize),
            Index::Node { target } => loro::Index::Node(target.into()),
        }
    }
}

impl From<&loro::event::Diff<'_>> for Diff {
    fn from(value: &loro::event::Diff) -> Self {
        match value {
            loro::event::Diff::List(l) => {
                let mut ans = Vec::with_capacity(l.len());
                for item in l.iter() {
                    match item {
                        loro::event::ListDiffItem::Insert { insert, is_move } => {
                            let mut new_insert = Vec::with_capacity(insert.len());
                            for v in insert.iter() {
                                new_insert.push(v.clone().into());
                            }
                            ans.push(ListDiffItem::Insert {
                                insert: new_insert,
                                is_move: *is_move,
                            });
                        }
                        loro::event::ListDiffItem::Delete { delete } => {
                            ans.push(ListDiffItem::Delete {
                                delete: *delete as u32,
                            });
                        }
                        loro::event::ListDiffItem::Retain { retain } => {
                            ans.push(ListDiffItem::Retain {
                                retain: *retain as u32,
                            });
                        }
                    }
                }
                Diff::List(ans)
            }
            loro::event::Diff::Text(t) => {
                Diff::Text(t.iter().map(|x| x.into()).collect::<Vec<_>>())
            }
            loro::event::Diff::Map(m) => {
                let mut updated = HashMap::new();
                for (key, value) in m.updated.iter() {
                    updated.insert(key.to_string(), value.as_ref().map(|v| v.clone().into()));
                }

                Diff::Map(MapDelta { updated })
            }
            loro::event::Diff::Tree(t) => {
                let mut diff = Vec::new();
                for item in t.iter() {
                    diff.push(TreeDiffItem {
                        target: item.target.into(),
                        action: match &item.action {
                            loro::TreeExternalDiff::Create {
                                parent,
                                index,
                                position,
                            } => TreeExternalDiff::Create {
                                parent: (*parent).into(),
                                index: *index as u32,
                                fractional_index: position.to_string(),
                            },
                            loro::TreeExternalDiff::Move {
                                parent,
                                index,
                                position,
                                old_parent,
                                old_index,
                            } => TreeExternalDiff::Move {
                                parent: (*parent).into(),
                                index: *index as u32,
                                fractional_index: position.to_string(),
                                old_parent: (*old_parent).into(),
                                old_index: *old_index as u32,
                            },
                            loro::TreeExternalDiff::Delete {
                                old_parent,
                                old_index,
                            } => TreeExternalDiff::Delete {
                                old_parent: (*old_parent).into(),
                                old_index: *old_index as u32,
                            },
                        },
                    });
                }
                Diff::Tree(TreeDiff { diff })
            }
            loro::event::Diff::Counter(c) => Diff::Counter(*c),
            loro::event::Diff::Unknown => Diff::Unknown {},
        }
    }
}
impl From<loro::EventTriggerKind> for EventTriggerKind {
    fn from(kind: loro::EventTriggerKind) -> Self {
        match kind {
            loro::EventTriggerKind::Local => Self::Local,
            loro::EventTriggerKind::Import => Self::Import,
            loro::EventTriggerKind::Checkout => Self::Checkout,
        }
    }
}

impl From<loro::ValueOrContainer> for ValueOrContainer {
    fn from(value: loro::ValueOrContainer) -> Self {
        match value {
            loro::ValueOrContainer::Value(v) => ValueOrContainer::Value(v.into()),
            loro::ValueOrContainer::Container(c) => ValueOrContainer::Container(c.into()),
        }
    }
}

impl From<loro::Container> for Container {
    fn from(value: loro::Container) -> Self {
        match value {
            loro::Container::List(c) => Container::List(LoroList(c)),
            loro::Container::Map(c) => Container::Map(LoroMap(c)),
            loro::Container::MovableList(c) => Container::MovableList(LoroMovableList(c)),
            loro::Container::Text(c) => Container::Text(LoroText(c)),
            loro::Container::Tree(c) => Container::Tree(LoroTree(c)),
            loro::Container::Counter(c) => Container::Counter(LoroCounter(c)),
            loro::Container::Unknown(c) => Container::Unknown(LoroUnknown(c)),
        }
    }
}
impl From<Container> for loro::Container {
    fn from(value: Container) -> Self {
        match value {
            Container::List(c) => loro::Container::List(c.0),
            Container::Map(c) => loro::Container::Map(c.0),
            Container::MovableList(c) => loro::Container::MovableList(c.0),
            Container::Text(c) => loro::Container::Text(c.0),
            Container::Tree(c) => loro::Container::Tree(c.0),
            Container::Counter(c) => loro::Container::Counter(c.0),
            Container::Unknown(c) => loro::Container::Unknown(c.0),
        }
    }
}

impl From<&Index> for loro::Index {
    fn from(value: &Index) -> Self {
        match value {
            Index::Key { key } => loro::Index::Key(key.clone().into()),
            Index::Seq { index } => loro::Index::Seq(*index as usize),
            Index::Node { target } => loro::Index::Node((*target).into()),
        }
    }
}

impl From<UpdateOptions> for loro::UpdateOptions {
    fn from(value: UpdateOptions) -> Self {
        Self {
            timeout_ms: value.timeout_ms,
            use_refined_diff: value.use_refined_diff,
        }
    }
}

impl From<&TextDelta> for loro::TextDelta {
    fn from(value: &TextDelta) -> Self {
        match value {
            TextDelta::Retain { retain, attributes } => loro::TextDelta::Retain {
                retain: *retain,
                attributes: attributes
                    .as_ref()
                    .map(|a| a.iter().map(|(k, v)| (k.to_string(), v.into())).collect()),
            },
            TextDelta::Insert { insert, attributes } => loro::TextDelta::Insert {
                insert: insert.to_string(),
                attributes: attributes
                    .as_ref()
                    .map(|a| a.iter().map(|(k, v)| (k.to_string(), v.into())).collect()),
            },
            TextDelta::Delete { delete } => loro::TextDelta::Delete { delete: *delete },
        }
    }
}

impl From<&loro::TextDelta> for TextDelta {
    fn from(value: &loro::TextDelta) -> Self {
        match value {
            loro::TextDelta::Retain { retain, attributes } => TextDelta::Retain {
                retain: *retain,
                attributes: attributes.as_ref().map(|a| {
                    a.iter()
                        .map(|(k, v)| (k.to_string(), v.clone().into()))
                        .collect()
                }),
            },
            loro::TextDelta::Insert { insert, attributes } => TextDelta::Insert {
                insert: insert.to_string(),
                attributes: attributes.as_ref().map(|a| {
                    a.iter()
                        .map(|(k, v)| (k.to_string(), v.clone().into()))
                        .collect()
                }),
            },
            loro::TextDelta::Delete { delete } => TextDelta::Delete { delete: *delete },
        }
    }
}

impl From<loro::event::DiffEvent<'_>> for DiffEvent {
    fn from(diff_event: loro::event::DiffEvent) -> Self {
        Self {
            triggered_by: diff_event.triggered_by.into(),
            origin: diff_event.origin.to_string(),
            current_target: diff_event.current_target.map(|v| v.into()),
            events: diff_event.events.iter().map(ContainerDiff::from).collect(),
        }
    }
}

impl From<Side> for loro::cursor::Side {
    fn from(value: Side) -> Self {
        match value {
            Side::Left => loro::cursor::Side::Left,
            Side::Middle => loro::cursor::Side::Middle,
            Side::Right => loro::cursor::Side::Right,
        }
    }
}

impl From<loro::cursor::Side> for Side {
    fn from(value: loro::cursor::Side) -> Self {
        match value {
            loro::cursor::Side::Left => Side::Left,
            loro::cursor::Side::Middle => Side::Middle,
            loro::cursor::Side::Right => Side::Right,
        }
    }
}

impl From<Cursor> for loro::cursor::Cursor {
    fn from(value: Cursor) -> Self {
        value.0
    }
}

impl From<loro::cursor::Cursor> for Cursor {
    fn from(value: loro::cursor::Cursor) -> Self {
        Cursor(value)
    }
}

impl From<loro::Subscription> for Subscription {
    fn from(value: loro::Subscription) -> Self {
        Subscription(Mutex::new(Some(value)))
    }
}

impl From<loro::cursor::PosQueryResult> for PosQueryResult {
    fn from(value: loro::cursor::PosQueryResult) -> Self {
        Self {
            update: value.update.map(|c| c.into()),
            current: AbsolutePosition {
                pos: value.current.pos,
                side: value.current.side.into(),
            },
        }
    }
}

impl From<IdSpan> for loro::IdSpan {
    fn from(value: IdSpan) -> Self {
        loro::IdSpan {
            peer: value.peer,
            counter: value.counter.into(),
        }
    }
}

impl From<CounterSpan> for loro::CounterSpan {
    fn from(value: CounterSpan) -> Self {
        loro::CounterSpan {
            start: value.start,
            end: value.end,
        }
    }
}

impl From<ExportMode> for loro::ExportMode<'_> {
    fn from(value: ExportMode) -> Self {
        match value {
            ExportMode::Snapshot => loro::ExportMode::Snapshot,
            ExportMode::Updates { from } => loro::ExportMode::Updates {
                from: Cow::Owned(from.into()),
            },
            ExportMode::UpdatesInRange { spans } => loro::ExportMode::UpdatesInRange {
                spans: Cow::Owned(spans.into_iter().map(|s| s.into()).collect()),
            },
            ExportMode::ShallowSnapshot { frontiers } => {
                loro::ExportMode::ShallowSnapshot(Cow::Owned(frontiers.into()))
            }
            ExportMode::StateOnly { frontiers } => {
                loro::ExportMode::StateOnly(frontiers.map(|f| Cow::Owned(f.into())))
            }
            ExportMode::SnapshotAt { version } => loro::ExportMode::SnapshotAt {
                version: Cow::Owned(version.into()),
            },
        }
    }
}

impl From<loro::ChangeMeta> for ChangeMeta {
    fn from(value: loro::ChangeMeta) -> Self {
        ChangeMeta {
            lamport: value.lamport,
            id: value.id.into(),
            timestamp: value.timestamp,
            message: value.message.map(|m| m.to_string()),
            deps: value.deps.into(),
            len: value.len,
        }
    }
}

impl From<loro::ImportBlobMetadata> for ImportBlobMetadata {
    fn from(value: loro::ImportBlobMetadata) -> Self {
        Self {
            partial_start_vv: value.partial_start_vv.into(),
            partial_end_vv: value.partial_end_vv.into(),
            start_timestamp: value.start_timestamp,
            start_frontiers: value.start_frontiers.into(),
            end_timestamp: value.end_timestamp,
            change_num: value.change_num,
            mode: match value.mode {
                loro_internal::encoding::EncodedBlobMode::Snapshot => EncodedBlobMode::Snapshot,
                loro_internal::encoding::EncodedBlobMode::OutdatedSnapshot => {
                    EncodedBlobMode::OutdatedSnapshot
                }
                loro_internal::encoding::EncodedBlobMode::ShallowSnapshot => {
                    EncodedBlobMode::ShallowSnapshot
                }
                loro_internal::encoding::EncodedBlobMode::OutdatedRle => {
                    EncodedBlobMode::OutdatedRle
                }
                loro_internal::encoding::EncodedBlobMode::Updates => EncodedBlobMode::Updates,
            },
        }
    }
}

impl From<loro::TreeNode> for TreeNode {
    fn from(node: loro::TreeNode) -> Self {
        Self {
            id: node.id.into(),
            parent: node.parent.into(),
            fractional_index: node.fractional_index.to_string(),
            index: node.index,
        }
    }
}
