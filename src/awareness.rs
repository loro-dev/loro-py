#![allow(deprecated)]
use std::{borrow::Cow, collections::HashMap};

use loro::{awareness::EphemeralEventTrigger, LoroError, PeerID};
use pyo3::{prelude::*, types::PyBytes};

use crate::{err::PyLoroResult, event::Subscription, value::LoroValue};

pub fn register_class(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Awareness>()?;
    m.add_class::<AwarenessPeerUpdate>()?;
    m.add_class::<PeerInfo>()?;
    m.add_class::<EphemeralStore>()?;
    m.add_class::<EphemeralStoreEvent>()?;
    Ok(())
}

#[pyclass]
pub struct Awareness(loro::awareness::Awareness);

#[pymethods]
impl Awareness {
    #[new]
    pub fn new(peer: PeerID, timeout: i64) -> Self {
        Self(loro::awareness::Awareness::new(peer, timeout))
    }

    pub fn encode(&self, peers: Vec<PeerID>) -> Cow<'_, [u8]> {
        let ans: Vec<u8> = self.0.encode(&peers);
        Cow::Owned(ans)
    }

    pub fn encode_all(&self) -> Cow<'_, [u8]> {
        let ans: Vec<u8> = self.0.encode_all();
        Cow::Owned(ans)
    }

    pub fn apply(&mut self, encoded_peers_info: Bound<'_, PyBytes>) -> AwarenessPeerUpdate {
        let (updated, added) = self.0.apply(encoded_peers_info.as_bytes());
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

#[pyclass(get_all)]
#[derive(Debug, Clone)]
pub struct AwarenessPeerUpdate {
    pub updated: Vec<PeerID>,
    pub added: Vec<PeerID>,
}

#[pyclass(get_all)]
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub state: LoroValue,
    pub counter: i32,
    pub timestamp: i64,
}

#[pyclass]
pub struct EphemeralStore(loro::awareness::EphemeralStore);

#[pymethods]
impl EphemeralStore {
    #[new]
    pub fn new(timeout: i64) -> Self {
        Self(loro::awareness::EphemeralStore::new(timeout))
    }

    pub fn encode(&self, key: &str) -> Vec<u8> {
        self.0.encode(key)
    }

    pub fn encode_all(&self) -> Vec<u8> {
        self.0.encode_all()
    }

    pub fn apply(&mut self, data: &[u8]) -> PyLoroResult<()> {
        self.0.apply(data).map_err(LoroError::DecodeError)?;
        Ok(())
    }

    pub fn set(&mut self, key: &str, value: LoroValue) {
        self.0.set(key, value);
    }

    pub fn delete(&mut self, key: &str) {
        self.0.delete(key);
    }

    pub fn get(&self, key: &str) -> Option<LoroValue> {
        self.0.get(key).map(|x| x.into())
    }

    pub fn remove_outdated(&mut self) {
        self.0.remove_outdated();
    }

    pub fn get_all_states(&self) -> HashMap<String, LoroValue> {
        self.0
            .get_all_states()
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect()
    }

    pub fn keys(&self) -> Vec<String> {
        self.0.keys()
    }

    pub fn subscribe_local_updates(&self, callback: PyObject) -> Subscription {
        let subscription = self.0.subscribe_local_updates(Box::new(move |updates| {
            Python::with_gil(|py| {
                let b = callback.call1(py, (updates,)).unwrap();
                b.extract::<bool>(py).unwrap()
            })
        }));
        subscription.into()
    }

    pub fn subscribe(&self, callback: PyObject) -> Subscription {
        let subscription = self.0.subscribe(Box::new(move |updates| {
            Python::with_gil(|py| {
                let b = callback
                    .call1(
                        py,
                        (EphemeralStoreEvent {
                            by: match updates.by {
                                EphemeralEventTrigger::Local => "Local".to_string(),
                                EphemeralEventTrigger::Import => "Import".to_string(),
                                EphemeralEventTrigger::Timeout => "Timeout".to_string(),
                            },
                            added: updates.added.to_vec(),
                            updated: updates.updated.to_vec(),
                            removed: updates.removed.to_vec(),
                        },),
                    )
                    .unwrap();
                b.extract::<bool>(py).unwrap()
            })
        }));
        subscription.into()
    }
}

#[pyclass(get_all, str)]
#[derive(Debug)]
pub struct EphemeralStoreEvent {
    by: String,
    added: Vec<String>,
    updated: Vec<String>,
    removed: Vec<String>,
}

impl std::fmt::Display for EphemeralStoreEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "EphemeralStoreEvent(by={:?}, added={:?}, updated={:?}, removed={:?})",
            self.by, self.added, self.updated, self.removed
        )
    }
}
