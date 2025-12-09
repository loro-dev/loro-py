import typing
from enum import Enum

LORO_VERSION: str
LoroValue = typing.Union[
    None,
    bool,
    float,
    int,
    bytes,
    str,
    typing.List["LoroValue"],
    typing.Dict[str, "LoroValue"],
    ContainerID,
]

Container = typing.Union[
    LoroList, LoroMap, LoroMovableList, LoroText, LoroTree, LoroCounter, LoroUnknown
]

ContainerId = typing.Union[str, ContainerID]

class AbsolutePosition:
    pos: int
    side: Side

class Awareness:
    all_states: dict[int, PeerInfo]
    peer: int

    def __new__(cls, peer: int, timeout: int): ...
    def encode(self, peers: typing.Sequence[int]) -> bytes: ...
    def encode_all(self) -> bytes: ...
    def apply(self, encoded_peers_info: bytes) -> AwarenessPeerUpdate: ...
    @property
    def local_state(self) -> typing.Optional[LoroValue]: ...
    @local_state.setter
    def local_state(self, value: LoroValue) -> None: ...
    def remove_outdated(self) -> list[int]: ...

class AwarenessPeerUpdate:
    updated: list[int]
    added: list[int]

class ChangeMeta:
    lamport: int
    id: ID
    timestamp: int
    message: typing.Optional[str]
    deps: Frontiers
    len: int

class Configure:
    def __new__(cls) -> Configure: ...
    
    def text_style_config(self) -> StyleConfigMap:
        """
        Get the text style configuration.
        """
        ...
    
    def record_timestamp(self) -> bool:
        """
        Get whether to record timestamp for changes.
        """
        ...
    
    def set_record_timestamp(self, record: bool) -> None:
        """
        Set whether to record timestamp for changes.
        """
        ...
    
    def detached_editing(self) -> bool:
        """
        Get whether detached editing mode is enabled.
        """
        ...
    
    def set_detached_editing(self, mode: bool) -> None:
        """
        Set whether to enable detached editing mode.
        """
        ...
    
    def merge_interval(self) -> int:
        """
        Get the merge interval in milliseconds.
        """
        ...
    
    def set_merge_interval(self, interval: int) -> None:
        """
        Set the merge interval in milliseconds.
        """
        ...

class ContainerDiff:
    r"""
    A diff of a container.
    """

    target: ContainerID
    path: list[PathItem]
    is_unknown: bool
    diff: Diff

class CounterSpan:
    r"""
    This struct supports reverse repr: `from` can be less than `to`.
    We need this because it'll make merging deletions easier.

    But we should use it behavior conservatively.
    If it is not necessary to be reverse, it should not.
    """

    start: int
    end: int
    def __new__(cls, start: int, end: int): ...

class Cursor:
    id: typing.Optional[ID]
    side: Side
    container: ContainerID
    
    def encode(self) -> bytes:
        r"""
        Encode the cursor to bytes.
        """
        ...
    
    @classmethod
    def decode(cls, bytes: bytes) -> Cursor:
        r"""
        Decode the cursor from bytes.
        """
        ...

class DiffEvent:
    triggered_by: EventTriggerKind
    origin: str
    current_target: typing.Optional[ContainerID]
    events: list[ContainerDiff]

class Frontiers:
    def __new__(
        cls,
    ): ...
    @classmethod
    def from_id(cls, id: ID) -> Frontiers: ...
    @classmethod
    def from_ids(cls, ids: typing.Sequence[ID]) -> Frontiers: ...
    def encode(self) -> bytes: ...
    @classmethod
    def decode(cls, bytes: bytes) -> Frontiers: ...

class ID:
    peer: int
    counter: int
    def __new__(cls, peer: int, counter: int): ...

class IdSpan:
    r"""
    This struct supports reverse repr: [CounterSpan]'s from can be less than to. But we should use it conservatively.
    We need this because it'll make merging deletions easier.
    """

    peer: int
    counter: CounterSpan
    def __new__(cls, peer: int, counter: CounterSpan): ...

class ImportBlobMetadata:
    partial_start_vv: VersionVector
    partial_end_vv: VersionVector
    start_timestamp: int
    start_frontiers: Frontiers
    end_timestamp: int
    change_num: int
    mode: EncodedBlobMode

class ImportStatus:
    success: VersionRange
    pending: typing.Optional[VersionRange]

class LoroCounter:
    id: ContainerID
    value: float
    def __float__(self) -> float: ...
    def __int__(self) -> int: ...
    def __add__(self, other: typing.Any) -> float: ...
    def __radd__(self, other: typing.Any) -> float: ...
    def __sub__(self, other: typing.Any) -> float: ...
    def __rsub__(self, other: typing.Any) -> float: ...
    def __neg__(self) -> float: ...
    def __abs__(self) -> float: ...
    def __new__(
        cls,
    ): ...
    def increment(self, value: typing.Any) -> None:
        r"""
        Increment the counter by the given value.
        """
        ...

    def decrement(self, value: typing.Any) -> None:
        r"""
        Decrement the counter by the given value.
        """
        ...
    
    def subscribe(self, callback: typing.Callable[[DiffEvent], None]) -> typing.Optional[Subscription]:
        r"""
        Subscribe the events of a container.
        
        The callback will be invoked when the container is changed.
        Returns a subscription that can be used to unsubscribe.
        
        The events will be emitted after a transaction is committed. A transaction is committed when:
        
        - `doc.commit()` is called.
        - `doc.export(mode)` is called.
        - `doc.import(data)` is called.
        - `doc.checkout(version)` is called.
        """
        ...

    def convert_pos(
        self, index: int, from_: "PositionType", to: "PositionType"
    ) -> typing.Optional[int]:
        r"""
        Convert a position between coordinate systems.
        """
        ...

