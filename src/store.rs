use anyhow::{anyhow, Result};
use sled::{open as open_db, Db};
use std::fmt;
use thiserror::Error;
use std::convert::TryInto;

#[derive(Error, Debug, PartialEq)]
pub enum StoreUserError {
    #[error("The provided update revision does not match")]
    BadRevision,
    #[error("Invalid owner key")]
    InvalidOwnerKey,
}

enum SubKey {
    OwnerKey,
    Blob,
    LastRev,
}
struct Key(SubKey, String);

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sub_key = match self.0 {
            SubKey::OwnerKey => "owner_key",
            SubKey::Blob => "blob",
            SubKey::LastRev => "last_rev",
        };

        write!(f, "{}_{}", sub_key, &self.1)
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

pub type DocumentRev = u32;

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
        rev: DocumentRev,
        maybe_owner_key: Option<&[u8]>,
    ) -> Result<DocumentRev> {
        let res = self
            .db
            .transaction::<_, DocumentRev, StoreUserError>(|db| {
                // Check rev
                let k_last_rev = &Key(SubKey::LastRev, key.to_string()).to_string();
                match (db.get(k_last_rev)?, rev) {
                    (Some(bd_rev), rev) if bd_rev == rev.to_be_bytes() => (),
                    (None, 0) => (),
                    _ => return sled::transaction::abort(StoreUserError::BadRevision),
                }

                // Set new rev
                let new_rev = rev + 1;
                db.insert(&k_last_rev[..], &new_rev.to_be_bytes())?;

                // Check owner key
                let k_owner_k = &Key(SubKey::OwnerKey, key.to_string()).to_string();
                let maybe_bd_owner_key = db.get(k_owner_k)?;
                match (&maybe_bd_owner_key, &maybe_owner_key) {
                    (Some(bd_owner_key), Some(owner_key)) if bd_owner_key == owner_key => (),
                    (None, _) => (),
                    _ => return sled::transaction::abort(StoreUserError::InvalidOwnerKey),
                }

                // Set owner key if necessary
                if let (None, Some(owner_key)) = (&maybe_bd_owner_key, &maybe_owner_key) {
                    db.insert(&k_owner_k[..], *owner_key)?;
                }

                // Set blob
                let blob_k = &Key(SubKey::Blob, key.to_string()).to_string();
                db.insert(&blob_k[..], val)?;

                Ok(new_rev)
            })
            .map_err(|err| match err {
                sled::transaction::TransactionError::Abort(inner_err) => anyhow::anyhow!(inner_err),
                other => anyhow::anyhow!(other),
            })?;

        return Ok(res);
    }

    pub fn get(&self, key: &str) -> Result<Option<(DocumentRev, Vec<u8>)>> {
        
        // get rev
        let k_last_rev = &Key(SubKey::LastRev, key.to_string()).to_string();
        let rev = self.db.get(k_last_rev)?;

        // get blob
        let blob_k = &Key(SubKey::Blob, key.to_string()).to_string();
        let blob = self.db.get(blob_k)?;

        //let p:[u8; 4] = (&rev.unwrap().to_vec()[..]).try_into()?;
        //let p2:[u8; 4] = rev.unwrap().to_vec().as_slice().try_into().unwrap();

        //let value = u32::from_be_bytes(&rev.unwrap().to_vec()[..]);
        let res = match (rev, blob) {
            (Some(rev), Some(e)) => {
                let arr:[u8; 4] = (&rev.to_vec()[..]).try_into()?;
                return Ok(Some((u32::from_be_bytes(arr) , e.to_vec())));
            }, 
            _ => Err(anyhow!("Db is in incorrect state {}", key)),
        };

        res
    }
}
