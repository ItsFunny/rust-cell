use crate::common::{map_to_batch, to_batch, to_batch_aux};
use crate::couple::*;
use crate::error::{TreeError, TreeResult};
use crate::operation::{Operation, RollBackOperation};
use crate::tree::{Batch, Read, RootHash, TreeDB, Write, DB, KV};
use merk::chunks::ChunkProducer;
use merk::proofs::Query;
use merk::restore::Restorer;
use merk::tree::Tree;
use merk::{Hash, Merk};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::intrinsics::transmute;
use std::path::{Path, PathBuf};

type Map = BTreeMap<Vec<u8>, Option<Vec<u8>>>;

struct MerkSnapshot {
    _checkpoint: Merk,
    chunks: RefCell<Option<ChunkProducer<'static>>>,
    length: u32,
    hash: Hash,
}

impl MerkSnapshot {
    fn new(checkpoint: Merk) -> TreeResult<Self> {
        let chunks = checkpoint.chunks()?;
        let chunks: ChunkProducer<'static> = unsafe { transmute(chunks) };

        let length = chunks.len() as u32;
        let hash = checkpoint.root_hash();

        Ok(Self {
            _checkpoint: checkpoint,
            chunks: RefCell::new(Some(chunks)),
            length,
            hash,
        })
    }

    fn chunk(&self, index: usize) -> TreeResult<Vec<u8>> {
        let mut chunks = self.chunks.borrow_mut();
        let chunks = chunks.as_mut().unwrap();
        let chunk = chunks.chunk(index)?;
        Ok(chunk)
    }
}

impl Drop for MerkSnapshot {
    fn drop(&mut self) {
        // drop the self-referential ChunkProducer before the Merk instance
        self.chunks.borrow_mut().take();
    }
}

pub struct MerkleRocksDB {
    m: Option<Merk>,
    map: Option<Map>,
    snapshots: BTreeMap<u64, MerkSnapshot>,
    restorer: Option<Restorer>,
    configuration: MerkleRocksDBConfiguration,
}

#[derive(Clone)]
pub struct MerkleRocksDBConfiguration {
    pub(crate) snapshot_interval: u64,
    pub(crate) snapshot_limit: u64,
    pub(crate) home: PathBuf,
}

impl MerkleRocksDBConfiguration {
    pub fn new<P: AsRef<Path>>(home: P) -> Self {
        let home = home.as_ref().to_path_buf();
        Self {
            snapshot_interval: 1,
            snapshot_limit: 1000,
            home,
        }
    }
    pub fn get_home(&self) -> PathBuf {
        self.home.clone()
    }
}

unsafe impl Send for MerkleRocksDB {}

unsafe impl Sync for MerkleRocksDB {}

impl MerkleRocksDB {
    pub fn new(cfg: MerkleRocksDBConfiguration) -> Self {
        let home = cfg.clone().home;
        let merk = Merk::open(merk_db_path(&home)).unwrap();

        maybe_remove_restore(&home).expect("Failed to remove incomplete state sync restore");

        let snapshot_path = home.join("snapshots");

        let no_snapshot = !snapshot_path.exists();
        if no_snapshot {
            std::fs::create_dir(&snapshot_path).expect("Failed to create 'snapshots' directory");
        }

        let snapshots = load_snapshots(&home)
            .map_err(|e| e)
            .expect("Failed to load snapshots");

        Self {
            m: Some(merk),
            map: Some(Default::default()),
            snapshots,
            restorer: None,
            configuration: cfg.clone(),
        }
    }
}

pub fn merk_db_path<P: AsRef<Path> + ?Sized>(home: &P) -> PathBuf {
    let home = home.as_ref().to_path_buf();
    home.join("merk_db")
}

pub fn restore_path<P: AsRef<Path> + ?Sized>(home: &P) -> PathBuf {
    let home = home.as_ref().to_path_buf();
    home.join("restore")
}

pub fn snapshot_path<P: AsRef<Path> + ?Sized>(home: &P) -> PathBuf {
    let home = home.as_ref().to_path_buf();
    home.join("snapshots")
}