class LoroDoc:
    config: Configure
    is_detached_editing_enabled: bool
    oplog_vv: VersionVector
    state_vv: VersionVector
    shallow_since_vv: VersionVector
    shallow_since_frontiers: Frontiers
    len_ops: int
    len_changes: int
    oplog_frontiers: Frontiers
    state_frontiers: Frontiers
    has_history_cache: bool
    def __new__(
        cls,
    ):
        """
        `LoroDoc` is the entry for the whole document.
        When it's dropped, all the associated [`Handler`]s will be invalidated.

        **Important:** Loro is a pure library and does not handle network protocols.
        It is the responsibility of the user to manage the storage, loading, and synchronization
        of the bytes exported by Loro in a manner suitable for their specific environment.
        """
        ...

    def fork(self) -> LoroDoc:
        r"""
        Duplicate the document with a different PeerID

        The time complexity and space complexity of this operation are both O(n),

        When called in detached mode, it will fork at the current state frontiers.
        It will have the same effect as `fork_at(&self.state_frontiers())`.
        """
        ...

    def fork_at(self, frontiers: Frontiers) -> LoroDoc:
        r"""
        Fork the document at the given frontiers.

        The created doc will only contain the history before the specified frontiers.
        """
        ...

    def get_change(self, id: ID) -> typing.Optional[ChangeMeta]:
        r"""
        Get `Change` at the given id.

        `Change` is a grouped continuous operations that share the same id, timestamp, commit message.

        - The id of the `Change` is the id of its first op.
        - The second op's id is `{ peer: change.id.peer, counter: change.id.counter + 1 }`

        The same applies on `Lamport`:

        - The lamport of the `Change` is the lamport of its first op.
        - The second op's lamport is `change.lamport + 1`

        The length of the `Change` is how many operations it contains
        """
        ...

    @classmethod
    def decode_import_blob_meta(
        cls, bytes: bytes, check_checksum: bool
    ) -> ImportBlobMetadata:
        r"""
        Decodes the metadata for an imported blob from the provided bytes.
        """
        ...

    def set_record_timestamp(self, record: bool) -> None:
        r"""
        Set whether to record the timestamp of each change. Default is `false`.

        If enabled, the Unix timestamp will be recorded for each change automatically.

        You can set each timestamp manually when committing a change.

        NOTE: Timestamps are forced to be in ascending order.
        If you commit a new change with a timestamp that is less than the existing one,
        the largest existing timestamp will be used instead.
        """
        ...

    def set_detached_editing(self, enable: bool) -> None:
        r"""
        Enables editing in detached mode, which is disabled by default.

        The doc enter detached mode after calling `detach` or checking out a non-latest version.

        # Important Notes:

        - This mode uses a different PeerID for each checkout.
        - Ensure no concurrent operations share the same PeerID if set manually.
        - Importing does not affect the document's state or version; changes are
          recorded in the [OpLog] only. Call `checkout` to apply changes.
        """
        ...

    def set_change_merge_interval(self, interval: int) -> None:
        r"""
        Set the interval of mergeable changes, in seconds.

        If two continuous local changes are within the interval, they will be merged into one change.
        The default value is 1000 seconds.
        """
        ...

    def config_text_style(self, text_style: StyleConfigMap) -> None:
        r"""
        Set the rich text format configuration of the document.

        You need to config it if you use rich text `mark` method.
        Specifically, you need to config the `expand` property of each style.

        Expand is used to specify the behavior of expanding when new text is inserted at the
        beginning or end of the style.
        """
        ...

    def attach(self) -> None:
        r"""
        Attach the document state to the latest known version.

        > The document becomes detached during a `checkout` operation.
        > Being `detached` implies that the `DocState` is not synchronized with the latest version of the `OpLog`.
        > In a detached state, the document is not editable, and any `import` operations will be
        > recorded in the `OpLog` without being applied to the `DocState`.
        """
        ...

    def checkout(self, frontiers: Frontiers) -> None:
        r"""
        Checkout the `DocState` to a specific version.

        The document becomes detached during a `checkout` operation.
        Being `detached` implies that the `DocState` is not synchronized with the latest version of the `OpLog`.
        In a detached state, the document is not editable, and any `import` operations will be
        recorded in the `OpLog` without being applied to the `DocState`.

        You should call `attach` to attach the `DocState` to the latest version of `OpLog`.
        """
        ...

    def checkout_to_latest(self) -> None:
        r"""
        Checkout the `DocState` to the latest version.

        > The document becomes detached during a `checkout` operation.
        > Being `detached` implies that the `DocState` is not synchronized with the latest version of the `OpLog`.
        > In a detached state, the document is not editable, and any `import` operations will be
        > recorded in the `OpLog` without being applied to the `DocState`.

        This has the same effect as `attach`.
        """
        ...

    def cmp_with_frontiers(self, other: Frontiers) -> Ordering:
        r"""
        Compare the frontiers with the current OpLog's version.

        If `other` contains any version that's not contained in the current OpLog, return [Ordering::Less].
        """
        ...

    def cmp_frontiers(self, a: Frontiers, b: Frontiers) -> typing.Optional[Ordering]:
        r"""
        Compare two frontiers.

        If the frontiers are not included in the document, return [`FrontiersNotIncluded`].
        """
        ...

    def detach(self) -> None:
        r"""
        Force the document enter the detached mode.

        In this mode, when you importing new updates, the [loro_internal::DocState] will not be changed.

        Learn more at https://loro.dev/docs/advanced/doc_state_and_oplog#attacheddetached-status
        """
        ...

    def import_batch(self, bytes: typing.Sequence[bytes]) -> ImportStatus: ...
    def get_movable_list(self, obj: ContainerId) -> LoroMovableList:
        r"""
        Get a [LoroMovableList] by container id.

        If the provided id is string, it will be converted into a root container id with the name of the string.
        """
        ...

    def get_list(self, obj: ContainerId) -> LoroList:
        r"""
        Get a [LoroList] by container id.

        If the provided id is string, it will be converted into a root container id with the name of the string.
        """
        ...

    def get_map(self, obj: ContainerId) -> LoroMap:
        r"""
        Get a [LoroMap] by container id.

        If the provided id is string, it will be converted into a root container id with the name of the string.
        """
        ...

    def get_text(self, obj: ContainerId) -> LoroText:
        r"""
        Get a [LoroText] by container id.

        If the provided id is string, it will be converted into a root container id with the name of the string.
        """
        ...

    def get_tree(self, obj: ContainerId) -> LoroTree:
        r"""
        Get a [LoroTree] by container id.

        If the provided id is string, it will be converted into a root container id with the name of the string.
        """
        ...

    def get_counter(self, obj: typing.Any) -> LoroCounter:
        r"""
        Get a [LoroCounter] by container id.

        If the provided id is string, it will be converted into a root container id with the name of the string.
        """
        ...

    def commit(self) -> None:
        r"""
        Commit the cumulative auto commit transaction.

        There is a transaction behind every operation.
        The events will be emitted after a transaction is committed. A transaction is committed when:

        - `doc.commit()` is called.
        - `doc.export(mode)` is called.
        - `doc.import(data)` is called.
        - `doc.checkout(version)` is called.
        """
        ...

    def commit_with(
        self,
        origin: typing.Optional[str] = ...,
        timestamp: typing.Optional[int] = ...,
        immediate_renew: typing.Optional[bool] = ...,
        commit_msg: typing.Optional[str] = ...,
    ) -> None:
        r"""
        Commit the cumulative auto commit transaction with custom configure.

        There is a transaction behind every operation.
        It will automatically commit when users invoke export or import.
        The event will be sent after a transaction is committed
        """
        ...

    def set_next_commit_message(self, msg: str) -> None:
        r"""
        Set commit message for the current uncommitted changes
        """
        ...

    def is_detached(self) -> bool:
        r"""
        Whether the document is in detached mode, where the [loro_internal::DocState] is not
        synchronized with the latest version of the [loro_internal::OpLog].
        """
        ...

    def import_(self, bytes: bytes) -> ImportStatus:
        r"""
        Import updates/snapshot exported by [`LoroDoc::export_snapshot`] or [`LoroDoc::export_from`].
        """
        ...

    def import_with(self, bytes: bytes, origin: str) -> ImportStatus:
        r"""
        Import updates/snapshot exported by [`LoroDoc::export_snapshot`] or [`LoroDoc::export_from`].

        It marks the import with a custom `origin` string. It can be used to track the import source
        in the generated events.
        """
        ...

    def import_json_updates(self, json: str) -> ImportStatus:
        r"""
        Import the json schema updates.

        only supports backward compatibility but not forward compatibility.
        """
        ...

    def export_json_updates(
        self, start_vv: VersionVector, end_vv: VersionVector
    ) -> str:
        r"""
        Export the current state with json-string format of the document.
        """
        ...

    def frontiers_to_vv(self, frontiers: Frontiers) -> typing.Optional[VersionVector]:
        r"""
        Convert `Frontiers` into `VersionVector`
        """
        ...

    def vv_to_frontiers(self, vv: VersionVector) -> Frontiers:
        r"""
        Convert `VersionVector` into `Frontiers`
        """
        ...

    def get_value(self) -> LoroValue:
        r"""
        Get the shallow value of the document.
        """
        ...

    def get_deep_value(self) -> LoroValue:
        r"""
        Get the entire state of the current DocState
        """
        ...

    def get_deep_value_with_id(self) -> LoroValue:
        r"""
        Get the entire state of the current DocState with container id
        """
        ...

    @property
    def peer_id(self) -> int: ...
    @peer_id.setter
    def peer_id(self, peer: int) -> None:
        r"""
        Change the PeerID

        NOTE: You need to make sure there is no chance two peer have the same PeerID.
        If it happens, the document will be corrupted.
        """
        ...

    def subscribe(
        self, container_id: ContainerID, callback: typing.Callable[[DiffEvent], None]
    ) -> Subscription:
        r"""
        Subscribe the events of a container.

        The callback will be invoked after a transaction that change the container.
        Returns a subscription that can be used to unsubscribe.

        The events will be emitted after a transaction is committed. A transaction is committed when:

        - `doc.commit()` is called.
        - `doc.export(mode)` is called.
        - `doc.import(data)` is called.
        - `doc.checkout(version)` is called.

        # Example

        ```
        # use loro::LoroDoc;
        # use std::sync::{atomic::AtomicBool, Arc};
        # use loro::{event::DiffEvent, LoroResult, TextDelta};
        #
        let doc = LoroDoc::new();
        let text = doc.get_text("text");
        let ran = Arc::new(AtomicBool::new(false));
        let ran2 = ran.clone();
        let sub = doc.subscribe(
            &text.id(),
            Arc::new(move |event| {
                assert!(event.triggered_by.is_local());
                for event in event.events {
                    let delta = event.diff.as_text().unwrap();
                    let d = TextDelta::Insert {
                        insert: "123".into(),
                        attributes: Default::default(),
                    };
                    assert_eq!(delta, &vec![d]);
                    ran2.store(true, std::sync::atomic::Ordering::Relaxed);
                }
            }),
        );
        text.insert(0, "123").unwrap();
        doc.commit();
        assert!(ran.load(std::sync::atomic::Ordering::Relaxed));
        // unsubscribe
        sub.unsubscribe();
        ```
        """
        ...

    def subscribe_root(
        self, callback: typing.Callable[[DiffEvent], None]
    ) -> Subscription:
        r"""
        Subscribe all the events.

        The callback will be invoked when any part of the [loro_internal::DocState] is changed.
        Returns a subscription that can be used to unsubscribe.

        The events will be emitted after a transaction is committed. A transaction is committed when:

        - `doc.commit()` is called.
        - `doc.export(mode)` is called.
        - `doc.import(data)` is called.
        - `doc.checkout(version)` is called.
        """
        ...

    def subscribe_local_update(
        self, callback: typing.Callable[[bytes], bool]
    ) -> Subscription:
        r"""
        Subscribe the local update of the document.
        """
        ...

    def subscribe_peer_id_change(
        self, callback: typing.Callable[[ID], bool]
    ) -> Subscription:
        r"""
        Subscribe the peer id change of the document.
        """
        ...

    def get_by_path(
        self, path: typing.Sequence[Index]
    ) -> typing.Optional[ValueOrContainer]:
        r"""
        Get the handler by the path.
        """
        ...

    def get_by_str_path(self, path: str) -> typing.Optional[ValueOrContainer]:
        r"""
        Get the handler by the string path.
        """
        ...

    def get_container(self, id: ContainerID) -> typing.Optional[Container]:
        r"""
        Get a container by its ID.
        """
        ...

    def get_cursor_pos(self, cursor: Cursor) -> PosQueryResult:
        r"""
        Get the absolute position of the given cursor.

        # Example

        ```
        # use loro::{LoroDoc, ToJson};
        let doc = LoroDoc::new();
        let text = &doc.get_text("text");
        text.insert(0, "01234").unwrap();
        let pos = text.get_cursor(5, Default::default()).unwrap();
        assert_eq!(doc.get_cursor_pos(&pos).unwrap().current.pos, 5);
        text.insert(0, "01234").unwrap();
        assert_eq!(doc.get_cursor_pos(&pos).unwrap().current.pos, 10);
        text.delete(0, 10).unwrap();
        assert_eq!(doc.get_cursor_pos(&pos).unwrap().current.pos, 0);
        text.insert(0, "01234").unwrap();
        assert_eq!(doc.get_cursor_pos(&pos).unwrap().current.pos, 5);
        ```
        """
        ...

    def free_history_cache(self) -> None:
        r"""
        Free the history cache that is used for making checkout faster.

        If you use checkout that switching to an old/concurrent version, the history cache will be built.
        You can free it by calling this method.
        """
        ...

    def free_diff_calculator(self) -> None:
        r"""
        Free the cached diff calculator that is used for checkout.
        """
        ...

    def compact_change_store(self) -> None:
        r"""
        Encoded all ops and history cache to bytes and store them in the kv store.

        This will free up the memory that used by parsed ops
        """
        ...

    def export(self, mode: ExportMode) -> bytes:
        r"""
        Export the document in the given mode.
        """
        ...

    def get_path_to_container(
        self, id: ContainerID
    ) -> typing.Optional[list[tuple[ContainerID, Index]]]:
        r"""
        Get the path from the root to the container
        """
        ...

    def jsonpath(self, path: str) -> list[ValueOrContainer]:
        r"""
        Evaluate a JSONPath expression on the document and return matching values or handlers.

        This method allows querying the document structure using JSONPath syntax.
        It returns a vector of `ValueOrHandler` which can represent either primitive values
        or container handlers, depending on what the JSONPath expression matches.

        # Arguments

        * `path` - A string slice containing the JSONPath expression to evaluate.

        # Returns

        A `Result` containing either:
        - `Ok(Vec<ValueOrHandler>)`: A vector of matching values or handlers.
        - `Err(String)`: An error message if the JSONPath expression is invalid or evaluation fails.

        # Example

        ```
        # use loro::{LoroDoc, ToJson};

        let doc = LoroDoc::new();
        let map = doc.get_map("users");
        map.insert("alice", 30).unwrap();
        map.insert("bob", 25).unwrap();

        let result = doc.jsonpath("$.users.alice").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].as_value().unwrap().to_json_value(), serde_json::json!(30));
        ```
        """
        ...

    def subscribe_jsonpath(
        self, path: str, callback: typing.Callable[[], None]
    ) -> Subscription:
        r"""
        Subscribe to updates that might affect the given JSONPath query.

        The callback may fire with false positives; use it as a lightweight signal before running
        JSONPath yourself.
        """
        ...

    def get_pending_txn_len(self) -> int:
        r"""
        Get the number of operations in the pending transaction.

        The pending transaction is the one that is not committed yet. It will be committed
        after calling `doc.commit()`, `doc.export(mode)` or `doc.checkout(version)`.
        """
        ...

    def travel_change_ancestors(
        self, ids: typing.Sequence[ID], cb: typing.Callable[[ChangeMeta], bool]
    ) -> None:
        r"""
        Traverses the ancestors of the Change containing the given ID, including itself.

        This method visits all ancestors in causal order, from the latest to the oldest,
        based on their Lamport timestamps.

        # Arguments

        * `ids` - The IDs of the Change to start the traversal from.
        * `cb` - A callback function that is called for each ancestor. It can return `True` to stop the traversal.
        """
        ...

    def is_shallow(self) -> bool:
        r"""
        Check if the doc contains the full history.
        """
        ...

    def get_changed_containers_in(self, id: ID, len: int) -> set[ContainerID]:
        r"""
        Gets container IDs modified in the given ID range.

        **NOTE:** This method will implicitly commit.

        This method can be used in conjunction with `doc.travel_change_ancestors()` to traverse
        the history and identify all changes that affected specific containers.

        # Arguments

        * `id` - The starting ID of the change range
        * `len` - The length of the change range to check
        """
        ...

    def revert_to(self, version: Frontiers) -> None:
        r"""
        Revert the current document state back to the target version

        This will generate a series of local operations that can revert the
        current doc to the target version. It will calculate the diff between the current
        state and the target state, and apply the diff to the current state.
        """
        ...

    def apply_diff(self, diff: DiffBatch) -> None:
        r"""
        Apply a diff to the current document state.

        This will apply the diff to the current state.
        """
        ...

    def diff(self, a: Frontiers, b: Frontiers) -> DiffBatch:
        r"""
        Calculate the diff between two versions
        """
        ...

    def config_default_text_style(self, text_style: typing.Optional[ExpandType] = None) -> None:
        r"""
        Configures the default text style for the document.

        This method sets the default text style configuration for the document when using LoroText.
        If `None` is provided, the default style is reset.

        # Parameters

        - `text_style`: The style configuration to set as the default. `None` to reset.
        """
        ...

    def set_next_commit_origin(self, origin: str) -> None:
        r"""
        Set `origin` for the current uncommitted changes, it can be used to track the source of changes in an event.

        It will NOT be persisted.
        """
        ...

    def set_next_commit_timestamp(self, timestamp: int) -> None:
        r"""
        Set the timestamp of the next commit.

        It will be persisted and stored in the `OpLog`.
        You can get the timestamp from the [`Change`] type.
        """
        ...
    
    def set_hide_empty_root_containers(self, hide: bool) -> None:
        r"""
        Set whether to hide empty root containers.

        # Example
        ```
        use loro::LoroDoc;

        let doc = LoroDoc::new();
        let map = doc.get_map("map");
        dbg!(doc.get_deep_value()); // {"map": {}}
        doc.set_hide_empty_root_containers(true);
        dbg!(doc.get_deep_value()); // {}
        ```
        """
        ...
    
    def delete_root_container(self, cid: ContainerID) -> None:
        r"""
        Delete all content from a root container and hide it from the document.

        When a root container is empty and hidden:
        - It won't show up in `get_deep_value()` results
        - It won't be included in document snapshots

        Only works on root containers (containers without parents).
        """
        ...
    
    def redact_json_updates(self, json: str, version_range: VersionRange) -> str:
        r"""
        Redacts sensitive content in JSON updates within the specified version range.

        This function allows you to share document history while removing potentially sensitive content.
        It preserves the document structure and collaboration capabilities while replacing content with
        placeholders according to these redaction rules:

        - Preserves delete and move operations
        - Replaces text insertion content with the Unicode replacement character
        - Substitutes list and map insert values with null
        - Maintains structure of child containers
        - Replaces text mark values with null
        - Preserves map keys and text annotation keys
        """
        ...

    def set_next_commit_options(
        self,
        origin: typing.Optional[str] = None,
        timestamp: typing.Optional[int] = None,
        immediate_renew: typing.Optional[bool] = True,
        commit_msg: typing.Optional[str] = None,
    ) -> None:
        r"""
        Set the options of the next commit.

        It will be used when the next commit is performed.
        """
        ...

    def clear_next_commit_options(self) -> None:
        r"""
        Clear the options of the next commit.
        """
        ...

    def has_container(self, id: ContainerID) -> bool:
        r"""
        Check if the doc contains the target container.

        A root container always exists, while a normal container exists
        if it has ever been created on the doc.
        """
        ...

    def export_json_in_id_span(self, id_span: IdSpan) -> str:
        r"""
        Exports changes within the specified ID span to JSON schema format.

        The JSON schema format produced by this method is identical to the one generated by `export_json_updates`.
        It ensures deterministic output, making it ideal for hash calculations and integrity checks.

        This method can also export pending changes from the uncommitted transaction that have not yet been applied to the OpLog.

        This method will NOT trigger a new commit implicitly.
        """
        ...

    def find_id_spans_between(self, from_: Frontiers, to: Frontiers) -> VersionVectorDiff:
        r"""
        Find the operation id spans that between the `from` version and the `to` version.
        """
        ...

    def subscribe_first_commit_from_peer(
        self, callback: typing.Callable[[FirstCommitFromPeerPayload], bool]
    ) -> Subscription:
        r"""
        Subscribe to the first commit from a peer. Operations performed on the `LoroDoc` within this callback
        will be merged into the current commit.

        This is useful for managing the relationship between `PeerID` and user information.
        For example, you could store user names in a `LoroMap` using `PeerID` as the key and the `UserID` as the value.
        """
        ...

    def subscribe_pre_commit(
        self, callback: typing.Callable[[PreCommitCallbackPayload], bool]
    ) -> Subscription:
        r"""
        Subscribe to the pre-commit event.

        The callback will be called when the changes are committed but not yet applied to the OpLog.
        You can modify the commit message and timestamp in the callback by [`ChangeModifier`].
        """
        ...

