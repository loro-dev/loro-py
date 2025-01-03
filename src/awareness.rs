use std::collections::HashMap;

use loro::PeerID;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

use crate::value::LoroValue;

pub fn register_class(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Awareness>()?;
    Ok(())
}

#[gen_stub_pyclass]
#[pyclass]
pub struct Awareness(loro::awareness::Awareness);

#[gen_stub_pymethods]
#[pymethods]
impl Awareness {
    #[new]
    pub fn new(peer: PeerID, timeout: i64) -> Self {
        Self(loro::awareness::Awareness::new(peer, timeout))
    }

    pub fn encode(&self, peers: Vec<PeerID>) -> Vec<u8> {
        self.0.encode(&peers)
    }

    pub fn encode_all(&self) -> Vec<u8> {
        self.0.encode_all()
    }

    pub fn apply(&mut self, encoded_peers_info: Vec<u8>) -> AwarenessPeerUpdate {
        let (updated, added) = self.0.apply(&encoded_peers_info);
        AwarenessPeerUpdate { updated, added }
    }

    #[setter]
    #[pyo3(name = "local_state")]
    pub fn set_local_state(&mut self, value: LoroValue) {
        self.0.set_local_state(value);
    }

    #[getter]
    #[pyo3(name = "local_state")]
    pub fn get_local_state(&self) -> Option<LoroValue> {
        self.0.get_local_state().map(|x| x.into())
    }

    pub fn remove_outdated(&mut self) -> Vec<PeerID> {
        self.0.remove_outdated()
    }

    #[getter]
    #[pyo3(name = "all_states")]
    pub fn get_all_states(&self) -> HashMap<PeerID, PeerInfo> {
        self.0
            .get_all_states()
            .iter()
            .map(|(p, i)| (*p, i.into()))
            .collect()
    }

    #[getter]
    pub fn peer(&self) -> PeerID {
        self.0.peer()
    }
}

#[gen_stub_pyclass]
#[derive(Debug, Clone, IntoPyObject)]
pub struct AwarenessPeerUpdate {
    pub updated: Vec<PeerID>,
    pub added: Vec<PeerID>,
}

#[gen_stub_pyclass]
#[derive(Debug, Clone, IntoPyObject)]
pub struct PeerInfo {
    pub state: LoroValue,
    pub counter: i32,
    pub timestamp: i64,
}
