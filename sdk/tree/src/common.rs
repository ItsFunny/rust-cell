use crate::error::{TreeError, TreeResult};
use crate::operation::Operation;

use merk::{BatchEntry, Op};

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(crate) type Map = BTreeMap<Vec<u8>, Option<Vec<u8>>>;

pub(crate) fn map_to_batch<I: IntoIterator<Item = (Vec<u8>, Option<Vec<u8>>)>>(
    map: I,
) -> Vec<Operation> {
    let mut operations = vec![];
    for (k, v) in map {
        match v.clone() {
            Some(value) => operations.push(Operation::Set(k, value)),
            None => operations.push(Operation::Delete(k)),
        }
    }
    return operations;
}

pub(crate) fn to_batch(ops: Vec<Operation>) -> (Vec<BatchEntry>, Vec<(Vec<u8>, Option<Vec<u8>>)>) {
    let mut batch = Vec::new();
    let mut aux_info = Vec::default();
    for val in ops {
        match val {
            Operation::Set(k, v) => batch.push((k, Op::Put(v))),
            Operation::Delete(k) => batch.push((k, Op::Delete)),
            Operation::Aux(aux) => {
                aux_info.extend(aux);
            }
        }
    }
    (batch, aux_info)
}

pub(crate) fn to_batch_aux<I: IntoIterator<Item = (Vec<u8>, Option<Vec<u8>>)>>(
    i: I,
) -> Vec<BatchEntry> {
    let mut batch = Vec::new();
    for (key, val) in i {
        match val {
            Some(val) => batch.push((key, Op::Put(val))),
            None => batch.push((key, Op::Delete)),
        }
    }
    batch
}

pub(crate) fn merk_db_path<P: AsRef<Path> + ?Sized>(home: &P) -> PathBuf {
    let home = home.as_ref().to_path_buf();
    home.join("merk_db")
}

pub(crate) fn restore_path<P: AsRef<Path> + ?Sized>(home: &P) -> PathBuf {
    let home = home.as_ref().to_path_buf();
    home.join("restore")
}

pub(crate) fn snapshot_path<P: AsRef<Path> + ?Sized>(home: &P) -> PathBuf {
    let home = home.as_ref().to_path_buf();
    home.join("snapshots")
}

pub(crate) fn maybe_remove_restore(home: &Path) -> TreeResult<()> {
    let restore_path = restore_path(home);
    if restore_path.exists() {
        std::fs::remove_dir_all(&restore_path).map_err(|_e| TreeError::Unknown)?;
    }

    Ok(())
}
