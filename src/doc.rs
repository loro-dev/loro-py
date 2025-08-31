use loro::{Counter, Lamport, LoroDoc as LoroDocInner, PeerID, Timestamp};
use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyBytes, PyType},
};
use std::{borrow::Cow, collections::HashSet, fmt::Display, ops::ControlFlow, sync::Arc};

use crate::{
    container::{
        Cursor, LoroCounter, LoroList, LoroMap, LoroMovableList, LoroText, LoroTree, Side,
    },
    convert::pyobject_to_container_id,
    err::{PyLoroError, PyLoroResult},
    event::{DiffBatch, DiffEvent, Index, Subscription},
    value::{ContainerID, ContainerType, LoroValue, Ordering, ValueOrContainer, ID},
    version::{Frontiers, VersionRange, VersionVector, VersionVectorDiff},
};

pub fn register_class(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<LoroDoc>()?;
    m.add_class::<Configure>()?;
    m.add_class::<ImportStatus>()?;
    m.add_class::<PosQueryResult>()?;
    m.add_class::<EncodedBlobMode>()?;
    m.add_class::<ImportBlobMetadata>()?;
    m.add_class::<StyleConfigMap>()?;
    m.add_class::<ExportMode>()?;
    m.add_class::<IdSpan>()?;
    m.add_class::<CounterSpan>()?;
    m.add_class::<ExpandType>()?;
    m.add_class::<ChangeMeta>()?;
    m.add_class::<ChangeModifier>()?;
    Ok(())
}

/// `LoroDoc` is the entry for the whole document.
/// When it's dropped, all the associated [`Handler`]s will be invalidated.
///
/// **Important:** Loro is a pure library and does not handle network protocols.
/// It is the responsibility of the user to manage the storage, loading, and synchronization
/// of the bytes exported by Loro in a manner suitable for their specific environment.
#[pyclass(frozen)]
pub struct LoroDoc {
    pub(crate) doc: LoroDocInner,
}

impl Default for LoroDoc {
    fn default() -> Self {
        let doc = LoroDocInner::new();
        Self { doc }
    }
}

#[pymethods]
impl LoroDoc {
    /// Create a new `LoroDoc` instance.
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    /// Duplicate the document with a different PeerID
    ///
    /// The time complexity and space complexity of this operation are both O(n),
    ///
    /// When called in detached mode, it will fork at the current state frontiers.
    /// It will have the same effect as `fork_at(&self.state_frontiers())`.
    #[inline]
    pub fn fork(&self) -> Self {
        let doc = self.doc.fork();
        Self { doc }
    }

    /// Fork the document at the given frontiers.
    ///
    /// The created doc will only contain the history before the specified frontiers.
    pub fn fork_at(&self, frontiers: &Frontiers) -> Self {
        let new_doc = self.doc.fork_at(&frontiers.into());
        Self { doc: new_doc }
    }

    /// Get the configurations of the document.
    #[inline]
    #[getter]
    pub fn config(&self) -> Configure {
        self.doc.config().clone().into()
    }

    /// Get `Change` at the given id.
    ///
    /// `Change` is a grouped continuous operations that share the same id, timestamp, commit message.
    ///
    /// - The id of the `Change` is the id of its first op.
    /// - The second op's id is `{ peer: change.id.peer, counter: change.id.counter + 1 }`
    ///
    /// The same applies on `Lamport`:
    ///
    /// - The lamport of the `Change` is the lamport of its first op.
    /// - The second op's lamport is `change.lamport + 1`
    ///
    /// The length of the `Change` is how many operations it contains
    pub fn get_change(&self, id: ID) -> Option<ChangeMeta> {
        self.doc.get_change(id.into()).map(|meta| meta.into())
    }

    /// Decodes the metadata for an imported blob from the provided bytes.
    #[classmethod]
    pub fn decode_import_blob_meta(
        _cls: &Bound<'_, PyType>,
        bytes: Bound<'_, PyBytes>,
        check_checksum: bool,
    ) -> PyLoroResult<ImportBlobMetadata> {
        let meta = LoroDocInner::decode_import_blob_meta(bytes.as_bytes(), check_checksum)?;
        Ok(meta.into())
    }

    /// Set whether to record the timestamp of each change. Default is `false`.
    ///
    /// If enabled, the Unix timestamp will be recorded for each change automatically.
    ///
    /// You can set each timestamp manually when committing a change.
    ///
    /// NOTE: Timestamps are forced to be in ascending order.
    /// If you commit a new change with a timestamp that is less than the existing one,
    /// the largest existing timestamp will be used instead.
    pub fn set_record_timestamp(&self, record: bool) {
        self.doc.set_record_timestamp(record);
    }

    /// Enables editing in detached mode, which is disabled by default.
    ///
    /// The doc enter detached mode after calling `detach` or checking out a non-latest version.
    ///
    /// # Important Notes:
    ///
    /// - This mode uses a different PeerID for each checkout.
    /// - Ensure no concurrent operations share the same PeerID if set manually.
    /// - Importing does not affect the document's state or version; changes are
    ///   recorded in the [OpLog] only. Call `checkout` to apply changes.
    #[inline]
    pub fn set_detached_editing(&self, enable: bool) {
        self.doc.set_detached_editing(enable);
    }

    /// Whether editing the doc in detached mode is allowed, which is disabled by
    /// default.
    ///
    /// The doc enter detached mode after calling `detach` or checking out a non-latest version.
    ///
    /// # Important Notes:
    ///
    /// - This mode uses a different PeerID for each checkout.
    /// - Ensure no concurrent operations share the same PeerID if set manually.
    /// - Importing does not affect the document's state or version; changes are
    ///   recorded in the [OpLog] only. Call `checkout` to apply changes.
    #[inline]
    #[getter]
    pub fn is_detached_editing_enabled(&self) -> bool {
        self.doc.is_detached_editing_enabled()
    }

