use bdk_chain::tx_graph::TxGraph;
use bitcoin::{Amount, Txid};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct TxEntry {
    pub txid: Txid,
    pub fee_rate: f64,
    pub vsize: u32,
    pub fee: Amount,
    pub first_seen: Instant,
}
#[derive(Debug, Clone, Default)]
pub struct MempoolStats {
    pub tx_count: usize,
    pub total_vsize: u64,
    pub p50_fee: f64,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum ConnectionStatus {
    #[default]
    Connecting,
    Connected,
    Failed,
}

#[derive(Debug, Default)]
pub struct MempoolState {
    pub txs: HashMap<Txid, TxEntry>,
    pub particles: Vec<Particle>,
    pub graph: TxGraph,
    pub stats: MempoolStats,
    pub screen_size: (f32, f32),
    pub connection_status: ConnectionStatus,
}
#[derive(Debug, Clone)]
pub struct Particle {
    pub pos: (f32, f32),
    pub velocity: (f32, f32),
    pub fee_rate: f32,
}

impl MempoolState {
    pub fn new() -> Self {
        Self {
            txs: HashMap::new(),
            particles: Vec::default(),
            graph: TxGraph::default(),
            stats: MempoolStats::default(),
            screen_size: (120.0, 40.0),
            connection_status: ConnectionStatus::Connecting,
        }
    }

    pub fn add_tx(&mut self, entry: TxEntry, tx: bitcoin::Transaction) {
        self.txs.insert(entry.txid, entry);
        let _ = self.graph.insert_tx(tx);
        self.update_stats();
    }

    pub fn remove_tx(&mut self, txid: &Txid) {
        self.txs.remove(txid);
        self.update_stats();
    }

    fn update_stats(&mut self) {
        self.stats.tx_count = self.txs.len();
        self.stats.total_vsize = self.txs.values().map(|e| e.vsize as u64).sum();
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MempoolEvent {
    TxAdded {
        txid: Txid,
        fee_rate: f64,
        vsize: u32,
    },
    TxRemove(Txid),
    BlockArrived {
        height: u32,
        confirmed: Vec<Txid>,
    },
}
