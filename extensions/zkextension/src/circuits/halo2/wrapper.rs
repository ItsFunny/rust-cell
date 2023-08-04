use halo2_proofs::circuit::AssignedCell;
use halo2_proofs::pasta::group::ff::PrimeField;

#[derive(Clone)]
pub struct CellWrapper<F: PrimeField> {
    cell: AssignedCell<F, F>,
}

impl<F: PrimeField> CellWrapper<F> {
    pub fn new(cell: AssignedCell<F, F>) -> Self {
        Self { cell }
    }

    pub fn cell(&self) -> AssignedCell<F, F> {
        self.cell.clone()
    }
}