    /// Set the interval of mergeable changes, **in seconds**.
    ///
    /// If two continuous local changes are within the interval, they will be merged into one change.
    /// The default value is 1000 seconds.
    ///
    /// By default, we record timestamps in seconds for each change. So if the merge interval is 1, and changes A and B
    /// have timestamps of 3 and 4 respectively, then they will be merged into one change
    #[inline]
    pub fn set_change_merge_interval(&self, interval: i64) {
        self.doc.set_change_merge_interval(interval);
    }

    /// Set the rich text format configuration of the document.
    ///
    /// You need to config it if you use rich text `mark` method.
    /// Specifically, you need to config the `expand` property of each style.
    ///
    /// Expand is used to specify the behavior of expanding when new text is inserted at the
    /// beginning or end of the style.
    #[inline]
    pub fn config_text_style(&self, text_style: StyleConfigMap) {
        self.doc.config_text_style(text_style.0)
    }

    /// Configures the default text style for the document.
    ///
    /// This method sets the default text style configuration for the document when using LoroText.
    /// If `None` is provided, the default style is reset.
    ///
    /// # Parameters
    ///
    /// - `text_style`: The style configuration to set as the default. `None` to reset.
    #[pyo3(signature = (text_style=None))]
    pub fn config_default_text_style(&self, text_style: Option<ExpandType>) {
        self.doc
            .config_default_text_style(text_style.map(|c| loro::StyleConfig { expand: c.into() }));
    }

    /// Attach the document state to the latest known version.
    ///
    /// > The document becomes detached during a `checkout` operation.
    /// > Being `detached` implies that the `DocState` is not synchronized with the latest version of the `OpLog`.
    /// > In a detached state, the document is not editable, and any `import` operations will be
    /// > recorded in the `OpLog` without being applied to the `DocState`.
    #[inline]
    pub fn attach(&self) {
        self.doc.attach()
    }

    /// Checkout the `DocState` to a specific version.
    ///
    /// The document becomes detached during a `checkout` operation.
    /// Being `detached` implies that the `DocState` is not synchronized with the latest version of the `OpLog`.
    /// In a detached state, the document is not editable, and any `import` operations will be
    /// recorded in the `OpLog` without being applied to the `DocState`.
    ///
    /// You should call `attach` to attach the `DocState` to the latest version of `OpLog`.
    #[inline]
    pub fn checkout(&self, frontiers: &Frontiers) -> PyLoroResult<()> {
        self.doc.checkout(&frontiers.into())?;
        Ok(())
    }

    /// Checkout the `DocState` to the latest version.
    ///
    /// > The document becomes detached during a `checkout` operation.
    /// > Being `detached` implies that the `DocState` is not synchronized with the latest version of the `OpLog`.
    /// > In a detached state, the document is not editable, and any `import` operations will be
    /// > recorded in the `OpLog` without being applied to the `DocState`.
    ///
    /// This has the same effect as `attach`.
    #[inline]
    pub fn checkout_to_latest(&self) {
        self.doc.checkout_to_latest()
    }

    /// Compare the frontiers with the current OpLog's version.
    ///
    /// If `other` contains any version that's not contained in the current OpLog, return [Ordering::Less].
    #[inline]
    pub fn cmp_with_frontiers(&self, other: &Frontiers) -> Ordering {
        self.doc.cmp_with_frontiers(&other.into()).into()
    }

    /// Compare two frontiers.
    ///
    /// If the frontiers are not included in the document, return [`FrontiersNotIncluded`].
    #[inline]
    pub fn cmp_frontiers(&self, a: &Frontiers, b: &Frontiers) -> PyResult<Option<Ordering>> {
        let ans = self
            .doc
            .cmp_frontiers(&a.into(), &b.into())
            .map_err(|e| PyValueError::new_err(e.to_string()))?
            .map(|o| o.into());
        Ok(ans)
    }

    /// Force the document enter the detached mode.
    ///
    /// In this mode, when you importing new updates, the [loro_internal::DocState] will not be changed.
    ///
    /// Learn more at https://loro.dev/docs/advanced/doc_state_and_oplog#attacheddetached-status
    #[inline]
    pub fn detach(&self) {
        self.doc.detach()
    }

    // /// Import a batch of updates/snapshot.
    // ///
    // /// The data can be in arbitrary order. The import result will be the same.
    #[inline]
    pub fn import_batch(&self, bytes: Vec<Bound<'_, PyBytes>>) -> PyLoroResult<ImportStatus> {
        let vec_bytes: Vec<Vec<u8>> = bytes.into_iter().map(|b| b.as_bytes().to_vec()).collect();
        let status = self.doc.import_batch(&vec_bytes)?;
        Ok(ImportStatus::from(status))
    }

    /// Get a [LoroMovableList] by container id.
    ///
    /// If the provided id is string, it will be converted into a root container id with the name of the string.
    #[inline]
    pub fn get_movable_list(&self, obj: &Bound<'_, PyAny>) -> PyResult<LoroMovableList> {
        let container_id = pyobject_to_container_id(obj, ContainerType::MovableList {})?;
        Ok(LoroMovableList(self.doc.get_movable_list(container_id)))
    }

    /// Get a [LoroList] by container id.
    ///
    /// If the provided id is string, it will be converted into a root container id with the name of the string.
    #[inline]
    pub fn get_list(&self, obj: &Bound<'_, PyAny>) -> PyResult<LoroList> {
        let container_id = pyobject_to_container_id(obj, ContainerType::List {})?;
        Ok(LoroList(self.doc.get_list(container_id)))
    }