fn maybe_remove_restore(home: &Path) -> TreeResult<()> {
    let restore_path = restore_path(home);
    if restore_path.exists() {
        std::fs::remove_dir_all(&restore_path).map_err(|_e| TreeError::Unknown)?;
    }

    Ok(())
}

fn load_snapshots(home: &Path) -> TreeResult<BTreeMap<u64, MerkSnapshot>> {
    let mut snapshots = BTreeMap::new();

    let snapshot_dir = snapshot_path(home)
        .read_dir()
        .map_err(|_e| TreeError::Unknown)?;
    for entry in snapshot_dir {
        let entry = entry.map_err(|_e| TreeError::Unknown)?;
        let path = entry.path();

        // TODO: open read-only
        let checkpoint = Merk::open(&path)?;
        let snapshot = MerkSnapshot::new(checkpoint)?;

        let height_str = path.file_name().unwrap().to_str().unwrap();
        let height: u64 = height_str.parse().map_err(|_e| TreeError::Unknown)?;
        snapshots.insert(height, snapshot);
    }

    Ok(snapshots)
}

impl Read for MerkleRocksDB {
    fn get(&self, k: &[u8]) -> TreeResult<Option<Vec<u8>>> {
        match self.map.as_ref().unwrap().get(k) {
            Some(Some(value)) => Ok(Some(value.clone())),
            Some(None) => Ok(None),
            None => {
                let ret = self
                    .m
                    .as_ref()
                    .unwrap()
                    .get(k)
                    .map_err(|e| TreeError::from(e))?;
                match ret {
                    Some(v) => Ok(Some(v.clone())),
                    None => Ok(None),
                }
            }
        }
    }

    fn get_next(&self, start: &[u8]) -> TreeResult<Option<KV>> {
        let mut iter = self.m.as_ref().unwrap().raw_iter();
        iter.seek(start);

        if !iter.valid() {
            iter.status().map_err(|e| TreeError::from(e))?;
            return Ok(None);
        }

        if iter.key().unwrap() == start {
            iter.next();

            if !iter.valid() {
                iter.status()?;
                return Ok(None);
            }
        }

        let key = iter.key().unwrap();
        let tree_bytes = iter.value().unwrap();
        let tree = Tree::decode(vec![], tree_bytes);
        let value = tree.value();
        Ok(Some((key.to_vec(), value.to_vec())))
    }
}

impl Write for MerkleRocksDB {
    fn set(&mut self, k: Vec<u8>, v: Vec<u8>) -> TreeResult<Vec<u8>> {
        self.map.as_mut().unwrap().insert(k.clone(), Some(v));
        Ok(k.clone())
        // self.m.as_mut().unwrap().apply(&vec![(k.clone(), Op::Put(v))], &[]).map_err(|e| {
        //     TreeError::from(e)
        // }).map(|_| {
        //     k.clone()
        // })
    }

    fn delete(&mut self, k: Vec<u8>) -> TreeResult<()> {
        self.map.as_mut().unwrap().insert(k, None);
        Ok(())
    }
}

impl Batch for MerkleRocksDB {
    fn commit(&mut self, mut operations: Vec<Operation>) -> RootHash {
        let map = self.map.take().unwrap();
        self.map = Some(Map::new());
        operations.extend(map_to_batch(map));

        let mut height = self.height().expect("fail to get");
        height = height + 1;
        let height_bytes = height.to_be_bytes();
        let metadata = vec![(b"height".to_vec(), Some(height_bytes.to_vec()))];
        self.batch_operation(operations, metadata)
            .expect("TODO: panic message");
        let merk = self.m.as_mut().unwrap();
        merk.flush().expect("TODO: panic message");

        let ret = merk.root_hash();
        // TODO,panic
        self.maybe_create_snapshot(height)
            .expect("fail to create snapshot");

        return ret;
    }
}

impl DB for MerkleRocksDB {
    fn get_configuration(&self) -> MerkleRocksDBConfiguration {
        self.configuration.clone()
    }

    fn roll_back_with_operation(&mut self, ops: RollBackOperation) -> TreeResult<()> {
        let mut borrow = true;
        match ops {
            RollBackOperation::MerkleHeight(h, op) => {
                if let Some(v) = op {
                    borrow = v;
                }
                self.roll_back_from_height(h, borrow)
            }
        }
    }

