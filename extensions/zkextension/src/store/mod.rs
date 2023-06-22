pub mod trace;
use crate::store::trace::TraceTable;
use tree::tree::{KeyHasher, TreeDB, DB};

pub trait Store<K: KeyHasher>: DB {
    fn get_traces(&self) -> TraceTable<K>;
}
