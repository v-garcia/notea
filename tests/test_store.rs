use anyhow::Result;
use notea::{Store, StoreUserError};
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs::{remove_dir_all};
const TEST_VALUE: [u8; 16] = [
    0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8, 10u8, 11u8, 12u8, 13u8, 14u8, 15u8,
];

fn gen_store_path() -> String {
    let rnd = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .collect::<String>();

    format!("{}_{}", "target/test_store", rnd)
}

fn run_test<T>(test: T) -> ()
where
    T: FnOnce(&Store, &sled::Db) -> Result<()> + std::panic::UnwindSafe,
{
    let result = std::panic::catch_unwind(|| {
        let path = &gen_store_path();
        let db = sled::open(path).unwrap();
        let db2 = db.clone();
        let store = Store::init_from_db(db);    
        test(&store, &db2).unwrap();
        db2.clear().unwrap();
        remove_dir_all(path).unwrap();
    });

    assert!(result.is_ok())
}

#[test]
fn set_new_value() {
    run_test(|store, db| {

        store.set("test_key", &TEST_VALUE, 0, None)?;

        let stored = db.get("blob_test_key")?;

        assert_eq!(Some(sled::IVec::from(TEST_VALUE.to_vec())), stored);

        Ok(())
    });
}

#[test]
fn update_value() {
    run_test(|store, db| {
        let key = "test_key";
        store.set(&key, "test_value_1".as_bytes(), 0, None)?;
        store.set(&key, &TEST_VALUE, 1, None)?;
        let stored = db.get("blob_test_key")?;

        assert_eq!(Some(sled::IVec::from(TEST_VALUE.to_vec())), stored);

        Ok(())
    });
}

#[test]
fn new_value_with_wrong_rev() {
    run_test(|store, db| {
        let res = store.set("test_key", &TEST_VALUE, 1, None);

        assert!(res.is_err());
        
        let err = res.unwrap_err();
        let typed_err = err.downcast_ref::<StoreUserError>();

        assert_eq!(Some(&StoreUserError::BadRevision), typed_err);
        assert_eq!(None, db.get("blob_test_key")?);
        assert_eq!(None, db.get("last_rev_test_key")?);

        Ok(())
    });
}

#[test]
fn update_value_with_wrong_rev() {
    run_test(|store, db| {
        let key = "test_key";
        store.set(&key, "test_value_1".as_bytes(), 0, None)?;
        let res = store.set(&key, &TEST_VALUE, 2, None);


        assert!(res.is_err());
        
        let err = res.unwrap_err();
        let typed_err = err.downcast_ref::<StoreUserError>();

        assert_eq!(Some(&StoreUserError::BadRevision), typed_err);
        assert_eq!(Some(sled::IVec::from("test_value_1".as_bytes())), db.get("blob_test_key")?);
        assert_eq!(Some(sled::IVec::from(&0x00000001u32.to_be_bytes())), db.get("last_rev_test_key")?);

        Ok(())
    });
}


#[test]
fn update_value_with_wrong_owner_key() {
    run_test(|store, db| {
        let key = "test_key";
        let owner_key = "owner_key";
        store.set(&key, "test_value_1".as_bytes(), 0, Some(owner_key.as_bytes()))?;
        let res = store.set(&key, &TEST_VALUE, 1, Some("wrong".as_bytes()));

        assert!(res.is_err());
        
        let err = res.unwrap_err();
        let typed_err = err.downcast_ref::<StoreUserError>();

        assert_eq!(Some(&StoreUserError::InvalidOwnerKey), typed_err);
        assert_eq!(Some(sled::IVec::from("test_value_1".as_bytes())), db.get("blob_test_key")?);
        assert_eq!(Some(sled::IVec::from(&0x00000001u32.to_be_bytes())), db.get("last_rev_test_key")?);
        assert_eq!(Some(sled::IVec::from("owner_key".as_bytes())), db.get("owner_key_test_key")?);
        Ok(())
    });
}