class LoroList:
    is_attached: bool
    id: ContainerID
    def __new__(
        cls,
    ): ...
    def insert(self, pos: int, v: LoroValue) -> None:
        r"""
        Insert a value at the given position.
        """
        ...

    def delete(self, pos: int, len: int) -> None:
        r"""
        Delete values at the given position.
        """
        ...

    def get(self, index: int) -> typing.Optional[ValueOrContainer]:
        r"""
        Get the value at the given position.
        """
        ...

    def get_deep_value(self) -> LoroValue:
        r"""
        Get the deep value of the container.
        """
        ...

    def get_value(self) -> LoroValue:
        r"""
        Get the shallow value of the container.

        This does not convert the state of sub-containers; instead, it represents them as [LoroValue::Container].
        """
        ...

    def pop(self) -> typing.Optional[LoroValue]:
        r"""
        Pop the last element of the list.
        """
        ...

    def push(self, v: LoroValue) -> None:
        r"""
        Push a value to the list.
        """
        ...

    def push_container(self, child: Container) -> Container:
        r"""
        Push a container to the list.
        """
        ...

    def for_each(self, f: typing.Callable[[ValueOrContainer], None]) -> None:
        r"""
        Iterate over the elements of the list.
        """
        ...

    def __len__(self) -> int:
        r"""
        Get the length of the list.
        """
        ...

    def is_empty(self) -> bool:
        r"""
        Whether the list is empty.
        """
        ...

    @typing.overload
    def __getitem__(self, index: int) -> ValueOrContainer: ...

    @typing.overload
    def __getitem__(self, index: slice) -> list[ValueOrContainer]: ...

    @typing.overload
    def __setitem__(self, index: int, value: LoroValue) -> None: ...

    @typing.overload
    def __setitem__(self, index: slice, value: typing.Iterable[LoroValue]) -> None: ...

    @typing.overload
    def __delitem__(self, index: int) -> None: ...

    @typing.overload
    def __delitem__(self, index: slice) -> None: ...

    def insert_container(self, pos: int, child: Container) -> Container:
        r"""
        Insert a container with the given type at the given index.

        # Example

        ```
        # use loro::{LoroDoc, ContainerType, LoroText, ToJson};
        # use serde_json::json;
        let doc = LoroDoc::new();
        let list = doc.get_list("m");
        let text = list.insert_container(0, LoroText::new()).unwrap();
        text.insert(0, "12");
        text.insert(0, "0");
        assert_eq!(doc.get_deep_value().to_json_value(), json!({"m": ["012"]}));
        ```
        """
        ...

    def get_cursor(self, pos: int, side: Side) -> typing.Optional[Cursor]:
        r"""
        Get the cursor at the given position.

        Using "index" to denote cursor positions can be unstable, as positions may
        shift with document edits. To reliably represent a position or range within
        a document, it is more effective to leverage the unique ID of each item/character
        in a List CRDT or Text CRDT.

        Loro optimizes State metadata by not storing the IDs of deleted elements. This
        approach complicates tracking cursors since they rely on these IDs. The solution
        recalculates position by replaying relevant history to update stable positions
        accurately. To minimize the performance impact of history replay, the system
        updates cursor info to reference only the IDs of currently present elements,
        thereby reducing the need for replay.

        # Example

        ```
        use loro::LoroDoc;
        use loro_internal::cursor::Side;

        let doc = LoroDoc::new();
        let list = doc.get_list("list");
        list.insert(0, 0).unwrap();
        let cursor = list.get_cursor(0, Side::Middle).unwrap();
        assert_eq!(doc.get_cursor_pos(&cursor).unwrap().current.pos, 0);
        list.insert(0, 0).unwrap();
        assert_eq!(doc.get_cursor_pos(&cursor).unwrap().current.pos, 1);
        list.insert(0, 0).unwrap();
        list.insert(0, 0).unwrap();
        assert_eq!(doc.get_cursor_pos(&cursor).unwrap().current.pos, 3);
        list.insert(4, 0).unwrap();
        assert_eq!(doc.get_cursor_pos(&cursor).unwrap().current.pos, 3);
        ```
        """
        ...

    def to_vec(self) -> list[LoroValue]:
        r"""
        Converts the LoroList to a Vec of LoroValue.

        This method unwraps the internal Arc and clones the data if necessary,
        returning a Vec containing all the elements of the LoroList as LoroValue.

        # Returns

        A Vec<LoroValue> containing all elements of the LoroList.

        # Example

        ```
        use loro::{LoroDoc, LoroValue};

        let doc = LoroDoc::new();
        let list = doc.get_list("my_list");
        list.insert(0, 1).unwrap();
        list.insert(1, "hello").unwrap();
        list.insert(2, true).unwrap();

        let vec = list.to_vec();
        ```
        """
        ...

    def clear(self) -> None:
        r"""
        Delete all elements in the list.
        """
        ...

    def get_id_at(self, pos: int) -> typing.Optional[ID]:
        r"""
        Get the ID of the list item at the given position.
        """
        ...

    def doc(self) -> typing.Optional[LoroDoc]:
        r"""
        Get the LoroDoc of the container.
        """
        ...
    
    def subscribe(self, callback: typing.Callable[[DiffEvent], None]) -> typing.Optional[Subscription]:
        r"""
        Subscribe the events of a container.
        
        The callback will be invoked when the container is changed.
        Returns a subscription that can be used to unsubscribe.
        
        The events will be emitted after a transaction is committed. A transaction is committed when:
        
        - `doc.commit()` is called.
        - `doc.export(mode)` is called.
        - `doc.import(data)` is called.
        - `doc.checkout(version)` is called.
        """
        ...