    fn height(&self) -> TreeResult<u64> {
        self.m
            .as_ref()
            .unwrap()
            .get_aux(b"height")
            .map_err(|e| TreeError::from(e))?
            .map_or_else(|| Ok(0), |v| Ok(read_u64(v.as_slice())))
    }

    fn clean(&mut self) -> TreeResult<()> {
        self.map = Some(Map::new());
        Ok(())
    }

    fn flush(&mut self) -> TreeResult<()> {
        Ok(())
    }
}

impl TreeDB for MerkleRocksDB {
    fn prove(&self, req_enums: ProveRequestEnums) -> TreeResult<ProveResponseEnums> {
        if let ProveRequestEnums::SimpleProve(req) = req_enums {
            let mut q = Query::default();
            for k in req.query {
                q.insert_key(k.clone());
            }
            self.m
                .as_ref()
                .unwrap()
                .prove(q)
                .map(|v| ProveResponseEnums::SimpleProve(SimpleProveResponse { proof: v }))
                .map_err(|e| TreeError::from(e))
        } else {
            unreachable!()
        }
    }

    fn verify(&self, req_enums: VerifyRequestEnums) -> TreeResult<VerifyResponse> {
        if let VerifyRequestEnums::SimpleVerify(req) = req_enums {
            let res = merk::verify(req.proof.as_slice(), req.expected_root as Hash)
                .map_err(|e| TreeError::from(e))?;
            // TODO,ics 23
            let mut ret = VerifyResponse::default();
            for (k, v) in req.kv {
                let value_opt = res.get(k.as_slice()).map_err(|e| TreeError::from(e))?;
                if let Some(value) = value_opt {
                    if value != v {
                        ret.valid = false;
                        return Ok(ret);
                    }
                }
            }
            ret.valid = true;
            Ok(ret)
        } else {
            unreachable!()
        }
    }

    fn root_hash(&self) -> RootHash {
        self.m.as_ref().unwrap().root_hash() as RootHash
    }
}

