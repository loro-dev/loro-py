use pyo3::prelude::*;

mod counter;
mod list;
mod map;
mod movable_list;
mod text;
mod tree;
mod unknown;
pub use counter::LoroCounter;
pub use list::LoroList;
pub use map::LoroMap;
pub use movable_list::LoroMovableList;
use pyo3_stub_gen_derive::*;
pub use text::{Cursor, LoroText, Side, UpdateOptions};
pub use tree::{LoroTree, TreeNode};
pub use unknown::LoroUnknown;

#[gen_stub_pyclass_enum]
#[derive(Debug, Clone, FromPyObject, IntoPyObject)]
pub enum Container {
    List(LoroList),
    Map(LoroMap),
    MovableList(LoroMovableList),
    Text(LoroText),
    Tree(LoroTree),
    Counter(LoroCounter),
    Unknown(LoroUnknown),
}

pub fn register_class(m: &Bound<'_, PyModule>) -> PyResult<()> {
    text::register_class(m)?;
    map::register_class(m)?;
    m.add_class::<LoroList>()?;
    m.add_class::<LoroTree>()?;
    m.add_class::<LoroMovableList>()?;
    m.add_class::<LoroCounter>()?;
    m.add_class::<LoroUnknown>()?;
    Ok(())
}