class LoroMap:
    is_attached: bool
    id: ContainerID
    def __new__(
        cls,
    ): ...
    def delete(self, key: str) -> None:
        r"""
        Delete a key-value pair from the map.
        """
        ...

    def insert(self, key: str, value: LoroValue) -> None:
        r"""
        Insert a key-value pair into the map.
        """
        ...

    def __len__(self) -> int:
        r"""
        Get the length of the map.
        """
        ...

    def __contains__(self, key: str) -> bool: ...

    @typing.overload
    def __getitem__(self, key: str) -> ValueOrContainer: ...

    def __setitem__(self, key: str, value: LoroValue) -> None: ...

    def __delitem__(self, key: str) -> None: ...

    def is_empty(self) -> bool:
        r"""
        Whether the map is empty.
        """
        ...

    def get(self, key: str) -> typing.Optional[ValueOrContainer]:
        r"""
        Get the value of the map with the given key.
        """
        ...

    def insert_container(self, key: str, child: Container) -> Container:
        r"""
        Insert a container with the given type at the given key.

        # Example

        ```
        # use loro::{LoroDoc, LoroText, ContainerType, ToJson};
        # use serde_json::json;
        let doc = LoroDoc::new();
        let map = doc.get_map("m");
        let text = map.insert_container("t", LoroText::new()).unwrap();
        text.insert(0, "12");
        text.insert(0, "0");
        assert_eq!(doc.get_deep_value().to_json_value(), json!({"m": {"t": "012"}}));
        ```
        """
        ...

    def get_value(self) -> LoroValue:
        r"""
        Get the shallow value of the map.

        It will not convert the state of sub-containers, but represent them as [LoroValue::Container].
        """
        ...

    def get_deep_value(self) -> LoroValue:
        r"""
        Get the deep value of the map.

        It will convert the state of sub-containers into a nested JSON value.
        """
        ...

    def get_or_create_container(self, key: str, child: Container) -> Container:
        r"""
        Get or create a container with the given key.
        """
        ...

    def clear(self) -> None:
        r"""
        Delete all key-value pairs in the map.
        """
        ...

    def keys(self) -> list[str]:
        r"""
        Get the keys of the map.
        """
        ...

    def values(self) -> list[ValueOrContainer]:
        r"""
        Get the values of the map.
        """
        ...

    def items(self) -> list[tuple[str, ValueOrContainer]]: ...

    def get_last_editor(self, key: str) -> typing.Optional[int]:
        r"""
        Get the peer id of the last editor on the given entry
        """
        ...

    def doc(self) -> typing.Optional[LoroDoc]:
        r"""
        Get the LoroDoc of the container.
        """
        ...
    
    def subscribe(self, callback: typing.Callable[[DiffEvent], None]) -> typing.Optional[Subscription]:
        r"""
        Subscribe the events of a container.
        
        The callback will be invoked when the container is changed.
        Returns a subscription that can be used to unsubscribe.
        
        The events will be emitted after a transaction is committed. A transaction is committed when:
        
        - `doc.commit()` is called.
        - `doc.export(mode)` is called.
        - `doc.import(data)` is called.
        - `doc.checkout(version)` is called.
        """
        ...

