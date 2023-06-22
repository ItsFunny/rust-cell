use crate::common::{
    map_to_batch, maybe_remove_restore, merk_db_path, snapshot_path, to_batch, to_batch_aux, Map,
};
use crate::error::{TreeError, TreeResult};
use crate::merkle::{read_u64, MerkleRocksDBConfiguration};
use crate::operation::{Operation, RollBackOperation};
use crate::tree::{Batch, Read, RootHash, Write, DB, KV};

use merk::rocksdb::checkpoint::Checkpoint;
use merk::rocksdb::{ColumnFamilyDescriptor, WriteBatch};

use merk::{rocksdb, Op};

use std::collections::BTreeMap;

use std::path::{Path, PathBuf};

const AUX_CF_NAME: &str = "aux";
const INTERNAL_CF_NAME: &str = "internal";

fn column_families() -> Vec<ColumnFamilyDescriptor> {
    vec![
        // TODO: clone opts or take args
        ColumnFamilyDescriptor::new(AUX_CF_NAME, RocksDB::default_db_opts()),
        ColumnFamilyDescriptor::new(INTERNAL_CF_NAME, RocksDB::default_db_opts()),
    ]
}

pub(crate) fn load_snapshots(home: &Path) -> TreeResult<BTreeMap<u64, PathBuf>> {
    let mut snapshots = BTreeMap::new();

    let snapshot_dir = snapshot_path(home)
        .read_dir()
        .map_err(|_e| TreeError::Unknown)?;
    for entry in snapshot_dir {
        let entry = entry.map_err(|_e| TreeError::Unknown)?;
        let path = entry.path();

        // TODO: open read-only
        // let checkpoint = RocksDB::open(&path);

        let height_str = path.file_name().unwrap().to_str().unwrap();
        let height: u64 = height_str.parse().map_err(|_e| TreeError::Unknown)?;
        snapshots.insert(height, path);
    }

    Ok(snapshots)
}

pub struct RocksDB {
    db: Option<rocksdb::DB>,
    map: Option<Map>,
    configuration: MerkleRocksDBConfiguration,
    snapshots: BTreeMap<u64, PathBuf>,
}

unsafe impl Send for RocksDB {}

unsafe impl Sync for RocksDB {}

impl RocksDB {
    fn open<P: AsRef<Path>>(p: P) -> Self {
        let cfg = MerkleRocksDBConfiguration::new(p);
        let db_opts = RocksDB::default_db_opts();
        RocksDB::open_opt(cfg, false, db_opts).expect("fail to open")
    }
    pub fn new(cfg: MerkleRocksDBConfiguration) -> Self {
        let db_opts = RocksDB::default_db_opts();
        let mut db = RocksDB::open_opt(cfg, false, db_opts).expect("fail");
        let home = db.configuration.clone().home;
        maybe_remove_restore(&home).expect("Failed to remove incomplete state sync restore");
        let snapshot_path = snapshot_path(&home);
        let no_snapshot = !snapshot_path.exists();
        if no_snapshot {
            std::fs::create_dir(&snapshot_path).expect("Failed to create 'snapshots' directory");
        }

        let snapshots = load_snapshots(&home)
            .map_err(|e| e)
            .expect("Failed to load snapshots");
        db.snapshots = snapshots;
        db
    }

    pub fn default_db_opts() -> rocksdb::Options {
        let mut opts = rocksdb::Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_atomic_flush(true);

        // TODO: tune
        opts.increase_parallelism(num_cpus::get() as i32);
        // opts.set_advise_random_on_open(false);
        opts.set_allow_mmap_writes(true);
        opts.set_allow_mmap_reads(true);

        opts.set_max_log_file_size(1_000_000);
        opts.set_recycle_log_file_num(5);
        opts.set_keep_log_file_num(5);
        opts.set_log_level(rocksdb::LogLevel::Warn);

        opts
    }

