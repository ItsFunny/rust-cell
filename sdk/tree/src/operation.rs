pub enum Operation {
    Set(Vec<u8>, Vec<u8>),
    Delete(Vec<u8>),
    Aux(Vec<(Vec<u8>, Option<Vec<u8>>)>),
}

pub enum RollBackOperation {
    // TODO
    MerkleHeight(u64, Option<bool>),
}

impl RollBackOperation {
    pub fn get_height(&self) -> Option<u64> {
        return match self {
            RollBackOperation::MerkleHeight(h, _v) => Some(*h),
        };
    }
}