class LoroMovableList:
    id: ContainerID
    is_attached: bool
    def __new__(
        cls,
    ): ...
    def insert(self, pos: int, v: LoroValue) -> None:
        r"""
        Insert a value at the given position.
        """
        ...

    def delete(self, pos: int, len: int) -> None:
        r"""
        Delete the value at the given position.
        """
        ...

    def get(self, index: int) -> typing.Optional[ValueOrContainer]:
        r"""
        Get the value at the given position.
        """
        ...

    def __len__(self) -> int:
        r"""
        Get the length of the list.
        """
        ...

    def is_empty(self) -> bool:
        r"""
        Whether the list is empty.
        """
        ...

    def get_value(self) -> LoroValue:
        r"""
        Get the shallow value of the list.

        It will not convert the state of sub-containers, but represent them as [LoroValue::Container].
        """
        ...

    def get_deep_value(self) -> LoroValue:
        r"""
        Get the deep value of the list.

        It will convert the state of sub-containers into a nested JSON value.
        """
        ...

    def pop(self) -> typing.Optional[ValueOrContainer]:
        r"""
        Pop the last element of the list.
        """
        ...

    def push(self, v: LoroValue) -> None:
        r"""
        Push a value to the end of the list.
        """
        ...

    def push_container(self, child: Container) -> Container:
        r"""
        Push a container to the end of the list.
        """
        ...

    def set(self, pos: int, value: LoroValue) -> None:
        r"""
        Set the value at the given position.
        """
        ...

    def mov(self, from_: int, to: int) -> None:
        r"""
        Move the value at the given position to the given position.
        """
        ...

    def insert_container(self, pos: int, child: Container) -> Container:
        r"""
        Insert a container at the given position.
        """
        ...

    def set_container(self, pos: int, child: Container) -> Container:
        r"""
        Set the container at the given position.
        """
        ...

    def get_cursor(self, pos: int, side: Side) -> typing.Optional[Cursor]:
        r"""
        Get the cursor at the given position.

        Using "index" to denote cursor positions can be unstable, as positions may
        shift with document edits. To reliably represent a position or range within
        a document, it is more effective to leverage the unique ID of each item/character
        in a List CRDT or Text CRDT.

        Loro optimizes State metadata by not storing the IDs of deleted elements. This
        approach complicates tracking cursors since they rely on these IDs. The solution
        recalculates position by replaying relevant history to update stable positions
        accurately. To minimize the performance impact of history replay, the system
        updates cursor info to reference only the IDs of currently present elements,
        thereby reducing the need for replay.

        # Example

        ```
        use loro::LoroDoc;
        use loro_internal::cursor::Side;

        let doc = LoroDoc::new();
        let list = doc.get_movable_list("list");
        list.insert(0, 0).unwrap();
        let cursor = list.get_cursor(0, Side::Middle).unwrap();
        assert_eq!(doc.get_cursor_pos(&cursor).unwrap().current.pos, 0);
        list.insert(0, 0).unwrap();
        assert_eq!(doc.get_cursor_pos(&cursor).unwrap().current.pos, 1);
        list.insert(0, 0).unwrap();
        list.insert(0, 0).unwrap();
        assert_eq!(doc.get_cursor_pos(&cursor).unwrap().current.pos, 3);
        list.insert(4, 0).unwrap();
        assert_eq!(doc.get_cursor_pos(&cursor).unwrap().current.pos, 3);
        ```
        """
        ...

    def to_vec(self) -> list[LoroValue]:
        r"""
        Get the elements of the list as a vector of LoroValues.

        This method returns a vector containing all the elements in the list as LoroValues.
        It provides a convenient way to access the entire contents of the LoroMovableList
        as a standard Rust vector.

        # Returns

        A `Vec<LoroValue>` containing all elements of the list.

        # Example

        ```
        use loro::LoroDoc;

        let doc = LoroDoc::new();
        let list = doc.get_movable_list("mylist");
        list.insert(0, 1).unwrap();
        list.insert(1, "hello").unwrap();
        list.insert(2, true).unwrap();

        let vec = list.to_vec();
        assert_eq!(vec.len(), 3);
        assert_eq!(vec[0], 1.into());
        assert_eq!(vec[1], "hello".into());
        assert_eq!(vec[2], true.into());
        ```
        """
        ...

    def clear(self) -> None:
        r"""
        Delete all elements in the list.
        """
        ...

    def for_each(self, f: typing.Any) -> None:
        r"""
        Iterate over the elements of the list.
        """
        ...

    @typing.overload
    def __setitem__(self, index: int, value: LoroValue) -> None: ...

    @typing.overload
    def __setitem__(self, index: slice, value: typing.Iterable[LoroValue]) -> None: ...

    @typing.overload
    def __delitem__(self, index: int) -> None: ...

    @typing.overload
    def __delitem__(self, index: slice) -> None: ...

    def get_creator_at(self, pos: int) -> typing.Optional[int]:
        r"""
        Get the creator of the list item at the given position.
        """
        ...

    def get_last_mover_at(self, pos: int) -> typing.Optional[int]:
        r"""
        Get the last mover of the list item at the given position.
        """
        ...

    def get_last_editor_at(self, pos: int) -> typing.Optional[int]:
        r"""
        Get the last editor of the list item at the given position.
        """
        ...

    def doc(self) -> typing.Optional[LoroDoc]:
        r"""
        Get the LoroDoc of the container.
        """
        ...
    
    def subscribe(self, callback: typing.Callable[[DiffEvent], None]) -> typing.Optional[Subscription]:
        r"""
        Subscribe the events of a container.
        
        The callback will be invoked when the container is changed.
        Returns a subscription that can be used to unsubscribe.
        
        The events will be emitted after a transaction is committed. A transaction is committed when:
        
        - `doc.commit()` is called.
        - `doc.export(mode)` is called.
        - `doc.import(data)` is called.
        - `doc.checkout(version)` is called.
        """
        ...

class LoroText:
    is_attached: bool
    id: ContainerID
    len_utf8: int
    len_unicode: int
    len_utf16: int
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __len__(self) -> int: ...
    @typing.overload
    def __getitem__(self, index: int) -> str: ...
    @typing.overload
    def __getitem__(self, index: slice) -> str: ...
    def __contains__(self, item: typing.Any) -> bool: ...
    def __add__(self, other: typing.Union[str, "LoroText"]) -> str: ...
    def __radd__(self, other: typing.Union[str, "LoroText"]) -> str: ...
    def __new__(
        cls,
    ): ...
    def insert(self, pos: int, s: str) -> None:
        r"""
        Insert a string at the given unicode position.
        """
        ...

    def insert_utf8(self, pos: int, s: str) -> None:
        r"""
        Insert a string at the given utf-8 position.
        """
        ...

    def insert_utf16(self, pos: int, s: str) -> None:
        r"""
        Insert a string at the given utf-16 position.
        """
        ...

    def delete(self, pos: int, len: int) -> None:
        r"""
        Delete a range of text at the given unicode position with unicode length.
        """
        ...

    def delete_utf8(self, pos: int, len: int) -> None:
        r"""
        Delete a range of text at the given utf-8 position with utf-8 length.
        """
        ...

    def delete_utf16(self, pos: int, len: int) -> None:
        r"""
        Delete a range of text at the given utf-16 position with utf-16 length.
        """
        ...

    def slice(self, start_index: int, end_index: int) -> str:
        r"""
        Get a string slice at the given Unicode range
        """
        ...

    def slice_utf16(self, start_index: int, end_index: int) -> str:
        r"""
        Get a string slice at the given UTF-16 range
        """
        ...

    def slice_delta(
        self, start_index: int, end_index: int, pos_type: "PositionType"
    ) -> list[TextDelta]:
        r"""
        Get the rich-text delta within a range for the given position type.
        """
        ...

    def char_at(self, pos: int) -> str:
        r"""
        Get the characters at given unicode position.
        """
        ...

    def splice(self, pos: int, len: int, s: str) -> str:
        r"""
        Delete specified character and insert string at the same position at given unicode position.
        """
        ...

    def splice_utf16(self, pos: int, len: int, s: str) -> None:
        r"""
        Delete specified range and insert a string at the same UTF-16 position.
        """
        ...

    def is_empty(self) -> bool:
        r"""
        Whether the text container is empty.
        """
        ...

    def update(self, text: str, use_refined_diff: bool = True, timeout_ms: float | None = None) -> None:
        r"""
        Update the current text based on the provided text.

        It will calculate the minimal difference and apply it to the current text.
        It uses Myers' diff algorithm to compute the optimal difference.

        This could take a long time for large texts (e.g. > 50_000 characters).
        In that case, you should use `updateByLine` instead.
        """
        ...

    def update_by_line(self, text: str, use_refined_diff: bool = True, timeout_ms: float | None = None) -> None:
        r"""
        Update the current text based on the provided text.

        This update calculation is line-based, which will be more efficient but less precise.
        """
        ...

    def apply_delta(self, delta: typing.Sequence[TextDelta]) -> None:
        r"""
        Apply a [delta](https://quilljs.com/docs/delta/) to the text container.
        """
        ...

    def mark(self, start: int, end: int, key: str, value: LoroValue) -> None:
        r"""
        Mark a range of text with a key-value pair.

        You can use it to create a highlight, make a range of text bold, or add a link to a range of text.

        You can specify the `expand` option to set the behavior when inserting text at the boundary of the range.

        - `after`(default): when inserting text right after the given range, the mark will be expanded to include the inserted text
        - `before`: when inserting text right before the given range, the mark will be expanded to include the inserted text
        - `none`: the mark will not be expanded to include the inserted text at the boundaries
        - `both`: when inserting text either right before or right after the given range, the mark will be expanded to include the inserted text

        *You should make sure that a key is always associated with the same expand type.*

        Note: this is not suitable for unmergeable annotations like comments.
        """
        ...

    def mark_utf8(self, start: int, end: int, key: str, value: LoroValue) -> None:
        r"""
        Mark a range of text using UTF-8 offsets.
        """
        ...

    def mark_utf16(self, start: int, end: int, key: str, value: LoroValue) -> None:
        r"""
        Mark a range of text using UTF-16 offsets.
        """
        ...

    def unmark(self, start: int, end: int, key: str) -> None:
        r"""
        Unmark a range of text with a key and a value.

        You can use it to remove highlights, bolds or links

        You can specify the `expand` option to set the behavior when inserting text at the boundary of the range.

        **Note: You should specify the same expand type as when you mark the text.**

        - `after`(default): when inserting text right after the given range, the mark will be expanded to include the inserted text
        - `before`: when inserting text right before the given range, the mark will be expanded to include the inserted text
        - `none`: the mark will not be expanded to include the inserted text at the boundaries
        - `both`: when inserting text either right before or right after the given range, the mark will be expanded to include the inserted text

        *You should make sure that a key is always associated with the same expand type.*

        Note: you cannot delete unmergeable annotations like comments by this method.
        """
        ...

    def unmark_utf16(self, start: int, end: int, key: str) -> None:
        r"""
        Unmark a UTF-16 range of text with a key.
        """
        ...

    def to_delta(self) -> list[TextDelta]:
        r"""
        Get the text in [Delta](https://quilljs.com/docs/delta/) format.
        """
        ...

    def get_richtext_value(self) -> LoroValue:
        r"""
        Get the rich text value of the text container.
        """
        ...

    def to_string(self) -> str:
        r"""
        Get the text content of the text container.
        """
        ...

    def get_cursor(self, pos: int, side: Side) -> typing.Optional[Cursor]:
        r"""
        Get the cursor at the given position in the given Unicode position.

        Using "index" to denote cursor positions can be unstable, as positions may
        shift with document edits. To reliably represent a position or range within
        a document, it is more effective to leverage the unique ID of each item/character
        in a List CRDT or Text CRDT.

        Loro optimizes State metadata by not storing the IDs of deleted elements. This
        approach complicates tracking cursors since they rely on these IDs. The solution
        recalculates position by replaying relevant history to update stable positions
        accurately. To minimize the performance impact of history replay, the system
        updates cursor info to reference only the IDs of currently present elements,
        thereby reducing the need for replay.

        # Example

        ```
        # use loro::{LoroDoc, ToJson};
        let doc = LoroDoc::new();
        let text = &doc.get_text("text");
        text.insert(0, "01234").unwrap();
        let pos = text.get_cursor(5, Default::default()).unwrap();
        assert_eq!(doc.get_cursor_pos(&pos).unwrap().current.pos, 5);
        text.insert(0, "01234").unwrap();
        assert_eq!(doc.get_cursor_pos(&pos).unwrap().current.pos, 10);
        text.delete(0, 10).unwrap();
        assert_eq!(doc.get_cursor_pos(&pos).unwrap().current.pos, 0);
        text.insert(0, "01234").unwrap();
        assert_eq!(doc.get_cursor_pos(&pos).unwrap().current.pos, 5);
        ```
        """
        ...

    def is_deleted(self) -> bool:
        r"""
        Whether the text container is deleted.
        """
        ...

    def push_str(self, s: str) -> None:
        r"""
        Push a string to the end of the text container.
        """
        ...

    def get_editor_at_unicode_pos(self, pos: int) -> typing.Optional[int]:
        r"""
        Get the editor of the text at the given position.
        """
        ...

    def doc(self) -> typing.Optional[LoroDoc]:
        r"""
        Get the LoroDoc of the container.
        """
        ...
    
    def subscribe(self, callback: typing.Callable[[DiffEvent], None]) -> typing.Optional[Subscription]:
        r"""
        Subscribe the events of a container.
        
        The callback will be invoked when the container is changed.
        Returns a subscription that can be used to unsubscribe.
        
        The events will be emitted after a transaction is committed. A transaction is committed when:
        
        - `doc.commit()` is called.
        - `doc.export(mode)` is called.
        - `doc.import(data)` is called.
        - `doc.checkout(version)` is called.
        """
        ...

