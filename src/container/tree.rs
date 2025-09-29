use std::sync::Arc;

use loro::{ContainerTrait, LoroTree as LoroTreeInner};
use pyo3::prelude::*;

use crate::{
    convert::tree_parent_id_to_option_tree_id,
    doc::LoroDoc,
    err::PyLoroResult,
    event::{DiffEvent, Subscription},
    value::{ContainerID, LoroValue, TreeID, TreeParentId, ID},
};

use super::LoroMap;

pub fn register_class(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<LoroTree>()?;
    m.add_class::<TreeNode>()?;
    Ok(())
}

#[pyclass(frozen)]
#[derive(Debug, Clone, Default)]
pub struct LoroTree(pub LoroTreeInner);

#[pymethods]
impl LoroTree {
    /// Create a new container that is detached from the document.
    ///
    /// The edits on a detached container will not be persisted.
    /// To attach the container to the document, please insert it into an attached container.
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the container is attached to a document
    ///
    /// The edits on a detached container will not be persisted.
    /// To attach the container to the document, please insert it into an attached container.
    #[getter]
    pub fn is_attached(&self) -> bool {
        self.0.is_attached()
    }

    pub fn __contains__(&self, target: TreeID) -> bool {
        self.contains(target)
    }

    /// Create a new tree node and return the [`TreeID`].
    ///
    /// If the `parent` is `None`, the created node is the root of a tree.
    /// Otherwise, the created node is a child of the parent tree node.
    ///
    /// # Example
    ///
    /// ```rust
    /// use loro::LoroDoc;
    ///
    /// let doc = LoroDoc::new();
    /// let tree = doc.get_tree("tree");
    /// // create a root
    /// let root = tree.create(None).unwrap();
    /// // create a new child
    /// let child = tree.create(root).unwrap();
    /// ```
    #[pyo3(signature = (parent=None))]
    pub fn create(&self, parent: Option<TreeID>) -> PyLoroResult<TreeID> {
        let ans = self.0.create(parent.map(loro::TreeID::from))?.into();
        Ok(ans)
    }

    /// Get the root nodes of the forest.
    #[getter]
    pub fn roots(&self) -> Vec<TreeID> {
        self.0.roots().into_iter().map(|x| x.into()).collect()
    }

    /// Create a new tree node at the given index and return the [`TreeID`].
    ///
    /// If the `parent` is `None`, the created node is the root of a tree.
    /// If the `index` is greater than the number of children of the parent, error will be returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use loro::LoroDoc;
    ///
    /// let doc = LoroDoc::new();
    /// let tree = doc.get_tree("tree");
    /// // enable generate fractional index
    /// tree.enable_fractional_index(0);
    /// // create a root
    /// let root = tree.create(None).unwrap();
    /// // create a new child at index 0
    /// let child = tree.create_at(root, 0).unwrap();
    /// ```
    #[pyo3(signature = (index, parent=None))]
    pub fn create_at(&self, index: usize, parent: Option<TreeID>) -> PyLoroResult<TreeID> {
        let ans = self
            .0
            .create_at(parent.map(loro::TreeID::from), index)?
            .into();
        Ok(ans)
    }

    /// Move the `target` node to be a child of the `parent` node.
    ///
    /// If the `parent` is `None`, the `target` node will be a root.
    ///
    /// # Example
    ///
    /// ```rust
    /// use loro::LoroDoc;
    ///
    /// let doc = LoroDoc::new();
    /// let tree = doc.get_tree("tree");
    /// let root = tree.create(None).unwrap();
    /// let root2 = tree.create(None).unwrap();
    /// // move `root2` to be a child of `root`.
    /// tree.mov(root2, root).unwrap();
    /// ```
    #[pyo3(signature = (target,parent=None))]
    pub fn mov(&self, target: TreeID, parent: Option<TreeID>) -> PyLoroResult<()> {
        self.0.mov(target.into(), parent.map(loro::TreeID::from))?;
        Ok(())
    }

