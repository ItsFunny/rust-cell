use merkle_tree::primitives::GetBits;

#[derive(Debug, Clone)]
pub struct Trace {
    pub(crate) operations: Vec<SortOperation>,
}
impl Trace {
    pub fn empty() -> Self {
        Trace { operations: vec![] }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SortOperation {
    index: u64,
    op: Operation,
}

impl GetBits for SortOperation {
    fn get_bits_le(&self) -> Vec<bool> {
        todo!()
    }
}

impl SortOperation {
    pub fn new(index: u64, op: Operation) -> Self {
        Self { index, op }
    }
}

#[derive(Debug, Clone)]
pub enum Operation {
    Set(TypeWrapper),
    Get(TypeWrapper),
    Delete(TypeWrapper),
    Alloc(TypeWrapper),
}

impl Default for Operation {
    fn default() -> Self {
        Operation::Alloc(TypeWrapper::default())
    }
}

#[derive(Debug, Clone)]
pub enum TypeWrapper {
    I128(i128),
    U128(u128),
    I64(i64),
    U64(u64),
    F64(f64),
    Bytes(Vec<u8>),

    Default,
}

impl Default for TypeWrapper {
    fn default() -> Self {
        TypeWrapper::Default
    }
}