    /// Get a [LoroMap] by container id.
    ///
    /// If the provided id is string, it will be converted into a root container id with the name of the string.
    #[inline]
    pub fn get_map(&self, obj: &Bound<'_, PyAny>) -> PyResult<LoroMap> {
        let container_id = pyobject_to_container_id(obj, ContainerType::Map {})?;
        Ok(LoroMap(self.doc.get_map(container_id)))
    }

    /// Get a [LoroText] by container id.
    ///
    /// If the provided id is string, it will be converted into a root container id with the name of the string.
    #[inline]
    pub fn get_text(&self, obj: &Bound<'_, PyAny>) -> PyResult<LoroText> {
        let container_id = pyobject_to_container_id(obj, ContainerType::Text {})?;
        Ok(LoroText(self.doc.get_text(container_id)))
    }

    /// Get a [LoroTree] by container id.
    ///
    /// If the provided id is string, it will be converted into a root container id with the name of the string.
    #[inline]
    pub fn get_tree(&self, obj: &Bound<'_, PyAny>) -> PyResult<LoroTree> {
        let container_id = pyobject_to_container_id(obj, ContainerType::Tree {})?;
        Ok(LoroTree(self.doc.get_tree(container_id)))
    }

    /// Get a [LoroCounter] by container id.
    ///
    /// If the provided id is string, it will be converted into a root container id with the name of the string.
    #[inline]
    pub fn get_counter(&self, obj: &Bound<'_, PyAny>) -> PyResult<LoroCounter> {
        let container_id = pyobject_to_container_id(obj, ContainerType::Counter {})?;
        Ok(LoroCounter(self.doc.get_counter(container_id)))
    }

    /// Commit the cumulative auto commit transaction.
    ///
    /// There is a transaction behind every operation.
    /// The events will be emitted after a transaction is committed. A transaction is committed when:
    ///
    /// - `doc.commit()` is called.
    /// - `doc.export(mode)` is called.
    /// - `doc.import(data)` is called.
    /// - `doc.checkout(version)` is called.
    #[inline]
    pub fn commit(&self) {
        self.doc.commit()
    }

    /// Commit the cumulative auto commit transaction with custom configure.
    ///
    /// There is a transaction behind every operation.
    /// It will automatically commit when users invoke export or import.
    /// The event will be sent after a transaction is committed
    #[pyo3(signature = (origin=None, timestamp=None, immediate_renew=true, commit_msg=None))]
    #[inline]
    pub fn commit_with(
        &self,
        origin: Option<&str>,
        timestamp: Option<i64>,
        immediate_renew: Option<bool>,
        commit_msg: Option<&str>,
    ) {
        self.doc.commit_with(loro::CommitOptions {
            origin: origin.map(|s| s.into()),
            immediate_renew: immediate_renew.unwrap_or(true),
            timestamp,
            commit_msg: commit_msg.map(|s| s.into()),
        })
    }

    /// Set commit message for the current uncommitted changes
    ///
    /// It will be persisted.
    pub fn set_next_commit_message(&self, msg: &str) {
        self.doc.set_next_commit_message(msg)
    }

    /// Set `origin` for the current uncommitted changes, it can be used to track the source of changes in an event.
    ///
    /// It will NOT be persisted.
    pub fn set_next_commit_origin(&self, origin: &str) {
        self.doc.set_next_commit_origin(origin)
    }

    /// Set the timestamp of the next commit.
    ///
    /// It will be persisted and stored in the `OpLog`.
    /// You can get the timestamp from the [`Change`] type.
    pub fn set_next_commit_timestamp(&self, timestamp: i64) {
        self.doc.set_next_commit_timestamp(timestamp)
    }

    /// Set the options of the next commit.
    ///
    /// It will be used when the next commit is performed.
    #[pyo3(signature = (origin=None, timestamp=None, immediate_renew=true, commit_msg=None))]
    pub fn set_next_commit_options(
        &self,
        origin: Option<&str>,
        timestamp: Option<i64>,
        immediate_renew: Option<bool>,
        commit_msg: Option<&str>,
    ) {
        self.doc.set_next_commit_options(loro::CommitOptions {
            origin: origin.map(|s| s.into()),
            immediate_renew: immediate_renew.unwrap_or(true),
            timestamp,
            commit_msg: commit_msg.map(|s| s.into()),
        })
    }

    /// Clear the options of the next commit.
    pub fn clear_next_commit_options(&self) {
        self.doc.clear_next_commit_options()
    }

    /// Whether the document is in detached mode, where the [loro_internal::DocState] is not
    /// synchronized with the latest version of the [loro_internal::OpLog].
    #[inline]
    pub fn is_detached(&self) -> bool {
        self.doc.is_detached()
    }

    /// Import updates/snapshot exported by [`LoroDoc::export_snapshot`] or [`LoroDoc::export_from`].
    #[pyo3(name = "import_")]
    #[inline]
    pub fn import(&self, bytes: Bound<'_, PyBytes>) -> PyLoroResult<ImportStatus> {
        let status = self.doc.import(bytes.as_bytes())?;
        Ok(ImportStatus::from(status))
    }

    /// Import updates/snapshot exported by [`LoroDoc::export_snapshot`] or [`LoroDoc::export_from`].
    ///
    /// It marks the import with a custom `origin` string. It can be used to track the import source
    /// in the generated events.
    #[inline]
    pub fn import_with(
        &self,
        bytes: Bound<'_, PyBytes>,
        origin: &str,
    ) -> PyLoroResult<ImportStatus> {
        let status = self.doc.import_with(bytes.as_bytes(), origin)?;
        Ok(ImportStatus::from(status))
    }