class LoroTree:
    is_attached: bool
    roots: list[TreeID]
    id: ContainerID
    def __contains__(self, target: TreeID) -> bool: ...
    def __new__(
        cls,
    ): ...
    def create(self, parent: typing.Optional[TreeID] = None) -> TreeID:
        r"""
        Create a new tree node and return the [`TreeID`].

        If the `parent` is `None`, the created node is the root of a tree.
        Otherwise, the created node is a child of the parent tree node.

        # Example

        ```rust
        use loro::LoroDoc;

        let doc = LoroDoc::new();
        let tree = doc.get_tree("tree");
        // create a root
        let root = tree.create(None).unwrap();
        // create a new child
        let child = tree.create(root).unwrap();
        ```
        """
        ...

    def create_at(self, index: int, parent: typing.Optional[TreeID] = None) -> TreeID:
        r"""
        Create a new tree node at the given index and return the [`TreeID`].

        If the `parent` is `None`, the created node is the root of a tree.
        If the `index` is greater than the number of children of the parent, error will be returned.

        # Example

        ```rust
        use loro::LoroDoc;

        let doc = LoroDoc::new();
        let tree = doc.get_tree("tree");
        // enable generate fractional index
        tree.enable_fractional_index(0);
        // create a root
        let root = tree.create(None).unwrap();
        // create a new child at index 0
        let child = tree.create_at(0, root).unwrap();
        ```
        """
        ...

    def mov(self, target: TreeID, parent: typing.Optional[TreeID] = None) -> None:
        r"""
        Move the `target` node to be a child of the `parent` node.

        If the `parent` is `None`, the `target` node will be a root.

        # Example

        ```rust
        use loro::LoroDoc;

        let doc = LoroDoc::new();
        let tree = doc.get_tree("tree");
        let root = tree.create(None).unwrap();
        let root2 = tree.create(None).unwrap();
        // move `root2` to be a child of `root`.
        tree.mov(root2, root).unwrap();
        ```
        """
        ...

    def mov_to(
        self, target: TreeID, to: int, parent: typing.Optional[TreeID] = None
    ) -> None:
        r"""
        Move the `target` node to be a child of the `parent` node at the given index.
        If the `parent` is `None`, the `target` node will be a root.

        # Example

        ```rust
        use loro::LoroDoc;

        let doc = LoroDoc::new();
        let tree = doc.get_tree("tree");
        // enable generate fractional index
        tree.enable_fractional_index(0);
        let root = tree.create(None).unwrap();
        let root2 = tree.create(None).unwrap();
        // move `root2` to be a child of `root` at index 0.
        tree.mov_to(root2, 0, root).unwrap();
        ```
        """
        ...

    def mov_after(self, target: TreeID, after: TreeID) -> None:
        r"""
        Move the `target` node to be a child after the `after` node with the same parent.

        # Example

        ```rust
        use loro::LoroDoc;

        let doc = LoroDoc::new();
        let tree = doc.get_tree("tree");
        // enable generate fractional index
        tree.enable_fractional_index(0);
        let root = tree.create(None).unwrap();
        let root2 = tree.create(None).unwrap();
        // move `root` to be a child after `root2`.
        tree.mov_after(root, root2).unwrap();
        ```
        """
        ...

    def mov_before(self, target: TreeID, before: TreeID) -> None:
        r"""
        Move the `target` node to be a child before the `before` node with the same parent.

        # Example

        ```rust
        use loro::LoroDoc;

        let doc = LoroDoc::new();
        let tree = doc.get_tree("tree");
        // enable generate fractional index
        tree.enable_fractional_index(0);
        let root = tree.create(None).unwrap();
        let root2 = tree.create(None).unwrap();
        // move `root` to be a child before `root2`.
        tree.mov_before(root, root2).unwrap();
        ```
        """
        ...

    def delete(self, target: TreeID) -> None:
        r"""
        Delete a tree node.

        Note: If the deleted node has children, the children do not appear in the state
        rather than actually being deleted.

        # Example

        ```rust
        use loro::LoroDoc;

        let doc = LoroDoc::new();
        let tree = doc.get_tree("tree");
        let root = tree.create(None).unwrap();
        tree.delete(root).unwrap();
        ```
        """
        ...

    def get_meta(self, target: TreeID) -> LoroMap:
        r"""
        Get the associated metadata map handler of a tree node.

        # Example
        ```rust
        use loro::LoroDoc;

        let doc = LoroDoc::new();
        let tree = doc.get_tree("tree");
        let root = tree.create(None).unwrap();
        let root_meta = tree.get_meta(root).unwrap();
        root_meta.insert("color", "red");
        ```
        """
        ...

    def parent(self, target: TreeID) -> typing.Optional[typing.Optional[TreeID]]:
        r"""
        Return the parent of target node.

        - If the target node does not exist, return `None`.
        - If the target node is a root node, return `Some(None)`.
        """
        ...

    def contains(self, target: TreeID) -> bool:
        r"""
        Return whether target node exists. including deleted node.
        """
        ...

    def is_node_deleted(self, target: TreeID) -> bool:
        r"""
        Return whether target node is deleted.

        # Errors

        - If the target node does not exist, return `LoroTreeError::TreeNodeNotExist`.
        """
        ...

    def nodes(self) -> list[TreeID]:
        r"""
        Return all node ids, including deleted nodes
        """
        ...

    def get_nodes(self, with_deleted: bool) -> list[TreeNode]:
        r"""
        Return all nodes, if `with_deleted` is true, the deleted nodes will be included.
        """
        ...

    def children(
        self, parent: typing.Optional[TreeID] = None
    ) -> typing.Optional[list[TreeID]]:
        r"""
        Return all children of the target node.

        If the parent node does not exist, return `None`.
        """
        ...

    def children_num(
        self, parent: typing.Optional[TreeID] = None
    ) -> typing.Optional[int]:
        r"""
        Return the number of children of the target node.
        """
        ...

    def fractional_index(self, target: TreeID) -> typing.Optional[str]:
        r"""
        Return the fractional index of the target node with hex format.
        """
        ...

    def get_value(self) -> LoroValue:
        r"""
        Return the hierarchy array of the forest.

        Note: the metadata will be not resolved. So if you don't only care about hierarchy
        but also the metadata, you should use [TreeHandler::get_value_with_meta()].
        """
        ...

    def get_value_with_meta(self) -> LoroValue:
        r"""
        Return the hierarchy array of the forest, each node is with metadata.
        """
        ...

    def is_fractional_index_enabled(self) -> bool:
        r"""
        Whether the fractional index is enabled.
        """
        ...

    def enable_fractional_index(self, jitter: int) -> None:
        r"""
        Enable fractional index for Tree Position.

        The jitter is used to avoid conflicts when multiple users are creating the node at the same position.
        value 0 is default, which means no jitter, any value larger than 0 will enable jitter.

        Generally speaking, jitter will affect the growth rate of document size.
        [Read more about it](https://www.loro.dev/blog/movable-tree#implementation-and-encoding-size)
        """
        ...

    def disable_fractional_index(self) -> None:
        r"""
        Disable the fractional index generation when you don't need the Tree's siblings to be sorted.
        The fractional index will always be set to the same default value 0.

        After calling this, you cannot use `tree.moveTo()`, `tree.moveBefore()`, `tree.moveAfter()`,
        and `tree.createAt()`.
        """
        ...

    def is_empty(self) -> bool:
        r"""
        Whether the tree is empty.
        """
        ...

    def get_last_move_id(self, target: TreeID) -> typing.Optional[ID]:
        r"""
        Get the last move id of the target node.
        """
        ...

    def doc(self) -> typing.Optional[LoroDoc]:
        r"""
        Get the LoroDoc of the container.
        """
        ...
    
    def subscribe(self, callback: typing.Callable[[DiffEvent], None]) -> typing.Optional[Subscription]:
        r"""
        Subscribe the events of a container.
        
        The callback will be invoked when the container is changed.
        Returns a subscription that can be used to unsubscribe.
        
        The events will be emitted after a transaction is committed. A transaction is committed when:
        
        - `doc.commit()` is called.
        - `doc.export(mode)` is called.
        - `doc.import(data)` is called.
        - `doc.checkout(version)` is called.
        """
        ...