    /// Move the `target` node to be a child of the `parent` node at the given index.
    /// If the `parent` is `None`, the `target` node will be a root.
    ///
    /// # Example
    ///
    /// ```rust
    /// use loro::LoroDoc;
    ///
    /// let doc = LoroDoc::new();
    /// let tree = doc.get_tree("tree");
    /// // enable generate fractional index
    /// tree.enable_fractional_index(0);
    /// let root = tree.create(None).unwrap();
    /// let root2 = tree.create(None).unwrap();
    /// // move `root2` to be a child of `root` at index 0.
    /// tree.mov_to(root2, 0, root).unwrap();
    /// ```
    #[pyo3(signature = (target, to, parent=None))]
    pub fn mov_to(&self, target: TreeID, to: usize, parent: Option<TreeID>) -> PyLoroResult<()> {
        self.0
            .mov_to(target.into(), parent.map(loro::TreeID::from), to)?;
        Ok(())
    }

    /// Move the `target` node to be a child after the `after` node with the same parent.
    ///
    /// # Example
    ///
    /// ```rust
    /// use loro::LoroDoc;
    ///
    /// let doc = LoroDoc::new();
    /// let tree = doc.get_tree("tree");
    /// // enable generate fractional index
    /// tree.enable_fractional_index(0);
    /// let root = tree.create(None).unwrap();
    /// let root2 = tree.create(None).unwrap();
    /// // move `root` to be a child after `root2`.
    /// tree.mov_after(root, root2).unwrap();
    /// ```
    pub fn mov_after(&self, target: TreeID, after: TreeID) -> PyLoroResult<()> {
        self.0.mov_after(target.into(), after.into())?;
        Ok(())
    }

    /// Move the `target` node to be a child before the `before` node with the same parent.
    ///
    /// # Example
    ///
    /// ```rust
    /// use loro::LoroDoc;
    ///
    /// let doc = LoroDoc::new();
    /// let tree = doc.get_tree("tree");
    /// // enable generate fractional index
    /// tree.enable_fractional_index(0);
    /// let root = tree.create(None).unwrap();
    /// let root2 = tree.create(None).unwrap();
    /// // move `root` to be a child before `root2`.
    /// tree.mov_before(root, root2).unwrap();
    /// ```
    pub fn mov_before(&self, target: TreeID, before: TreeID) -> PyLoroResult<()> {
        self.0.mov_before(target.into(), before.into())?;
        Ok(())
    }

    /// Delete a tree node.
    ///
    /// Note: If the deleted node has children, the children do not appear in the state
    /// rather than actually being deleted.
    ///
    /// # Example
    ///
    /// ```rust
    /// use loro::LoroDoc;
    ///
    /// let doc = LoroDoc::new();
    /// let tree = doc.get_tree("tree");
    /// let root = tree.create(None).unwrap();
    /// tree.delete(root).unwrap();
    /// ```
    pub fn delete(&self, target: TreeID) -> PyLoroResult<()> {
        self.0.delete(target.into())?;
        Ok(())
    }

    /// Get the associated metadata map handler of a tree node.
    ///
    /// # Example
    /// ```rust
    /// use loro::LoroDoc;
    ///
    /// let doc = LoroDoc::new();
    /// let tree = doc.get_tree("tree");
    /// let root = tree.create(None).unwrap();
    /// let root_meta = tree.get_meta(root).unwrap();
    /// root_meta.insert("color", "red");
    /// ```
    pub fn get_meta(&self, target: TreeID) -> PyLoroResult<LoroMap> {
        let ans = self.0.get_meta(target.into()).map(LoroMap)?;
        Ok(ans)
    }

    /// Return the parent of target node.
    ///
    /// - If the target node does not exist, return `None`.
    /// - If the target node is a root node, return `Some(None)`.
    pub fn parent(&self, target: TreeID) -> Option<Option<TreeID>> {
        self.0
            .parent(target.into())
            .map(tree_parent_id_to_option_tree_id)
    }

    /// Return whether target node exists. including deleted node.
    pub fn contains(&self, target: TreeID) -> bool {
        self.0.contains(target.into())
    }

    /// Return whether target node is deleted.
    ///
    /// # Errors
    ///
    /// - If the target node does not exist, return `LoroTreeError::TreeNodeNotExist`.
    pub fn is_node_deleted(&self, target: &TreeID) -> PyLoroResult<bool> {
        let ans = self.0.is_node_deleted(&(*target).into())?;
        Ok(ans)
    }

    /// Return all node ids, including deleted nodes
    pub fn nodes(&self) -> Vec<TreeID> {
        self.0.nodes().into_iter().map(|x| x.into()).collect()
    }

    /// Return all nodes, if `with_deleted` is true, the deleted nodes will be included.
    pub fn get_nodes(&self, with_deleted: bool) -> Vec<TreeNode> {
        self.0
            .get_nodes(with_deleted)
            .into_iter()
            .map(|x| x.into())
            .collect()
    }