    /// Import the json schema updates.
    ///
    /// only supports backward compatibility but not forward compatibility.
    #[inline]
    pub fn import_json_updates(&self, json: String) -> PyLoroResult<ImportStatus> {
        let status = self.doc.import_json_updates(json)?;
        Ok(ImportStatus::from(status))
    }

    // TODO: return an object
    /// Export the current state with json-string format of the document.
    #[inline]
    pub fn export_json_updates(&self, start_vv: VersionVector, end_vv: VersionVector) -> String {
        let json = self
            .doc
            .export_json_updates(&start_vv.into(), &end_vv.into());
        serde_json::to_string(&json).unwrap()
    }

    /// Exports changes within the specified ID span to JSON schema format.
    ///
    /// The JSON schema format produced by this method is identical to the one generated by `export_json_updates`.
    /// It ensures deterministic output, making it ideal for hash calculations and integrity checks.
    ///
    /// This method can also export pending changes from the uncommitted transaction that have not yet been applied to the OpLog.
    ///
    /// This method will NOT trigger a new commit implicitly.
    pub fn export_json_in_id_span(&self, id_span: IdSpan) -> String {
        let json = self.doc.export_json_in_id_span(id_span.into());
        serde_json::to_string(&json).unwrap()
    }

    /// Convert `Frontiers` into `VersionVector`
    #[inline]
    pub fn frontiers_to_vv(&self, frontiers: &Frontiers) -> Option<VersionVector> {
        self.doc
            .frontiers_to_vv(&frontiers.into())
            .map(|vv| vv.into())
    }

    // /// Minimize the frontiers by removing the unnecessary entries.
    // pub fn minimize_frontiers(&self, frontiers: &Frontiers) -> Result<Frontiers, ID> {
    //     self.with_oplog(|oplog| shrink_frontiers(frontiers, oplog.dag()))
    // }

    /// Convert `VersionVector` into `Frontiers`
    #[inline]
    pub fn vv_to_frontiers(&self, vv: VersionVector) -> Frontiers {
        self.doc.vv_to_frontiers(&vv.into()).into()
    }

    // /// Access the `OpLog`.
    // ///
    // /// NOTE: Please be ware that the API in `OpLog` is unstable
    // #[inline]
    // pub fn with_oplog<R>(&self, f: impl FnOnce(&OpLog) -> R) -> R {
    //     let oplog = self.doc.oplog().try_lock().unwrap();
    //     f(&oplog)
    // }

    // /// Access the `DocState`.
    // ///
    // /// NOTE: Please be ware that the API in `DocState` is unstable
    // #[inline]
    // pub fn with_state<R>(&self, f: impl FnOnce(&mut DocState) -> R) -> R {
    //     let mut state = self.doc.app_state().try_lock().unwrap();
    //     f(&mut state)
    // }

    /// Get the `VersionVector` version of `OpLog`
    #[inline]
    #[getter]
    pub fn oplog_vv(&self) -> VersionVector {
        self.doc.oplog_vv().into()
    }

    /// Get the `VersionVector` version of `DocState`
    #[inline]
    #[getter]
    pub fn state_vv(&self) -> VersionVector {
        self.doc.state_vv().into()
    }

    /// The doc only contains the history since this version
    ///
    /// This is empty if the doc is not shallow.
    ///
    /// The ops included by the shallow history start version vector are not in the doc.
    #[inline]
    #[getter]
    pub fn shallow_since_vv(&self) -> VersionVector {
        loro::VersionVector::from_im_vv(&self.doc.shallow_since_vv()).into()
    }

    /// The doc only contains the history since this version
    ///
    /// This is empty if the doc is not shallow.
    ///
    /// The ops included by the shallow history start frontiers are not in the doc.
    #[inline]
    #[getter]
    pub fn shallow_since_frontiers(&self) -> Frontiers {
        self.doc.shallow_since_frontiers().into()
    }

    /// Get the total number of operations in the `OpLog`
    #[inline]
    #[getter]
    pub fn len_ops(&self) -> usize {
        self.doc.len_ops()
    }

    /// Get the total number of changes in the `OpLog`
    #[inline]
    #[getter]
    pub fn len_changes(&self) -> usize {
        self.doc.len_changes()
    }

    /// Get the shallow value of the document.
    #[inline]
    pub fn get_value(&self) -> LoroValue {
        self.doc.get_value().into()
    }

    /// Get the entire state of the current DocState
    #[inline]
    pub fn get_deep_value(&self) -> LoroValue {
        self.doc.get_deep_value().into()
    }

    /// Get the entire state of the current DocState with container id
    #[inline]
    pub fn get_deep_value_with_id(&self) -> LoroValue {
        self.doc.get_deep_value_with_id().into()
    }

    /// Get the `Frontiers` version of `OpLog`
    #[getter]
    #[inline]
    pub fn oplog_frontiers(&self) -> Frontiers {
        self.doc.oplog_frontiers().into()
    }

    /// Get the `Frontiers` version of `DocState`
    ///
    /// Learn more about [`Frontiers`](https://loro.dev/docs/advanced/version_deep_dive)
    #[getter]
    #[inline]
    pub fn state_frontiers(&self) -> Frontiers {
        self.doc.state_frontiers().into()
    }

    /// Get the PeerID
    #[getter]
    #[inline]
    pub fn peer_id(&self) -> PeerID {
        self.doc.peer_id()
    }

    /// Change the PeerID
    ///
    /// NOTE: You need to make sure there is no chance two peer have the same PeerID.
    /// If it happens, the document will be corrupted.
    #[setter]
    #[inline]
    #[pyo3(name = "peer_id")]
    pub fn set_peer_id(&self, peer: PeerID) -> PyLoroResult<()> {
        self.doc.set_peer_id(peer)?;
        Ok(())
    }

