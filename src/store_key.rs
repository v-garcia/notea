use anyhow::{Result, anyhow};
use std::convert::TryFrom;

pub type Space = u16;

pub type DocumentRev = u16;

pub type DocumentId = u32;

const SPACE_AREA_METADATA: Space = 0;

const SPACE_DOCUMENT_LAST_REVS: Space = 1;

const SPACE_DOCUMENT_BLOBS: Space = 2;

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SpaceEnum {
    AreaMetadatas = SPACE_AREA_METADATA,
    DocumentLastRevs = SPACE_DOCUMENT_LAST_REVS,
    DocumentsBlobs = SPACE_DOCUMENT_BLOBS,
}

const AREA_UUID: DocumentId = 0;

const BASE_REV: DocumentRev = 0;

pub enum StoreKey {
    AreaGuid ,
    LastRevision(DocumentId),
    Blob {id: DocumentId, rev:DocumentRev},
}

impl StoreKey {

    fn to_ints(&self) -> (SpaceEnum, DocumentId,  DocumentRev) {
        match self {
            StoreKey::AreaGuid                   => (SpaceEnum::AreaMetadatas, AREA_UUID , BASE_REV),
            StoreKey::Blob {id, rev} => (SpaceEnum::DocumentsBlobs, *id , *rev),
            StoreKey::LastRevision(id)     => (SpaceEnum::DocumentLastRevs, *id, BASE_REV),
        }
    }

    pub fn to_be_bytes(&self) -> [u8; 8] {
        let (a, b, c) = self.to_ints();
        let main_id: u64 = ((a as u64) << 24) + ((b as u64) << 8) + (c as u64);
        return main_id.to_be_bytes()
    }

    pub fn get_document_prefix(&self) -> [u8; 6] {
        let mut res: [u8; 6] = [0; 6];
        
        let (a, b, _) = self.to_ints();
        let a: [u8; 2] = (a as u16).to_be_bytes();
        let b: [u8; 4] = (b as u32).to_be_bytes();

        for (place, data) in res.iter_mut().zip(a.iter()) {
            *place = *data
        }

        for (place, data) in res.iter_mut().skip(2).zip(b.iter()) {
            *place = *data
        }
        
        res
    }

}


impl TryFrom<u64> for StoreKey {
    type Error = anyhow::Error;

    fn try_from(id: u64) -> Result<Self> {
        let a:u16 = (id >> 24) as u16;
        let b:u32 = ((id << 8) >> 16) as u32;
        let c:u16 = ((id << 24) >> 24) as u16;

        return match (a, b, c) {
            (SPACE_AREA_METADATA, AREA_UUID, BASE_REV) => Ok(StoreKey::AreaGuid),
            (SPACE_DOCUMENT_LAST_REVS, i, BASE_REV)    => Ok(StoreKey::LastRevision(i)),
            (SPACE_DOCUMENT_BLOBS, i, r)               => Ok(StoreKey::Blob {id: i, rev: r}),
            _                                                         => Err(anyhow!("Unsupported integer")),

        }
    }
}


