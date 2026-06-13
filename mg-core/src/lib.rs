use std::collections::HashMap;
use std::time::Instant;
use bitcoin::{Txid, Amount};
use bdk_chain::tx_graph::TxGraph;
use serde::{Serialize, Deserialize};

#[derive (Debug,Clone)]
pub struct TxEntry {
    pub txid: Txid,
    pub fee_rate:f64, 
    pub vsize: u32,
    pub fee: Amount,
    pub first_seen: Instant,
}
#[derive (Debug, Clone, Default)]
pub struct MempoolStats{
    pub tx_count: usize,
    pub total_vsize:u64,
    pub p50_fee:f64,
}

#[derive(Debug, Default)]
pub struct MempoolState {
    pub txs: HashMap<Txid, TxEntry>,
    pub graph: TxGraph,
    pub stats: MempoolStats,
}
 
impl MempoolState{
    pub fn new() -> Self{
        Self{
            txs: HashMap::new(),
            graph: TxGraph::default(),
            stats: MempoolStats::default(),
        }
    }
    pub fn add_tx(&mut self, entry: TxEntry, tx: bitcoin::Transaction) {
        self.txs.insert(entry.txid, entry);
        let _ = self.graph.insert_tx(tx);
        self.update_stats();
    }
    pub fn remove_tx(&mut self, txid: &Txid){
        self.txs.remove(txid);
        self.update_stats();
    }
    fn update_stats(&mut self){
        self.stats.tx_count = self.txs.len();
        self.stats.total_vsize = self.txs.values().map(|e| e.vsize as u64).sum();

    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MempoolEvent {
        TxAdded{
            txid: Txid,
            fee_rate: f64,
            vsize: u32,
        },
        TxRemove(Txid),
        BlockArrived{
            height:u32,
            confirmed: Vec<Txid>,
        },
        
        
    }