    fn open_opt(
        cfg: MerkleRocksDBConfiguration,
        checkpoint: bool,
        db_opts: rocksdb::Options,
    ) -> TreeResult<RocksDB> {
        let home = cfg.clone().home;
        let mut path = home;
        if !checkpoint {
            path = merk_db_path(&path);
        }
        let db = RocksDB::new_rocksdb(path, db_opts)?;
        Ok(RocksDB {
            db: Some(db),
            map: Some(Default::default()),
            configuration: cfg.clone(),
            snapshots: Default::default(),
        })
    }
    fn new_rocksdb(path: PathBuf, db_opts: rocksdb::Options) -> TreeResult<rocksdb::DB> {
        let res = rocksdb::DB::open_cf_descriptors(&db_opts, path, column_families())?;
        return Ok(res);
    }
}

impl Read for RocksDB {
    fn get(&self, k: &[u8]) -> TreeResult<Option<Vec<u8>>> {
        match self.map.as_ref().unwrap().get(k) {
            Some(Some(value)) => Ok(Some(value.clone())),
            Some(None) => Ok(None),
            None => {
                let res = self.db.as_ref().unwrap().get(k)?;
                return Ok(res);
            }
        }
    }

    fn get_next(&self, start: &[u8]) -> TreeResult<Option<KV>> {
        let mut iter = self.db.as_ref().unwrap().raw_iterator();
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
        let value = iter.value().unwrap();
        Ok(Some((key.to_vec(), value.to_vec())))
    }
}

impl Write for RocksDB {
    fn set(&mut self, k: Vec<u8>, v: Vec<u8>) -> TreeResult<Vec<u8>> {
        self.map.as_mut().unwrap().insert(k.clone(), Some(v));
        Ok(k.clone())
    }

    fn delete(&mut self, k: Vec<u8>) -> TreeResult<()> {
        self.map.as_mut().unwrap().insert(k, None);
        Ok(())
    }
}

impl DB for RocksDB {
    fn get_configuration(&self) -> MerkleRocksDBConfiguration {
        self.configuration.clone()
    }