    /// Subscribe the events of a container.
    ///
    /// The callback will be invoked after a transaction that change the container.
    /// Returns a subscription that can be used to unsubscribe.
    ///
    /// The events will be emitted after a transaction is committed. A transaction is committed when:
    ///
    /// - `doc.commit()` is called.
    /// - `doc.export(mode)` is called.
    /// - `doc.import(data)` is called.
    /// - `doc.checkout(version)` is called.
    ///
    /// # Example
    ///
    /// ```
    /// # use loro::LoroDoc;
    /// # use std::sync::{atomic::AtomicBool, Arc};
    /// # use loro::{event::DiffEvent, LoroResult, TextDelta};
    /// #
    /// let doc = LoroDoc::new();
    /// let text = doc.get_text("text");
    /// let ran = Arc::new(AtomicBool::new(false));
    /// let ran2 = ran.clone();
    /// let sub = doc.subscribe(
    ///     &text.id(),
    ///     Arc::new(move |event| {
    ///         assert!(event.triggered_by.is_local());
    ///         for event in event.events {
    ///             let delta = event.diff.as_text().unwrap();
    ///             let d = TextDelta::Insert {
    ///                 insert: "123".into(),
    ///                 attributes: Default::default(),
    ///             };
    ///             assert_eq!(delta, &vec![d]);
    ///             ran2.store(true, std::sync::atomic::Ordering::Relaxed);
    ///         }
    ///     }),
    /// );
    /// text.insert(0, "123").unwrap();
    /// doc.commit();
    /// assert!(ran.load(std::sync::atomic::Ordering::Relaxed));
    /// // unsubscribe
    /// sub.unsubscribe();
    /// ```
    #[inline]
    pub fn subscribe(&self, container_id: &ContainerID, callback: Py<PyAny>) -> Subscription {
        let subscription = self.doc.subscribe(
            &container_id.into(),
            Arc::new(move |e| {
                Python::attach(|py| {
                    callback.call1(py, (DiffEvent::from(e),)).unwrap();
                });
            }),
        );
        subscription.into()
    }

    /// Subscribe all the events.
    ///
    /// The callback will be invoked when any part of the [loro_internal::DocState] is changed.
    /// Returns a subscription that can be used to unsubscribe.
    ///
    /// The events will be emitted after a transaction is committed. A transaction is committed when:
    ///
    /// - `doc.commit()` is called.
    /// - `doc.export(mode)` is called.
    /// - `doc.import(data)` is called.
    /// - `doc.checkout(version)` is called.
    #[inline]
    pub fn subscribe_root(&self, callback: Py<PyAny>) -> Subscription {
        let subscription = self.doc.subscribe_root(Arc::new(move |e| {
            Python::attach(|py| {
                callback.call1(py, (DiffEvent::from(e),)).unwrap();
            });
        }));
        subscription.into()
    }

    /// Subscribe the local update of the document.
    pub fn subscribe_local_update(&self, callback: Py<PyAny>) -> Subscription {
        let subscription = self.doc.subscribe_local_update(Box::new(move |updates| {
            Python::attach(|py| {
                let b = callback.call1(py, (updates,)).unwrap();
                b.extract::<bool>(py).unwrap()
            })
        }));
        subscription.into()
    }

    /// Subscribe the peer id change of the document.
    pub fn subscribe_peer_id_change(&self, callback: Py<PyAny>) -> Subscription {
        let subscription = self.doc.subscribe_peer_id_change(Box::new(move |id| {
            Python::attach(|py| {
                let b = callback.call1(py, (ID::from(*id),)).unwrap();
                b.extract::<bool>(py).unwrap()
            })
        }));
        subscription.into()
    }

    // /// Estimate the size of the document states in memory.
    // #[inline]
    // pub fn log_estimate_size(&self) {
    //     self.doc.log_estimated_size();
    // }

    // /// Check the correctness of the document state by comparing it with the state
    // /// calculated by applying all the history.
    // #[inline]
    // pub fn check_state_correctness_slow(&self) {
    //     self.doc.check_state_diff_calc_consistency_slow()
    // }

    /// Get the handler by the path.
    #[inline]
    pub fn get_by_path(&self, path: Vec<Index>) -> Option<ValueOrContainer> {
        self.doc
            .get_by_path(&path.iter().map(|x| x.into()).collect::<Vec<_>>())
            .map(ValueOrContainer::from)
    }

    /// Get the handler by the string path.
    #[inline]
    pub fn get_by_str_path(&self, path: &str) -> Option<ValueOrContainer> {
        self.doc.get_by_str_path(path).map(ValueOrContainer::from)
    }

    /// Get the absolute position of the given cursor.
    ///
    /// # Example
    ///
    /// ```
    /// # use loro::{LoroDoc, ToJson};
    /// let doc = LoroDoc::new();
    /// let text = &doc.get_text("text");
    /// text.insert(0, "01234").unwrap();
    /// let pos = text.get_cursor(5, Default::default()).unwrap();
    /// assert_eq!(doc.get_cursor_pos(&pos).unwrap().current.pos, 5);
    /// text.insert(0, "01234").unwrap();
    /// assert_eq!(doc.get_cursor_pos(&pos).unwrap().current.pos, 10);
    /// text.delete(0, 10).unwrap();
    /// assert_eq!(doc.get_cursor_pos(&pos).unwrap().current.pos, 0);
    /// text.insert(0, "01234").unwrap();
    /// assert_eq!(doc.get_cursor_pos(&pos).unwrap().current.pos, 5);
    /// ```
    #[inline]
    pub fn get_cursor_pos(&self, cursor: Cursor) -> PyLoroResult<PosQueryResult> {
        let result = self.doc.get_cursor_pos(&cursor.into())?;
        Ok(result.into())
    }

