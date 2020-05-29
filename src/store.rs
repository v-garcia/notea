use sled::{open as open_db, Db, abort, Tree, Batch};
use anyhow::{Result, anyhow};
use super::*;

pub struct Store {
    db: Db,
}

impl Store {
    pub fn init(path: &str) -> Result<Store> {

        let db = open_db(path)?;

        Ok(Self { db: db })
    }

    pub fn get_area(&self, tree_name: &str) -> Result<Area>{
      return Area::init(self, tree_name);
    }

    pub fn inner_db(&self) -> &Db {
      return &self.db;
  }

}