    fn roll_back_with_operation(&mut self, ops: RollBackOperation) -> TreeResult<()> {
        match ops {
            RollBackOperation::MerkleHeight(u, b) => {
                let borrow = b.unwrap_or(true);
                self.do_roll_back(u, borrow)
            }
        }
    }
    fn height(&self) -> TreeResult<u64> {
        let db = self.db.as_ref().unwrap();
        let aux_cf = db.cf_handle(AUX_CF_NAME);
        db.get_cf(aux_cf.unwrap(), b"height")
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

impl Batch for RocksDB {
    fn commit(&mut self, mut ops: Vec<Operation>) -> RootHash {
        let map = self.map.take().unwrap();
        self.map = Some(Map::new());
        ops.extend(map_to_batch(map));

        {
            let mut height = self.height().expect("fail to get");
            height = height + 1;
            let height_bytes = height.to_be_bytes();
            let metadata = vec![(b"height".to_vec(), Some(height_bytes.to_vec()))];
            ops.push(Operation::Aux(metadata));
        }

        let mut batch = rocksdb::WriteBatch::default();
        let db = self.db.as_mut().unwrap();
        let aux_cf = db.cf_handle(AUX_CF_NAME).unwrap();

        let batches = to_batch(ops);
        let mut aux = to_batch_aux(batches.1);
        aux.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let mut batc = batches.0;
        batc.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        for (key, maybe_value) in batc {
            match maybe_value {
                Op::Put(value) => {
                    batch.put(key, value);
                }
                Op::Delete => {
                    batch.delete(key);
                }
            }
        }

        for (key, value) in aux {
            match value {
                Op::Put(value) => batch.put_cf(aux_cf, key, value),
                Op::Delete => batch.delete_cf(aux_cf, key),
            };
        }

        self.write(batch, true).expect("fail to write");
        // TODO
        Default::default()
    }
}

impl RocksDB {
    fn write(&mut self, batch: WriteBatch, snapshot: bool) -> TreeResult<()> {
        let mut opts = rocksdb::WriteOptions::default();
        opts.set_sync(false);
        // TODO: disable WAL once we can ensure consistency with transactions
        let db = self.db.as_mut().unwrap();
        db.write_opt(batch, &opts)?;
        db.flush()?;
        if snapshot {
            let h = self.height().unwrap();
            self.maybe_create_snapshot(h)?;
        }
        Ok(())
    }

    fn maybe_create_snapshot(&mut self, height: u64) -> TreeResult<()> {
        if self.configuration.snapshot_interval == 0 {
            return Ok(());
        }
        if height == 0 || (height % self.configuration.snapshot_interval != 0) {
            return Ok(());
        }
        self.do_create_snapshot(height)
    }
    fn do_create_snapshot(&mut self, height: u64) -> TreeResult<()> {
        if self.snapshots.contains_key(&height) {
            return Ok(());
        }

        let path = self.snapshot_path(height);
        let _checkpoint = self.checkpoint(path.clone())?;
        // let snapshot = RocksDbSnapshot::new(checkpoint)?;
        self.snapshots.insert(height, path.clone());

        self.maybe_prune_snapshots()
    }

    fn maybe_prune_snapshots(&mut self) -> TreeResult<()> {
        let height = 1;
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

    fn snapshot_path(&self, height: u64) -> PathBuf {
        snapshot_path(&self.configuration.home).join(height.to_string())
    }

    pub fn checkpoint<P: AsRef<Path>>(&mut self, path: P) -> TreeResult<RocksDB> {
        let db = self.db.as_mut().unwrap();
        let ck = Checkpoint::new(db)?;
        ck.create_checkpoint(&path)?;
        let mut cfg = self.configuration.clone();
        cfg.home = path.as_ref().to_path_buf();
        let db_opts = RocksDB::default_db_opts();
        Ok(RocksDB::open_opt(cfg, true, db_opts)?)
    }

    fn do_roll_back(&mut self, h: u64, borrow: bool) -> TreeResult<()> {
        let snapshot_opt = self.snapshots.remove(&h);
        if let None = snapshot_opt {
            return Err(TreeError::Unknown);
        }
        self.destory()?;

        self.replace(h, snapshot_opt.unwrap())?;

        self.do_create_snapshot(h)?;
        let height_bytes = h.to_be_bytes().to_vec();
        let metadata = vec![(b"height".to_vec(), Some(height_bytes))];
        // TODO: rename firstly or metadata firstly
        self.write_metadata(metadata)?;
        if !borrow {
            let db = self.db.take().unwrap();
            drop(db);
        }
        Ok(())
    }
    fn replace(&mut self, _h: u64, path: PathBuf) -> TreeResult<()> {
        let after_path = path;
        let origin_path = merk_db_path(&self.configuration.clone().home);
        std::fs::rename(&after_path, &origin_path).map_err(|_e| TreeError::Unknown)?;

        let db = RocksDB::new_rocksdb(origin_path, RocksDB::default_db_opts())?;
        self.db.replace(db);
        self.snapshots = load_snapshots(&self.configuration.clone().home)?;

        Ok(())
    }
    fn destory(&mut self) -> TreeResult<()> {
        let origin = self.db.take().unwrap();
        let opts = RocksDB::default_db_opts();
        let path = merk_db_path(&self.configuration.clone().home);
        drop(origin);
        rocksdb::DB::destroy(&opts, path)?;
        Ok(())
    }
    fn write_metadata(&mut self, meta: Vec<(Vec<u8>, Option<Vec<u8>>)>) -> TreeResult<()> {
        let mut batch = rocksdb::WriteBatch::default();
        let aux = to_batch_aux(meta);
        let db = self.db.as_mut().unwrap();
        let aux_cf = db.cf_handle(AUX_CF_NAME).unwrap();
        for (key, value) in aux {
            match value {
                Op::Put(value) => batch.put_cf(aux_cf, key, value),
                Op::Delete => batch.delete_cf(aux_cf, key),
            };
        }
        self.write(batch, false)
    }
}
