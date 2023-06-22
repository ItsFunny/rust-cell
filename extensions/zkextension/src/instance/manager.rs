use crate::blocks::block::Block;
use crate::instance::{calculate_contract_id, Contract, Executor, Instance};
use crate::store::trace::{TraceStore, TraceTable};
use crate::store::Store;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet};
use tree::prefix::PrefixWrapper;
use tree::shared::SharedDB;
use tree::tree::Batch;
use tree::tree::{NullHasher, TreeDB, DB};

pub struct InstanceManager {
    instances: BTreeMap<u32, Instance>,
    db: SharedDB<Box<dyn DB>>,
}

impl InstanceManager {
    pub fn register_contract(&mut self, contract: Box<dyn Contract<NullHasher>>) {
        let id = calculate_contract_id(&contract);
        if self.instances.contains_key(&id) {
            panic!("duplicate contract");
        }
        let prefix = id.to_be_bytes().to_vec();
        let db = self.db.clone();
        let prefix: Box<dyn DB> = Box::new(PrefixWrapper::new(prefix, db));
        let store = Box::new(TraceStore::new(prefix));
        let instance = Instance::new(store, contract);
        self.instances.insert(id, instance);
    }
    pub fn start(&mut self) {}
    pub fn handle_a_block(&mut self, block: &Block) {
        for (id, instance) in &mut self.instances {
            let res = instance.begin_block(block);
        }
        for (id, instance) in &mut self.instances {
            let res = instance.execute_block(&block);
        }
        self.db.flush().unwrap();
        self.db.commit(vec![]);
    }

    pub fn generate_block_witness(&mut self) -> BTreeMap<u32, TraceTable<NullHasher>> {
        let mut witness = BTreeMap::new();
        for (id, instance) in &self.instances {
            let traces = instance.get_traces();
            if traces.is_empty() {
                continue;
            }
            witness.insert(*id, traces);
        }
        witness
    }
}
