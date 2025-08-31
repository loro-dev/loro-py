use crate::{
    doc::{CounterSpan, IdSpan},
    err::PyLoroResult,
    value::ID,
};
use loro::{Counter, PeerID};
use pyo3::{
    basic::CompareOp,
    exceptions::PyNotImplementedError,
    prelude::*,
    types::{PyBytes, PyDict, PyType},
};
use std::{borrow::Cow, collections::HashMap, fmt::Display};

pub fn register_class(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Frontiers>()?;
    m.add_class::<VersionRange>()?;
    m.add_class::<VersionVector>()?;
    Ok(())
}

#[pyclass(str)]
#[derive(Debug, Clone, Default)]
pub struct Frontiers(loro::Frontiers);

impl Display for Frontiers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[pymethods]
impl Frontiers {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    #[classmethod]
    pub fn from_id(_cls: &Bound<'_, PyType>, id: ID) -> Self {
        Self(loro::Frontiers::from(loro::ID::from(id)))
    }

    #[classmethod]
    pub fn from_ids(_cls: &Bound<'_, PyType>, ids: Vec<ID>) -> Self {
        Self(loro::Frontiers::from(
            ids.into_iter().map(loro::ID::from).collect::<Vec<_>>(),
        ))
    }

    pub fn encode(&self) -> Cow<'_, [u8]> {
        let ans: Vec<u8> = self.0.encode();
        Cow::Owned(ans)
    }

    #[classmethod]
    pub fn decode(_cls: &Bound<'_, PyType>, bytes: Bound<'_, PyBytes>) -> PyLoroResult<Self> {
        let ans = Self(loro::Frontiers::decode(bytes.as_bytes())?);
        Ok(ans)
    }
}

impl From<Frontiers> for loro::Frontiers {
    fn from(value: Frontiers) -> Self {
        value.0
    }
}

impl From<loro::Frontiers> for Frontiers {
    fn from(value: loro::Frontiers) -> Self {
        Self(value)
    }
}

impl From<&Frontiers> for loro::Frontiers {
    fn from(value: &Frontiers) -> Self {
        value.0.clone()
    }
}

#[pyclass(str)]
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct VersionRange(loro::VersionRange);

#[pymethods]
impl VersionRange {
    #[new]
    pub fn new() -> Self {
        Self(Default::default())
    }

    #[classmethod]
    pub fn from_map(_cls: &Bound<'_, PyType>, map: Bound<'_, PyDict>) -> PyLoroResult<Self> {
        let mut ans = Self::new();
        for peer in map.keys() {
            let peer = peer.extract::<PeerID>().unwrap();
            let (start, end) = map
                .get_item(peer)?
                .unwrap()
                .extract::<(Counter, Counter)>()
                .unwrap();
            ans.insert(peer, start, end);
        }
        Ok(ans)
    }

    // TODO: iter
    // pub fn iter(&self) -> impl Iterator<Item = (&PeerID, &(Counter, Counter))> + '_ {
    //     self.0.iter()
    // }

    // pub fn iter_mut(&mut self) -> impl Iterator<Item = (&PeerID, &mut (Counter, Counter))> + '_ {
    //     self.0.iter_mut()
    // }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn get(&self, peer: PeerID) -> Option<&(Counter, Counter)> {
        self.0.get(&peer)
    }

    pub fn insert(&mut self, peer: PeerID, start: Counter, end: Counter) {
        self.0.insert(peer, start, end);
    }

    #[classmethod]
    pub fn from_vv(_cls: &Bound<'_, PyType>, vv: &VersionVector) -> Self {
        Self(loro::VersionRange::from_vv(&vv.0))
    }

    pub fn contains_ops_between(&self, vv_a: &VersionVector, vv_b: &VersionVector) -> bool {
        self.0.contains_ops_between(&vv_a.0, &vv_b.0)
    }

    pub fn has_overlap_with(&self, span: IdSpan) -> bool {
        self.0.has_overlap_with(span.into())
    }

    pub fn contains_id(&self, id: ID) -> bool {
        self.0.contains_id(id.into())
    }

    pub fn contains_id_span(&self, span: IdSpan) -> bool {
        self.0.contains_id_span(span.into())
    }

    pub fn extends_to_include_id_span(&mut self, span: IdSpan) {
        self.0.extends_to_include_id_span(span.into())
    }

    #[getter]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn inner(&self) -> HashMap<PeerID, (Counter, Counter)> {
        self.0.inner().iter().map(|(k, v)| (*k, *v)).collect()
    }
}

impl Display for VersionRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl From<VersionRange> for loro::VersionRange {
    fn from(value: VersionRange) -> Self {
        value.0
    }
}

impl From<loro::VersionRange> for VersionRange {
    fn from(value: loro::VersionRange) -> Self {
        Self(value)
    }
}

#[pyclass(str)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionVector(loro::VersionVector);

#[pymethods]
impl VersionVector {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn diff(&self, rhs: &Self) -> VersionVectorDiff {
        VersionVectorDiff::from(self.0.diff(&rhs.0))
    }

