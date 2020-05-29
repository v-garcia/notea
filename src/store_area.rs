
use sled::{open as open_db, Db, abort, Tree, Batch};
use anyhow::{Result, anyhow};
use super::*;

pub struct Area {
    tree: Tree,
}


impl Area {

    pub fn init(store:&Store, tree_name: &str) -> Result<Area> {
        return match  store.inner_db().open_tree(tree_name) {
            Ok(t)   => Ok(Self {tree: t}),
            Err(e) => Err(anyhow!(e))
          };
    }

    pub  fn set_document(&self,  id: DocumentId, rev: DocumentRev, val: &[u8], keep_old_rev: bool) -> Result<DocumentRev> {

        self.tree.transaction(|tree| {

            // Get last revision in db

            let bd_rev_k = StoreKey::LastRevision(id);
            let bd_rev_v = self.tree.get(bd_rev_k.to_be_bytes())?;

            // Check if there are no new revisions
            match (&bd_rev_v, &(rev - 1).to_be_bytes()) {
                (Some(i), r) if (i != r) => return abort("Your revision does not match last one"),
                _                                         => ()
            }

            // Insert blob + update rev
            let store_key = StoreKey::Blob {id: id, rev:rev};
            tree.insert(&bd_rev_k.to_be_bytes()[..], &rev.to_be_bytes())?;
            tree.insert(&store_key.to_be_bytes()[..], val)?;

            // Rm old rev blob if needed
            if let (Some(_), false) = (&bd_rev_v, keep_old_rev) {
                tree.remove(&StoreKey::Blob{id:id, rev:(rev - 1)}.to_be_bytes()[..]);
            }

            Ok(())
        }).map_err(anyhow::Error::msg)?;

        return Ok(rev);
    }




    pub fn remove_document(&self,  id: DocumentId, rev: DocumentRev, val: &[u8]) -> Result<()> {
        let prefix = StoreKey::Blob {id:id, rev:rev}.get_document_prefix();
        let mut iter = self.tree.scan_prefix(prefix);
        let mut batch = Batch::default();

        while let  Some(e) = iter.next() {
            let (k, _) = e?;
            batch.remove(&k);
        };

        self.tree.transaction(|tree| {

            // Get last revision in db
            let bd_rev_k = StoreKey::LastRevision(id);
            let bd_rev_v = self.tree.get(bd_rev_k.to_be_bytes())?;
            
            // Check if there are no new revisions
            match (bd_rev_v, (rev - 1).to_be_bytes()) {
                (Some(i), r) if (i != r) => return abort("Your revision does not match last one"),
                _                                       => ()
            }

            // Remove revision
            let mut b = batch.to_owned();

            b.remove(&bd_rev_k.to_be_bytes());
            
            tree.apply_batch(b)?;

            Ok(())
    
        }).map_err(anyhow::Error::msg)?;

        return Ok(());
    }

    
    pub fn get_document_revision(&self, id: DocumentId, rev: DocumentRev) -> Result<Vec<u8>> {
        
        let key = StoreKey::Blob {id: id, rev:rev };

        let r = match self.tree.get(key.to_be_bytes()) {
            Ok(Some(res)) => Ok(res.to_vec()),
            Ok(None)            => Err(anyhow!("Key not found {}:{}" ,&id ,&rev)),
            Err(e)       => Err(anyhow!(e))
        };

        r
    }

    pub fn inner_tree(&self) -> &Tree {
        return &self.tree;
    }


}