use anyhow::{anyhow, Result};
use sled::{open as open_db, Db};
use std::convert::TryInto;
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum StoreUserError {
    #[error("The provided update hash does not match")]
    InvalidHash,
}

enum SubKey {
    Blob,
    Hash,
}
struct Key(SubKey, String);

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sub_key = match self.0 {
            SubKey::Blob => "blob",
            SubKey::Hash => "hash",
        };

        write!(f, "{}_{}", &self.1, sub_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_tostring() {
        let k: Key = Key(SubKey::Blob, "this_is_my_key".to_owned());
        assert_eq!("blob_this_is_my_key", k.to_string());
    }
}

pub struct Store {
    db: Db,
}

pub type DocumentHash = str;

impl Store {
    pub fn init_from_path(path: &str) -> Result<Store> {
        let db = open_db(path)?;

        Ok(Self { db: db })
    }

    pub fn init_from_db(db: Db) -> Store {
        return Self { db: db };
    }

    pub fn set(
        &self,
        key: &str,
        val: &[u8],
        hash: Option<&DocumentHash>,
        new_hash: &DocumentHash,
    ) -> Result<()> {
        let res = self
            .db
            .transaction::<_, (), StoreUserError>(|db| {
                // Check hash
                let k_hash = &Key(SubKey::Hash, key.to_string()).to_string();
                match (db.get(k_hash)?, hash) {
                    (None, None) => (),
                    (Some(bd_hash), Some(hash)) if bd_hash == hash => (),
                    _ => return sled::transaction::abort(StoreUserError::InvalidHash),
                }

                // Set new hash
                db.insert(&k_hash[..], new_hash.as_bytes())?;

                // Set blob
                let blob_k = &Key(SubKey::Blob, key.to_string()).to_string();
                db.insert(&blob_k[..], val)?;

                Ok(())
            })
            .map_err(|err| match err {
                sled::transaction::TransactionError::Abort(inner_err) => anyhow::anyhow!(inner_err),
                other => anyhow::anyhow!(other),
            })?;

        return Ok(());
    }

    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        // Get blob
        let blob_k = &Key(SubKey::Blob, key.to_string()).to_string();
        let blob = self.db.get(blob_k)?;

        let res = match blob {
            Some(e) => Some(e.to_vec()),
            None => None,
        };

        Ok(res)
    }
}