    /// Returns two iterators that cover the differences between two version vectors.
    ///
    /// - The first iterator contains the spans that are in `self` but not in `rhs`
    /// - The second iterator contains the spans that are in `rhs` but not in `self`
    pub fn diff_iter<'a>(&'a self, rhs: &'a Self) -> (Vec<IdSpan>, Vec<IdSpan>) {
        (self.sub_iter(rhs), rhs.sub_iter(self))
    }

    /// Returns the spans that are in `self` but not in `rhs`
    pub fn sub_iter<'a>(&'a self, rhs: &'a Self) -> Vec<IdSpan> {
        self.0.sub_iter(&rhs.0).map(|x| x.into()).collect()
    }

    /// Iter all span from a -> b and b -> a
    pub fn iter_between<'a>(&'a self, other: &'a Self) -> Vec<IdSpan> {
        // PERF: can be optimized a little
        self.0.iter_between(&other.0).map(|x| x.into()).collect()
    }

    pub fn sub_vec(&self, rhs: &Self) -> VersionRange {
        let v = self.0.sub_vec(&rhs.0);
        VersionRange(loro::VersionRange::from_map(
            v.iter().map(|(k, v)| (*k, (v.start, v.end))).collect(),
        ))
    }

    pub fn distance_between(&self, other: &Self) -> usize {
        self.0.distance_between(&other.0)
    }

    pub fn to_spans(&self) -> VersionRange {
        VersionRange(loro::VersionRange::from_map(
            self.0
                .to_spans()
                .iter()
                .map(|(k, v)| (*k, (v.start, v.end)))
                .collect(),
        ))
    }

    #[inline]
    pub fn get_frontiers(&self) -> Frontiers {
        self.0.get_frontiers().into()
    }

    /// set the inclusive ending point. target id will be included by self
    #[inline]
    pub fn set_last(&mut self, id: ID) {
        self.0.set_last(id.into());
    }

    #[inline]
    pub fn get_last(&self, client_id: PeerID) -> Option<Counter> {
        self.0.get_last(client_id)
    }

    /// set the exclusive ending point. target id will NOT be included by self
    #[inline]
    pub fn set_end(&mut self, id: ID) {
        self.0.set_end(id.into());
    }

    /// Update the end counter of the given client if the end is greater.
    /// Return whether updated
    #[inline]
    pub fn try_update_last(&mut self, id: ID) -> bool {
        self.0.try_update_last(id.into())
    }

    pub fn get_missing_span(&self, target: &Self) -> Vec<IdSpan> {
        self.0
            .get_missing_span(&target.0)
            .into_iter()
            .map(|x| x.into())
            .collect()
    }

    pub fn merge(&mut self, other: &Self) {
        self.0.merge(&other.0);
    }

    pub fn includes_vv(&self, other: &VersionVector) -> bool {
        self.0.includes_vv(&other.0)
    }

    pub fn includes_id(&self, id: ID) -> bool {
        self.0.includes_id(id.into())
    }

    pub fn intersect_span(&self, target: IdSpan) -> Option<CounterSpan> {
        self.0.intersect_span(target.into()).map(|x| x.into())
    }

    pub fn extend_to_include_vv(&mut self, vv: VersionVector) {
        self.0.extend_to_include_vv(vv.0.iter());
    }

    pub fn extend_to_include_last_id(&mut self, id: ID) {
        self.0.extend_to_include_last_id(id.into());
    }

    pub fn extend_to_include_end_id(&mut self, id: ID) {
        self.0.extend_to_include_end_id(id.into());
    }

    pub fn extend_to_include(&mut self, span: IdSpan) {
        self.0.extend_to_include(span.into());
    }

    pub fn shrink_to_exclude(&mut self, span: IdSpan) {
        self.0.shrink_to_exclude(span.into());
    }

    pub fn intersection(&self, other: &VersionVector) -> VersionVector {
        self.0.intersection(&other.0).into()
    }

    #[inline(always)]
    pub fn encode(&self) -> Cow<'_, [u8]> {
        let ans: Vec<u8> = self.0.encode();
        Cow::Owned(ans)
    }

    #[classmethod]
    #[inline(always)]
    pub fn decode(_cls: &Bound<'_, PyType>, bytes: Bound<'_, PyBytes>) -> PyLoroResult<Self> {
        let ans = Self(loro::VersionVector::decode(bytes.as_bytes())?);
        Ok(ans)
    }

    fn __richcmp__(&self, other: PyRef<Self>, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self.0 == other.0),
            CompareOp::Ne => Ok(self.0 != other.0),
            _ => Err(PyNotImplementedError::new_err(
                "Only == and != comparisons are supported",
            )),
        }
    }
}

impl Default for VersionVector {
    fn default() -> Self {
        Self(loro::VersionVector::new())
    }
}

impl Display for VersionVector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl From<VersionVector> for loro::VersionVector {
    fn from(value: VersionVector) -> Self {
        value.0
    }
}

impl From<loro::VersionVector> for VersionVector {
    fn from(value: loro::VersionVector) -> Self {
        Self(value)
    }
}

#[pyclass(str, get_all)]
#[derive(Debug, Clone)]
pub struct VersionVectorDiff {
    pub retreat: VersionRange,
    pub forward: VersionRange,
}

impl Display for VersionVectorDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<loro::VersionVectorDiff> for VersionVectorDiff {
    // TODO: a better way in loro-rs
    fn from(value: loro::VersionVectorDiff) -> Self {
        Self {
            retreat: VersionRange(loro::VersionRange::from_map(
                value
                    .retreat
                    .iter()
                    .map(|(k, v)| (*k, (v.start, v.end)))
                    .collect(),
            )),
            forward: VersionRange(loro::VersionRange::from_map(
                value
                    .forward
                    .iter()
                    .map(|(k, v)| (*k, (v.start, v.end)))
                    .collect(),
            )),
        }
    }
}