    /// Return all children of the target node.
    ///
    /// If the parent node does not exist, return `None`.
    #[pyo3(signature = (parent=None))]
    pub fn children(&self, parent: Option<TreeID>) -> Option<Vec<TreeID>> {
        self.0
            .children(parent.map(loro::TreeID::from))
            .map(|x| x.into_iter().map(|x| x.into()).collect())
    }

    /// Return the number of children of the target node.
    #[pyo3(signature = (parent=None))]
    pub fn children_num(&self, parent: Option<TreeID>) -> Option<usize> {
        self.0.children_num(parent.map(loro::TreeID::from))
    }

    /// Return container id of the tree.
    #[getter]
    pub fn id(&self) -> ContainerID {
        self.0.id().into()
    }

    /// Return the fractional index of the target node with hex format.
    pub fn fractional_index(&self, target: TreeID) -> Option<String> {
        self.0.fractional_index(target.into())
    }

    /// Return the hierarchy array of the forest.
    ///
    /// Note: the metadata will be not resolved. So if you don't only care about hierarchy
    /// but also the metadata, you should use [TreeHandler::get_value_with_meta()].
    pub fn get_value(&self) -> LoroValue {
        self.0.get_value().into()
    }

    /// Return the hierarchy array of the forest, each node is with metadata.
    pub fn get_value_with_meta(&self) -> LoroValue {
        self.0.get_value_with_meta().into()
    }

    /// Whether the fractional index is enabled.
    pub fn is_fractional_index_enabled(&self) -> bool {
        self.0.is_fractional_index_enabled()
    }

    /// Enable fractional index for Tree Position.
    ///
    /// The jitter is used to avoid conflicts when multiple users are creating the node at the same position.
    /// value 0 is default, which means no jitter, any value larger than 0 will enable jitter.
    ///
    /// Generally speaking, jitter will affect the growth rate of document size.
    /// [Read more about it](https://www.loro.dev/blog/movable-tree#implementation-and-encoding-size)
    #[inline]
    pub fn enable_fractional_index(&self, jitter: u8) {
        self.0.enable_fractional_index(jitter);
    }

    /// Disable the fractional index generation when you don't need the Tree's siblings to be sorted.
    /// The fractional index will always be set to the same default value 0.
    ///
    /// After calling this, you cannot use `tree.moveTo()`, `tree.moveBefore()`, `tree.moveAfter()`,
    /// and `tree.createAt()`.
    #[inline]
    pub fn disable_fractional_index(&self) {
        self.0.disable_fractional_index();
    }

    /// Whether the tree is empty.
    ///
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the last move id of the target node.
    pub fn get_last_move_id(&self, target: &TreeID) -> Option<ID> {
        self.0.get_last_move_id(&(*target).into()).map(|x| x.into())
    }

    pub fn doc(&self) -> Option<LoroDoc> {
        self.0.doc().map(|doc| doc.into())
    }

    /// Subscribe the events of a container.
    ///
    /// The callback will be invoked when the container is changed.
    /// Returns a subscription that can be used to unsubscribe.
    ///
    /// The events will be emitted after a transaction is committed. A transaction is committed when:
    ///
    /// - `doc.commit()` is called.
    /// - `doc.export(mode)` is called.
    /// - `doc.import(data)` is called.
    /// - `doc.checkout(version)` is called.
    pub fn subscribe(&self, callback: Py<PyAny>) -> Option<Subscription> {
        let subscription = self.0.subscribe(Arc::new(move |e| {
            Python::attach(|py| {
                callback.call1(py, (DiffEvent::from(e),)).unwrap();
            });
        }));
        subscription.map(|s| s.into())
    }
}

/// A tree node in the [LoroTree].
#[pyclass(str, get_all, set_all)]
#[derive(Debug, Clone)]
pub struct TreeNode {
    /// ID of the tree node.
    pub id: TreeID,
    /// ID of the parent tree node.
    /// If the ndoe is deleted this value is TreeParentId::Deleted.
    /// If you checkout to a version before the node is created, this value is TreeParentId::Unexist.
    pub parent: TreeParentId,
    /// Fraction index of the node
    pub fractional_index: String,
    /// The current index of the node in its parent's children list.
    pub index: usize,
}

impl std::fmt::Display for TreeNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
