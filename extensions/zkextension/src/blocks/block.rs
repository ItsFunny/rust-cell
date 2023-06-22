use crate::blocks::transaction::Transaction;

pub struct Block {
    pub transactions: Vec<Box<dyn Transaction>>,
}
