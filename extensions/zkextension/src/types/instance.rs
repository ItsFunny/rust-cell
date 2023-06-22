use merkle_tree::primitives::GetBits;

pub struct Instance {}
impl GetBits for Instance {
    fn get_bits_le(&self) -> Vec<bool> {
        todo!()
    }
}