impl MerkleRocksDB {
    pub fn get_aux(&self, key: &[u8]) -> TreeResult<Option<Vec<u8>>> {
        self.m
            .as_ref()
            .unwrap()
            .get_aux(key)
            .map_err(|e| TreeError::from(e))
    }
    pub fn roll_back_from_height(&mut self, h: u64, borrow: bool) -> TreeResult<()> {
        let restore_path = restore_path(&self.configuration.home);
        let snapshot_opt = self.snapshots.get(&h);
        if let None = snapshot_opt {
            panic!("fail to roll back");
        }
        let snapshot = { snapshot_opt.unwrap() };
        let len = snapshot.length;

        {
            let restorer = Restorer::new(&restore_path, snapshot.hash, len as usize)?;
            self.restorer = Some(restorer);
        }

        let restorer = self.restorer.as_mut().unwrap();
        let mut chunk_datas = Vec::new();
        for i in 0..len {
            let data = snapshot.chunk(i as usize)?;
            chunk_datas.extend(data);
        }
        let chunks_remaining = restorer
            .process_chunk(chunk_datas.as_slice())
            .map_err(|_e| TreeError::Unknown)?;
        if chunks_remaining == 0 {
            let restored = self.restorer.take().unwrap().finalize()?;
            self.m
                .take()
                .unwrap()
                .destroy()
                .map_err(|_e| TreeError::Unknown)?;
            let db_path = merk_db_path(&self.configuration.home);
            drop(restored);

            std::fs::rename(&restore_path, &db_path).map_err(|_e| TreeError::Unknown)?;

            self.m = Some(Merk::open(db_path).map_err(|_e| TreeError::Unknown)?);

            let height_bytes = h.to_be_bytes().to_vec();
            let metadata = vec![(b"height".to_vec(), Some(height_bytes))];
            self.batch_operation(vec![], metadata)?;
            self.m
                .as_mut()
                .unwrap()
                .flush()
                .map_err(|_e| TreeError::Unknown)?;
            if !borrow {
                self.m.take();
                loop {
                    match self.snapshots.pop_first() {
                        Some(v) => drop(v.1),
                        None => {
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn batch_operation(
        &mut self,
        ops: Vec<Operation>,
        mut aux: Vec<(Vec<u8>, Option<Vec<u8>>)>,
    ) -> TreeResult<()> {
        let batches = to_batch(ops);
        aux.extend(batches.1);
        let mut aux_batch = to_batch_aux(aux);
        aux_batch.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let mut batc = batches.0;
        batc.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        self.m
            .as_mut()
            .unwrap()
            .apply(&batc, &aux_batch)
            .map_err(|e| TreeError::from(e))
    }
    fn maybe_create_snapshot(&mut self, height: u64) -> TreeResult<()> {
        if self.configuration.snapshot_interval == 0 {
            return Ok(());
        }
        if height == 0 || (height % self.configuration.snapshot_interval != 0) {
            return Ok(());
        }
        if self.snapshots.contains_key(&height) {
            return Ok(());
        }

        let path = self.snapshot_path(height);
        let checkpoint = self.m.as_mut().unwrap().checkpoint(path)?;

        let snapshot = MerkSnapshot::new(checkpoint)?;
        self.snapshots.insert(height, snapshot);

        self.maybe_prune_snapshots()
    }
    fn snapshot_path(&self, height: u64) -> PathBuf {
        snapshot_path(&self.configuration.home).join(height.to_string())
    }

    fn path<T: ToString>(&self, name: T) -> PathBuf {
        self.configuration.home.join(name.to_string())
    }

    fn maybe_prune_snapshots(&mut self) -> TreeResult<()> {
        let height = self.height()?;
        if height <= self.configuration.snapshot_interval * self.configuration.snapshot_limit {
            return Ok(());
        }

        let remove_height =
            height - self.configuration.snapshot_interval * self.configuration.snapshot_limit;
        self.snapshots.remove(&remove_height);

        let path = self.snapshot_path(remove_height);
        if path.exists() {
            std::fs::remove_dir_all(path).map_err(|_e| TreeError::Unknown)?;
        }

        Ok(())
    }
}

pub(crate) fn read_u64(bytes: &[u8]) -> u64 {
    let mut array = [0; 8];
    array.copy_from_slice(bytes);
    u64::from_be_bytes(array)
}

#[cfg(test)]
mod test {
    use crate::merkle::{MerkleRocksDB, MerkleRocksDBConfiguration};
    use crate::tree::{Batch, Read, Write, DB};

    #[test]
    pub fn test_roll_back() {
        let mut db = MerkleRocksDB::new(MerkleRocksDBConfiguration::new("./merk.db"));
        let key = b"dog";
        let value = b"cat";
        {
            db.set(key.to_vec(), value.to_vec()).expect("fail to set");
            db.commit(vec![]);
            let h = db.height().expect("fail to get height");
            assert_eq!(h, 1);
            let data = db.get(key).expect("fail to get").unwrap();
            assert_eq!(data.as_slice(), value);
        }

        {
            let value2 = b"aaa";
            db.set(key.to_vec(), value2.to_vec()).expect("fail to set");
            db.commit(vec![]);
            {
                let h = db.height().expect("fail to get height");
                assert_eq!(h, 2);
                let data = db.get(key).expect("fail to get").unwrap();
                assert_eq!(data.as_slice(), value2);
            }
        }

        {
            let value = b"ccc";
            db.set(key.to_vec(), value.to_vec()).expect("fail to set");
            db.commit(vec![]);
            {
                let h = db.height().expect("fail to get height");
                assert_eq!(h, 3);
                let data = db.get(key).expect("fail to get").unwrap();
                assert_eq!(data.as_slice(), value);
            }
        }

        {
            let value = b"ddd";
            db.set(key.to_vec(), value.to_vec()).expect("fail to set");
            db.commit(vec![]);
            {
                let h = db.height().expect("fail to get height");
                assert_eq!(h, 4);
                let data = db.get(key).expect("fail to get").unwrap();
                assert_eq!(data.as_slice(), value);
            }
        }

        db.roll_back_from_height(1, true).expect("fail to rollback");

        let ret = db.get(key).expect("fail to get").unwrap();
        println!("{:?}", String::from_utf8(ret.clone()));
        assert_eq!(value, ret.clone().as_slice());

        std::fs::remove_dir_all(db.get_configuration().clone().home).unwrap();
    }
}
