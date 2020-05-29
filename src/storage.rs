use std::time::{SystemTime, UNIX_EPOCH};
use sled::{open as open_db, Db, Batch};
use anyhow::{Result, anyhow};

pub struct KVStore {
    db: Db,
}

impl KVStore {

    pub fn init(path: &str) -> Result<KVStore> {

        let db = open_db(path)?;

        Ok(Self { db: db })
    }

    pub fn safe_set(&self, old_timestamp: Option<u128> , tree_name: &str, key: &str, val: &[u8]) -> Result<u128> {

        let tree = self.db.open_tree(tree_name)?;
        let mut batch = Batch::default();
       
        let epoch_ms = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .expect("Current timespan not found")
                                .as_millis();

        let ts_k: &str = format!("{}_timestamp", key).as_ref();
        let ts_v = &epoch_ms.to_be_bytes();

        tree.transaction(|tree| {
            let bd_ts = tree.get(ts_k)?;
            let p = match (old_timestamp, bd_ts) {
                (Some(old_ts), Some(bd_ts)) => "throw error", 
                _                                       => "nothing",
            };
           
            tree.insert(ts_k.as_ref(), ts_v)?;
            tree.insert(key, val)?;
            Ok(())

        });

        Ok(epoch_ms);
    }

    pub  fn set(&self, tree_name: &str, key: &str, val: &[u8]) -> Result<u128> {

        let tree = self.db.open_tree(tree_name)?;

        let epoch_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Current timespan not found")
            .as_millis();

        tree.insert(&key, val)?;

        Ok(epoch_ms)
    }

    
    pub fn get(&self, tree_name: &str, key: &str) -> Result<Vec<u8>> {
        
        let tree = self.db.open_tree(tree_name)?;

        let r = match tree.get(key) {
            Ok(Some(res)) => Ok(res.to_vec()),
            Ok(None) => Err(anyhow!("Key not found {}:{}", &tree_name, &key)),
            Err(e) => Err(anyhow!(e))
        };

        r
    }

    pub fn inner_db(&self) -> &Db {
        return &self.db;
    }


}
