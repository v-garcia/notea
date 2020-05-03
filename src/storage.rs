use std::time::{SystemTime, UNIX_EPOCH};
use sled::{open as open_db, Db};
use std::path::Path;
use anyhow::{Result, anyhow};

pub struct KVStore {
    db: Db,
}

impl KVStore {

    pub fn init(path: &Path) -> Result<KVStore> {

        let db = open_db(path)?;

        Ok(Self { db: db })
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


}
