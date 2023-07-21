use halo2_proofs::circuit::AssignedCell;
use halo2_proofs::pasta::group::ff::PrimeField;

pub struct CellWrapper<F: PrimeField> {
    pub cell: AssignedCell<F, F>,
}
