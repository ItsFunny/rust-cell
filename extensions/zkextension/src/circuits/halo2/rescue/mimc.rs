use halo2_proofs::circuit::Chip;
use halo2_proofs::pasta::group::ff::PrimeField;
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct MiMCConfig {}

pub struct MIMCChip<F: PrimeField> {
    _ph: PhantomData<F>,
}

impl<F: PrimeField> Chip<F> for MIMCChip<F> {
    type Config = MiMCConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        todo!()
    }

    fn loaded(&self) -> &Self::Loaded {
        todo!()
    }
}