class LoroUnknown:
    id: ContainerID

    def doc(self) -> typing.Optional[LoroDoc]:
        r"""
        Get the LoroDoc of the container.
        """
        ...
    
    def subscribe(self, callback: typing.Callable[[DiffEvent], None]) -> typing.Optional[Subscription]:
        r"""
        Subscribe the events of a container.
        
        The callback will be invoked when the container is changed.
        Returns a subscription that can be used to unsubscribe.
        
        The events will be emitted after a transaction is committed. A transaction is committed when:
        
        - `doc.commit()` is called.
        - `doc.export(mode)` is called.
        - `doc.import(data)` is called.
        - `doc.checkout(version)` is called.
        """
        ...

class MapDelta:
    updated: dict[str, typing.Optional[ValueOrContainer]]

class PathItem:
    container: ContainerID
    index: Index

class PeerInfo:
    state: LoroValue
    counter: int
    timestamp: int

class PosQueryResult:
    update: typing.Optional[Cursor]
    current: AbsolutePosition

class StyleConfigMap:
    def __new__(
        cls,
    ): ...
    def insert(self, key: str, value: ExpandType) -> None: ...
    def get(self, key: str) -> typing.Optional[ExpandType]: ...
    @classmethod
    def default_rich_text_config(cls) -> StyleConfigMap: ...

class Subscription:
    def detach(self) -> None:
        """
        Detaches the subscription from this handle. The callback will
        continue to be invoked until the doc has been subscribed to
        are dropped
        """
        ...

    def unsubscribe(self) -> None:
        """
        Unsubscribes the subscription. The callback will not be invoked anymore.
        """
        ...

    def __call__(self) -> None:
        """
        Unsubscribes the subscription. The callback will not be invoked anymore.
        """
        ...

class TreeDiff:
    diff: list[TreeDiffItem]

class TreeDiffItem:
    target: TreeID
    action: TreeExternalDiff

class TreeID:
    peer: int
    counter: int
    def __new__(cls, peer: int, counter: int): ...

class TreeNode:
    r"""
    A tree node in the [LoroTree].
    """

    id: TreeID
    parent: typing.Optional[TreeID]
    fractional_index: str
    index: int

class UndoItemMeta:
    value: LoroValue
    cursors: list[CursorWithPos]

class CursorWithPos:
    cursor: Cursor
    pos: AbsolutePosition

class UndoManager:
    def __new__(cls, doc: LoroDoc): ...
    def undo(self) -> bool:
        r"""
        Undo the last change made by the peer.
        """
        ...

    def redo(self) -> bool:
        r"""
        Redo the last change made by the peer.
        """
        ...
    
    def undo_count(self) -> int:
        r"""
        How many times the undo manager can undo.
        """
        ...
    
    def redo_count(self) -> int:
        r"""
        How many times the undo manager can redo.
        """
        ...

    def record_new_checkpoint(self) -> None:
        r"""
        Record a new checkpoint.
        """
        ...

    def can_undo(self) -> bool:
        r"""
        Whether the undo manager can undo.
        """
        ...

    def can_redo(self) -> bool:
        r"""
        Whether the undo manager can redo.
        """
        ...

    def add_exclude_origin_prefix(self, prefix: str) -> None:
        r"""
        If a local event's origin matches the given prefix, it will not be recorded in the
        undo stack.
        """
        ...

    def set_max_undo_steps(self, size: int) -> None:
        r"""
        Set the maximum number of undo steps. The default value is 100.
        """
        ...

    def set_merge_interval(self, interval: int) -> None:
        r"""
        Set the merge interval in ms. The default value is 0, which means no merge.
        """
        ...

    def set_on_push(
        self,
        on_push: typing.Callable[
            [UndoOrRedo, CounterSpan, typing.Optional[DiffEvent]], UndoItemMeta
        ],
    ) -> None:
        r"""
        Set the listener for push events.
        The listener will be called when a new undo/redo item is pushed into the stack.
        """
        ...

    def set_on_pop(
        self, on_pop: typing.Callable[[UndoOrRedo, CounterSpan, UndoItemMeta], None]
    ) -> None:
        r"""
        Set the listener for pop events.
        The listener will be called when an undo/redo item is popped from the stack.
        """
        ...

    def clear(self) -> None:
        r"""
        Clear the undo stack and the redo stack
        """
        ...

    def group_start(self) -> None:
        r"""
        Will start a new group of changes, all subsequent changes will be merged
        into a new item on the undo stack. If we receive remote changes, we determine
        whether or not they are conflicting. If the remote changes are conflicting
        we split the undo item and close the group. If there are no conflict
        in changed container ids we continue the group merge.
        """
        ...

    def group_end(self) -> None:
        r"""
        Ends the current group, calling UndoManager::undo() after this will
        undo all changes that occurred during the group.
        """
        ...

    def peer(self) -> int:
        r"""
        Get the peer id of the undo manager
        """
        ...

    def top_undo_meta(self) -> typing.Optional[UndoItemMeta]:
        r"""
        Get the metadata of the top undo stack item, if any.
        """
        ...
    
    def top_redo_meta(self) -> typing.Optional[UndoItemMeta]:
        r"""
        Get the metadata of the top redo stack item, if any.
        """
        ...
    
    def top_undo_value(self) -> typing.Optional[LoroValue]:
        r"""
        Get the value associated with the top undo stack item, if any.
        """
        ...
    
    def top_redo_value(self) -> typing.Optional[LoroValue]:
        r"""
        Get the value associated with the top redo stack item, if any.
        """
        ...

class VersionVector:
    def __new__(
        cls,
    ): ...
    def diff(self, rhs: VersionVector) -> VersionVectorDiff: ...
    def diff_iter(self, rhs: VersionVector) -> tuple[list[IdSpan], list[IdSpan]]:
        r"""
        Returns two iterators that cover the differences between two version vectors.

        - The first iterator contains the spans that are in `self` but not in `rhs`
        - The second iterator contains the spans that are in `rhs` but not in `self`
        """
        ...

    def sub_iter(self, rhs: VersionVector) -> list[IdSpan]:
        r"""
        Returns the spans that are in `self` but not in `rhs`
        """
        ...

    def iter_between(self, other: VersionVector) -> list[IdSpan]:
        r"""
        Iter all span from a -> b and b -> a
        """
        ...

    def sub_vec(self, rhs: VersionVector) -> VersionRange: ...
    def distance_between(self, other: VersionVector) -> int: ...
    def to_spans(self) -> VersionRange: ...
    def get_frontiers(self) -> Frontiers: ...
    def set_last(self, id: ID) -> None:
        r"""
        set the inclusive ending point. target id will be included by self
        """
        ...

    def get_last(self, client_id: int) -> typing.Optional[int]: ...
    def set_end(self, id: ID) -> None:
        r"""
        set the exclusive ending point. target id will NOT be included by self
        """
        ...

    def try_update_last(self, id: ID) -> bool:
        r"""
        Update the end counter of the given client if the end is greater.
        Return whether updated
        """
        ...

    def get_missing_span(self, target: VersionVector) -> list[IdSpan]: ...
    def merge(self, other: VersionVector) -> None: ...
    def includes_vv(self, other: VersionVector) -> bool: ...
    def includes_id(self, id: ID) -> bool: ...
    def intersect_span(self, target: IdSpan) -> typing.Optional[CounterSpan]: ...
    def extend_to_include_vv(self, vv: VersionVector) -> None: ...
    def extend_to_include_last_id(self, id: ID) -> None: ...
    def extend_to_include_end_id(self, id: ID) -> None: ...
    def extend_to_include(self, span: IdSpan) -> None: ...
    def shrink_to_exclude(self, span: IdSpan) -> None: ...
    def intersection(self, other: VersionVector) -> VersionVector: ...
    def encode(self) -> bytes: ...
    @classmethod
    def decode(cls, bytes: bytes) -> VersionVector: ...

