use merkle_tree::primitives::GetBits;

pub enum TraceEnum {
    Write(WriteTrace),
    Read(ReadTrace),
}
pub enum WriteTrace {}
pub enum ReadTrace {}

pub struct Instance {}

pub trait IndexAble {
    fn get_index(&self) -> u32;
}
pub struct TraceHistory {
    pub traces: Vec<Box<dyn IndexAble>>,
}
impl GetBits for Instance {
    fn get_bits_le(&self) -> Vec<bool> {
        todo!()
    }
}