    // /// Get the inner LoroDoc ref.
    // // #[inline]
    // // pub fn inner(&self) -> &InnerLoroDoc {
    // //     &self.doc
    // // }

    /// Whether the history cache is built.
    #[inline]
    #[getter]
    pub fn has_history_cache(&self) -> bool {
        self.doc.has_history_cache()
    }

    /// Free the history cache that is used for making checkout faster.
    ///
    /// If you use checkout that switching to an old/concurrent version, the history cache will be built.
    /// You can free it by calling this method.
    #[inline]
    pub fn free_history_cache(&self) {
        self.doc.free_history_cache()
    }

    /// Free the cached diff calculator that is used for checkout.
    #[inline]
    pub fn free_diff_calculator(&self) {
        self.doc.free_diff_calculator()
    }

    /// Encoded all ops and history cache to bytes and store them in the kv store.
    ///
    /// This will free up the memory that used by parsed ops
    #[inline]
    pub fn compact_change_store(&self) {
        self.doc.compact_change_store()
    }

    /// Export the document in the given mode.
    pub fn export(&self, mode: ExportMode) -> PyLoroResult<Cow<'_, [u8]>> {
        let ans = self.doc.export(mode.into())?;
        Ok(Cow::Owned(ans))
    }

    // /// Analyze the container info of the doc
    // ///
    // /// This is used for development and debugging. It can be slow.
    // // TODO:
    // // pub fn analyze(&self) -> DocAnalysis {
    // //     self.doc.analyze()
    // // }

    /// Get the path from the root to the container
    pub fn get_path_to_container(&self, id: &ContainerID) -> Option<Vec<(ContainerID, Index)>> {
        self.doc.get_path_to_container(&id.into()).map(|v| {
            v.into_iter()
                .map(|(id, index)| (ContainerID::from(id), (&index).into()))
                .collect()
        })
    }

    /// Evaluate a JSONPath expression on the document and return matching values or handlers.
    ///
    /// This method allows querying the document structure using JSONPath syntax.
    /// It returns a vector of `ValueOrHandler` which can represent either primitive values
    /// or container handlers, depending on what the JSONPath expression matches.
    ///
    /// # Arguments
    ///
    /// * `path` - A string slice containing the JSONPath expression to evaluate.
    ///
    /// # Returns
    ///
    /// A `Result` containing either:
    /// - `Ok(Vec<ValueOrHandler>)`: A vector of matching values or handlers.
    /// - `Err(String)`: An error message if the JSONPath expression is invalid or evaluation fails.
    ///
    /// # Example
    ///
    /// ```
    /// # use loro::{LoroDoc, ToJson};
    ///
    /// let doc = LoroDoc::new();
    /// let map = doc.get_map("users");
    /// map.insert("alice", 30).unwrap();
    /// map.insert("bob", 25).unwrap();
    ///
    /// let result = doc.jsonpath("$.users.alice").unwrap();
    /// assert_eq!(result.len(), 1);
    /// assert_eq!(result[0].as_value().unwrap().to_json_value(), serde_json::json!(30));
    /// ```
    #[inline]
    pub fn jsonpath(&self, path: &str) -> PyResult<Vec<ValueOrContainer>> {
        self.doc
            .jsonpath(path)
            .map(|vec| vec.into_iter().map(|v| v.into()).collect())
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// Get the number of operations in the pending transaction.
    ///
    /// The pending transaction is the one that is not committed yet. It will be committed
    /// after calling `doc.commit()`, `doc.export(mode)` or `doc.checkout(version)`.
    pub fn get_pending_txn_len(&self) -> usize {
        self.doc.get_pending_txn_len()
    }

    /// Traverses the ancestors of the Change containing the given ID, including itself.
    ///
    /// This method visits all ancestors in causal order, from the latest to the oldest,
    /// based on their Lamport timestamps.
    ///
    /// # Arguments
    ///
    /// * `ids` - The IDs of the Change to start the traversal from.
    /// * `cb` - A callback function that is called for each ancestor. It can return `True` to stop the traversal.
    pub fn travel_change_ancestors(&self, ids: Vec<ID>, cb: Py<PyAny>) -> PyLoroResult<()> {
        self.doc.travel_change_ancestors(
            &ids.into_iter().map(|id| id.into()).collect::<Vec<_>>(),
            &mut |meta| {
                let b = Python::attach(|py| {
                    cb.call1(py, (ChangeMeta::from(meta),))
                        .unwrap()
                        .extract::<bool>(py)
                        .unwrap()
                });
                if b {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            },
        )?;
        Ok(())
    }

    /// Check if the doc contains the full history.
    pub fn is_shallow(&self) -> bool {
        self.doc.is_shallow()
    }

    /// Gets container IDs modified in the given ID range.
    ///
    /// **NOTE:** This method will implicitly commit.
    ///
    /// This method can be used in conjunction with `doc.travel_change_ancestors()` to traverse
    /// the history and identify all changes that affected specific containers.
    ///
    /// # Arguments
    ///
    /// * `id` - The starting ID of the change range
    /// * `len` - The length of the change range to check
    pub fn get_changed_containers_in(&self, id: ID, len: usize) -> HashSet<ContainerID> {
        self.doc
            .get_changed_containers_in(id.into(), len)
            .into_iter()
            .map(ContainerID::from)
            .collect()
    }

    /// Revert the current document state back to the target version
    ///
    /// Internally, it will generate a series of local operations that can revert the
    /// current doc to the target version. It will calculate the diff between the current
    /// state and the target state, and apply the diff to the current state.
    #[inline]
    pub fn revert_to(&self, version: &Frontiers) -> PyLoroResult<()> {
        self.doc.revert_to(&version.into())?;
        Ok(())
    }

    /// Apply a diff to the current document state.
    ///
    /// Internally, it will apply the diff to the current state.
    #[inline]
    pub fn apply_diff(&self, diff: DiffBatch) -> PyLoroResult<()> {
        self.doc.apply_diff(diff.into())?;
        Ok(())
    }

    /// Calculate the diff between two versions
    #[inline]
    pub fn diff(&self, a: &Frontiers, b: &Frontiers) -> PyLoroResult<DiffBatch> {
        let ans = self.doc.diff(&a.into(), &b.into())?;
        Ok(ans.into())
    }

    /// Check if the doc contains the target container.
    ///
    /// A root container always exists, while a normal container exists
    /// if it has ever been created on the doc.
    pub fn has_container(&self, id: &ContainerID) -> bool {
        self.doc.has_container(&id.into())
    }

    /// Find the operation id spans that between the `from` version and the `to` version.
    pub fn find_id_spans_between(&self, from: &Frontiers, to: &Frontiers) -> VersionVectorDiff {
        self.doc
            .find_id_spans_between(&from.into(), &to.into())
            .into()
    }

    /// Subscribe to the first commit from a peer. Operations performed on the `LoroDoc` within this callback
    /// will be merged into the current commit.
    ///
    /// This is useful for managing the relationship between `PeerID` and user information.
    /// For example, you could store user names in a `LoroMap` using `PeerID` as the key and the `UserID` as the value.
    pub fn subscribe_first_commit_from_peer(&self, callback: Py<PyAny>) -> Subscription {
        let subscription = self
            .doc
            .subscribe_first_commit_from_peer(Box::new(move |payload| {
                Python::attach(|py| {
                    let b = callback
                        .call1(py, (FirstCommitFromPeerPayload { peer: payload.peer },))
                        .unwrap();
                    b.extract::<bool>(py).unwrap()
                })
            }));
        subscription.into()
    }

    /// Subscribe to the pre-commit event.
    ///
    /// The callback will be called when the changes are committed but not yet applied to the OpLog.
    /// You can modify the commit message and timestamp in the callback by [`ChangeModifier`].
    pub fn subscribe_pre_commit(&self, callback: Py<PyAny>) -> Subscription {
        let subscription = self.doc.subscribe_pre_commit(Box::new(move |payload| {
            Python::attach(|py| {
                let b = callback
                    .call1(
                        py,
                        (PreCommitCallbackPayload {
                            change_meta: payload.change_meta.clone().into(),
                            origin: payload.origin.clone(),
                            modifier: ChangeModifier(payload.modifier.clone()),
                        },),
                    )
                    .unwrap();
                b.extract::<bool>(py).unwrap()
            })
        }));
        subscription.into()
    }

    /// Set whether to hide empty root containers.
    ///
    /// # Example
    /// ```
    /// use loro::LoroDoc;
    ///
    /// let doc = LoroDoc::new();
    /// let map = doc.get_map("map");
    /// dbg!(doc.get_deep_value()); // {"map": {}}
    /// doc.set_hide_empty_root_containers(true);
    /// dbg!(doc.get_deep_value()); // {}
    /// ```
    pub fn set_hide_empty_root_containers(&self, hide: bool) {
        self.doc.set_hide_empty_root_containers(hide);
    }

    /// Delete all content from a root container and hide it from the document.
    ///
    /// When a root container is empty and hidden:
    /// - It won't show up in `get_deep_value()` results
    /// - It won't be included in document snapshots
    ///
    /// Only works on root containers (containers without parents).
    pub fn delete_root_container(&self, cid: ContainerID) {
        self.doc.delete_root_container(cid.into());
    }

    /// Redacts sensitive content in JSON updates within the specified version range.
    ///
    /// This function allows you to share document history while removing potentially sensitive content.
    /// It preserves the document structure and collaboration capabilities while replacing content with
    /// placeholders according to these redaction rules:
    ///
    /// - Preserves delete and move operations
    /// - Replaces text insertion content with the Unicode replacement character
    /// - Substitutes list and map insert values with null
    /// - Maintains structure of child containers
    /// - Replaces text mark values with null
    /// - Preserves map keys and text annotation keys
    pub fn redact_json_updates(
        &self,
        json: &str,
        version_range: VersionRange,
    ) -> PyLoroResult<String> {
        let mut schema =
            serde_json::from_str(json).map_err(|_e| PyLoroError::Error(_e.to_string()))?;
        loro::json::redact(&mut schema, version_range.into())
            .map_err(|e| PyLoroError::Error(e.to_string()))?;
        Ok(serde_json::to_string(&schema).unwrap())
    }
}

#[derive(Debug, IntoPyObject)]
pub struct FirstCommitFromPeerPayload {
    pub peer: PeerID,
}

#[derive(Debug, IntoPyObject)]
pub struct PreCommitCallbackPayload {
    pub change_meta: ChangeMeta,
    pub origin: String,
    pub modifier: ChangeModifier,
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct ChangeModifier(loro::ChangeModifier);

#[pymethods]
impl ChangeModifier {
    pub fn set_message(&self, msg: &str) {
        self.0.set_message(msg);
    }

    pub fn set_timestamp(&self, timestamp: i64) {
        self.0.set_timestamp(timestamp);
    }
}

impl Display for PreCommitCallbackPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[pyclass(frozen)]
pub struct Configure(pub loro::Configure);

#[pymethods]
impl Configure {
    #[new]
    pub fn default() -> Self {
        Self(loro::Configure::default())
    }

    pub fn text_style_config(&self) -> StyleConfigMap {
        StyleConfigMap(self.0.text_style_config().read().unwrap().clone())
    }

    pub fn record_timestamp(&self) -> bool {
        self.0.record_timestamp()
    }

    pub fn set_record_timestamp(&self, record: bool) {
        self.0.set_record_timestamp(record);
    }

    pub fn detached_editing(&self) -> bool {
        self.0.detached_editing()
    }

    pub fn set_detached_editing(&self, mode: bool) {
        self.0.set_detached_editing(mode);
    }

    pub fn merge_interval(&self) -> i64 {
        self.0.merge_interval()
    }

    pub fn set_merge_interval(&self, interval: i64) {
        self.0.set_merge_interval(interval);
    }
}

#[pyclass(get_all, set_all, str)]
#[derive(Debug)]
pub struct ImportStatus {
    pub success: VersionRange,
    pub pending: Option<VersionRange>,
}

impl From<loro::ImportStatus> for ImportStatus {
    fn from(value: loro::ImportStatus) -> Self {
        let a = value.success;
        Self {
            success: a.into(),
            pending: value.pending.map(|x| x.into()),
        }
    }
}

impl Display for ImportStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[pyclass(get_all, str)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PosQueryResult {
    pub update: Option<Cursor>,
    pub current: AbsolutePosition,
}

impl Display for PosQueryResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[pyclass(get_all, str)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbsolutePosition {
    pub pos: usize,
    /// The target position is at the left, middle, or right of the given pos.
    pub side: Side,
}

impl Display for AbsolutePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub enum ExportMode {
    Snapshot {},
    Updates { from_: VersionVector },
    UpdatesInRange { spans: Vec<IdSpan> },
    ShallowSnapshot { frontiers: Frontiers },
    StateOnly { frontiers: Option<Frontiers> },
    SnapshotAt { version: Frontiers },
}

/// This struct supports reverse repr: [CounterSpan]'s from can be less than to. But we should use it conservatively.
/// We need this because it'll make merging deletions easier.
#[pyclass(get_all, str)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdSpan {
    pub peer: PeerID,
    pub counter: CounterSpan,
}

impl Display for IdSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[pymethods]
impl IdSpan {
    #[new]
    pub fn new(peer: PeerID, counter: CounterSpan) -> Self {
        Self { peer, counter }
    }
}

/// This struct supports reverse repr: `from` can be less than `to`.
/// We need this because it'll make merging deletions easier.
///
/// But we should use it behavior conservatively.
/// If it is not necessary to be reverse, it should not.
#[pyclass(get_all, str)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CounterSpan {
    pub start: Counter,
    pub end: Counter,
}

#[pymethods]
impl CounterSpan {
    #[new]
    pub fn new(start: Counter, end: Counter) -> Self {
        Self { start, end }
    }
}

impl Display for CounterSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[pyclass(get_all, str)]
#[derive(Debug, Clone)]
pub struct ChangeMeta {
    /// Lamport timestamp of the Change
    pub lamport: Lamport,
    /// The first Op id of the Change
    pub id: ID,
    /// [Unix time](https://en.wikipedia.org/wiki/Unix_time)
    /// It is the number of seconds that have elapsed since 00:00:00 UTC on 1 January 1970.
    pub timestamp: Timestamp,
    /// The commit message of the change
    pub message: Option<String>,
    /// The dependencies of the first op of the change
    pub deps: Frontiers,
    /// The total op num inside this change
    pub len: usize,
}

impl Display for ChangeMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[pyclass(get_all, str)]
#[derive(Debug, Clone)]
pub struct ImportBlobMetadata {
    /// The partial start version vector.
    ///
    /// Import blob includes all the ops from `partial_start_vv` to `partial_end_vv`.
    /// However, it does not constitute a complete version vector, as it only contains counters
    /// from peers included within the import blob.
    pub partial_start_vv: VersionVector,
    /// The partial end version vector.
    ///
    /// Import blob includes all the ops from `partial_start_vv` to `partial_end_vv`.
    /// However, it does not constitute a complete version vector, as it only contains counters
    /// from peers included within the import blob.
    pub partial_end_vv: VersionVector,
    pub start_timestamp: i64,
    pub start_frontiers: Frontiers,
    pub end_timestamp: i64,
    pub change_num: u32,
    pub mode: EncodedBlobMode,
}

impl Display for ImportBlobMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EncodedBlobMode {
    Snapshot,
    OutdatedSnapshot,
    ShallowSnapshot,
    OutdatedRle,
    Updates,
}

#[pyclass(str)]
#[derive(Debug, Clone, Default)]
pub struct StyleConfigMap(loro::StyleConfigMap);

#[pymethods]
impl StyleConfigMap {
    #[new]
    pub fn new() -> Self {
        Self(loro::StyleConfigMap::new())
    }

    pub fn insert(&mut self, key: String, value: ExpandType) {
        if key.contains(':') {
            panic!("style key should not contain ':'");
        }

        self.0.insert(
            key.into(),
            loro::StyleConfig {
                expand: value.into(),
            },
        );
    }

    pub fn get(&self, key: &str) -> Option<ExpandType> {
        self.0.get(&key.into()).map(|x| x.expand.into())
    }

    #[classmethod]
    pub fn default_rich_text_config(_cls: &Bound<'_, PyType>) -> Self {
        Self(loro::StyleConfigMap::default_rich_text_config())
    }
}

impl Display for StyleConfigMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Whether to expand the style when inserting new text around it.
///
/// - Before: when inserting new text before this style, the new text should inherit this style.
/// - After: when inserting new text after this style, the new text should inherit this style.
/// - Both: when inserting new text before or after this style, the new text should inherit this style.
/// - Null: when inserting new text before or after this style, the new text should **not** inherit this style.
#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub enum ExpandType {
    Before,
    After,
    Both,
    Null,
}