class VersionVectorDiff:
    retreat: VersionRange
    forward: VersionRange

class VersionRange:
    is_empty: bool
    def __new__(
        cls,
    ): ...
    @classmethod
    def from_map(cls, map: dict) -> VersionRange: ...
    def clear(self) -> None: ...
    def get(self, peer: int) -> typing.Optional[tuple[int, int]]: ...
    def insert(self, peer: int, start: int, end: int) -> None: ...
    @classmethod
    def from_vv(cls, vv: VersionVector) -> VersionRange: ...
    def contains_ops_between(
        self, vv_a: VersionVector, vv_b: VersionVector
    ) -> bool: ...
    def has_overlap_with(self, span: IdSpan) -> bool: ...
    def contains_id(self, id: ID) -> bool: ...
    def contains_id_span(self, span: IdSpan) -> bool: ...
    def extends_to_include_id_span(self, span: IdSpan) -> None: ...
    def inner(self) -> dict[int, tuple[int, int]]: ...

class ContainerID:
    class Root(ContainerID):
        def __init__(self, name: str, container_type: ContainerType): ...
        name: str
        container_type: ContainerType

    class Normal(ContainerID):
        def __init__(self, peer: int, counter: int, container_type: ContainerType): ...
        peer: int
        counter: int
        container_type: ContainerType

class ContainerType:
    class Text(ContainerType):
        pass

    class Map(ContainerType):
        pass

    class List(ContainerType):
        pass

    class MovableList(ContainerType):
        pass

    class Tree(ContainerType):
        pass

    class Counter(ContainerType):
        pass

    class Unknown(ContainerType):
        def __init__(self, kind: int): ...
        kind: int

class DiffBatch:
    def __init__(self): ...
    def push(self, cid: ContainerID, diff: Diff) -> None: ...
    def get_diff(self) -> list[tuple[ContainerID, Diff]]: ...

class Diff:
    class List(Diff):
        def __init__(self, diff: list[ListDiffItem]): ...
        diff: list[ListDiffItem]

    class Text(Diff):
        def __init__(self, diff: list[TextDelta]): ...
        diff: list[TextDelta]

    class Map(Diff):
        def __init__(self, diff: MapDelta): ...
        diff: MapDelta

    class Tree(Diff):
        def __init__(self, diff: TreeDiff): ...
        diff: TreeDiff

    class Counter(Diff):
        def __init__(self, diff: float): ...
        diff: float

    class Unknown(Diff):
        pass

class EncodedBlobMode(Enum):
    Snapshot = "snapshot"
    OutdatedSnapshot = "outdated_snapshot"
    ShallowSnapshot = "shallow_snapshot"
    OutdatedRle = "outdated_rle"
    Updates = "updates"


class EventTriggerKind(Enum):
    r"""
    The kind of the event trigger.
    """
    Local = "local"
    Import = "import"
    Checkout = "checkout"

class ExpandType(Enum):
    r"""
    Whether to expand the style when inserting new text around it.

    - Before: when inserting new text before this style, the new text should inherit this style.
    - After: when inserting new text after this style, the new text should inherit this style.
    - Both: when inserting new text before or after this style, the new text should inherit this style.
    - None: when inserting new text before or after this style, the new text should **not** inherit this style.
    """
    Before = "before"
    After = "after"
    Both = "both"
    Null = "null"

class ExportMode:
    class Snapshot(ExportMode):
        pass

    class Updates(ExportMode):
        def __init__(self, from_: VersionVector): ...
        from_: VersionVector

    class UpdatesInRange(ExportMode):
        def __init__(self, spans: list[IdSpan]): ...
        spans: list[IdSpan]

    class ShallowSnapshot(ExportMode):
        def __init__(self, frontiers: Frontiers): ...
        frontiers: Frontiers

    class StateOnly(ExportMode):
        def __init__(self, frontiers: typing.Optional[Frontiers]): ...
        frontiers: typing.Optional[Frontiers]

    class SnapshotAt(ExportMode):
        def __init__(self, version: Frontiers): ...
        version: Frontiers

class Index:
    class Key(Index):
        def __init__(self, key: str): ...
        key: str

    class Seq(Index):
        def __init__(self, index: int): ...
        index: int

    class Node(Index):
        def __init__(self, target: TreeID): ...
        target: TreeID

class ListDiffItem:
    class Insert(ListDiffItem):
        def __init__(self, insert: list[ValueOrContainer], is_move: bool): ...
        insert: list[ValueOrContainer]
        is_move: bool

    class Delete(ListDiffItem):
        def __init__(self, delete: int): ...
        delete: int

    class Retain(ListDiffItem):
        def __init__(self, retain: int): ...
        retain: int

class Ordering(Enum):
    Less = "less"
    Equal = "equal"
    Greater = "greater"

class Side(Enum):
    Left = "left"
    Middle = "middle"
    Right = "right"

class PositionType(Enum):
    Bytes = "bytes"
    Unicode = "unicode"
    Utf16 = "utf16"
    Event = "event"
    Entity = "entity"

class TextDelta:
    class Retain(TextDelta):
        def __init__(
            self,
            retain: int,
            attributes: typing.Optional[typing.Mapping[str, LoroValue]],
        ): ...
        retain: int
        attributes: typing.Optional[dict[str, LoroValue]]

    class Insert(TextDelta):
        def __init__(
            self,
            insert: str,
            attributes: typing.Optional[typing.Mapping[str, LoroValue]],
        ): ...
        insert: str
        attributes: typing.Optional[dict[str, LoroValue]]

    class Delete(TextDelta):
        def __init__(self, delete: int): ...
        delete: int

class TreeExternalDiff:
    class Create(TreeExternalDiff):
        parent: typing.Optional[TreeID]
        index: int
        fractional_index: str

    class Move(TreeExternalDiff):
        def __init__(
            self,
            parent: typing.Optional[TreeID],
            index: int,
            fractional_index: str,
            old_parent: typing.Optional[TreeID],
            old_index: int,
        ): ...
        parent: typing.Optional[TreeID]
        index: int
        fractional_index: str
        old_parent: typing.Optional[TreeID]
        old_index: int

    class Delete(TreeExternalDiff):
        def __init__(self, old_parent: typing.Optional[TreeID], old_index: int): ...
        old_parent: typing.Optional[TreeID]
        old_index: int

class UndoOrRedo(Enum):
    Undo = "undo"
    Redo = "redo"

class ValueOrContainer:
    class Value(ValueOrContainer):
        def __init__(self, value: LoroValue): ...
        value: LoroValue

    class Container(ValueOrContainer):
        def __init__(self, container: Container): ...
        container: Container

    @classmethod
    def is_value(
        cls, value: ValueOrContainer | None
    ) -> typing.TypeGuard[ValueOrContainer.Value]: ...
    @classmethod
    def is_container(
        cls, value: ValueOrContainer | None
    ) -> typing.TypeGuard[ValueOrContainer.Container]: ...

class EphemeralStore:
    r"""
    A store for ephemeral data that automatically expires after a timeout.
    """
    def __new__(cls, timeout: int) -> EphemeralStore: ...

    def encode(self, key: str) -> bytes:
        r"""
        Encode the state of a specific key into bytes.
        """
        ...

    def encode_all(self) -> bytes:
        r"""
        Encode all states into bytes.
        """
        ...

    def apply(self, data: bytes) -> None:
        r"""
        Apply encoded data to update the store.
        """
        ...

    def set(self, key: str, value: LoroValue) -> None:
        r"""
        Set a value for a key.
        """
        ...

    def delete(self, key: str) -> None:
        r"""
        Delete a key from the store.
        """
        ...

    def get(self, key: str) -> typing.Optional[LoroValue]:
        r"""
        Get the value of a key.
        """
        ...

    def remove_outdated(self) -> None:
        r"""
        Remove all outdated entries.
        """
        ...

    def get_all_states(self) -> dict[str, LoroValue]:
        r"""
        Get all states in the store.
        """
        ...

    def keys(self) -> list[str]:
        r"""
        Get all keys in the store.
        """
        ...

    def subscribe_local_updates(
        self, callback: typing.Callable[[bytes], bool]
    ) -> Subscription:
        r"""
        Subscribe to local updates.
        """
        ...

    def subscribe(
        self, callback: typing.Callable[[EphemeralStoreEvent], bool]
    ) -> Subscription:
        r"""
        Subscribe to all updates.
        """
        ...

class EphemeralStoreEvent:
    r"""
    An event that represents changes in the EphemeralStore.
    """
    by: typing.Literal["Local", "Import", "Timeout"]
    added: list[str]
    updated: list[str]
    removed: list[str]

class ChangeModifier:
    r"""
    A modifier that can be used to modify the commit message and timestamp in a pre-commit callback.
    """
    def set_message(self, msg: str) -> None:
        r"""
        Set the commit message.
        """
        ...

    def set_timestamp(self, timestamp: int) -> None:
        r"""
        Set the commit timestamp.
        """
        ...

class PreCommitCallbackPayload:
    r"""
    The payload passed to the pre-commit callback.
    """
    change_meta: ChangeMeta
    origin: str
    modifier: ChangeModifier

class FirstCommitFromPeerPayload:
    r"""
    The payload passed to the first commit from peer callback.
    """
    peer: int


class IdLp:
    r"""
    ID with Lamport timestamp.
    """
    peer: int
    lamport: int
    
    def __new__(cls, peer: int, lamport: int) -> IdLp: ...
