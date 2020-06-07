use anyhow::Result;
use notea::{Store, StoreUserError};
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::fs::remove_dir_all;
use std::time::{SystemTime, UNIX_EPOCH};
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
        store.set("test_key", &TEST_VALUE, None, "new_hash")?;

        let stored_hash = db.get("test_key_hash")?;
        let stored_blob = db.get("test_key_blob")?;

        assert_eq!(Some(sled::IVec::from("new_hash".as_bytes())), stored_hash);
        assert_eq!(Some(sled::IVec::from(TEST_VALUE.to_vec())), stored_blob);

        Ok(())
    });
}

#[test]
fn update_value() {
    run_test(|store, db| {
        let key = "test_key";
        store.set(&key, "test_value_1".as_bytes(), None, "hash1")?;
        store.set(&key, &TEST_VALUE, Some("hash1"), "hash2")?;

        let stored_hash = db.get("test_key_hash")?;
        let stored_blob = db.get("test_key_blob")?;

        assert_eq!(Some(sled::IVec::from("hash2".as_bytes())), stored_hash);
        assert_eq!(Some(sled::IVec::from(TEST_VALUE.to_vec())), stored_blob);

        Ok(())
    });
}

#[test]
fn new_value_with_wrong_hash() {
    run_test(|store, db| {
        let r = store.set(
            "test_key",
            &TEST_VALUE,
            Some("wanna_be_nothing"),
            "new_hash",
        );

        let stored_hash = db.get("test_key_hash")?;
        let stored_blob = db.get("test_key_blob")?;

        assert_eq!(None, stored_hash);
        assert_eq!(None, stored_blob);

        let err = r.unwrap_err();
        let typed_err = err.downcast_ref::<StoreUserError>();

        assert_eq!(Some(&StoreUserError::InvalidHash), typed_err);

        Ok(())
    });
}

#[test]
fn update_value_with_wrong_hash() {
    run_test(|store, db| {
        store.set("test_key", &TEST_VALUE, None, "hash1")?;
        let r = store.set(
            "test_key",
            "some-val".as_bytes(),
            Some("wrong_hash"),
            "hash2",
        );

        let stored_hash = db.get("test_key_hash")?;
        let stored_blob = db.get("test_key_blob")?;

        assert_eq!(Some(sled::IVec::from("hash1".as_bytes())), stored_hash);
        assert_eq!(Some(sled::IVec::from(TEST_VALUE.to_vec())), stored_blob);

        let err = r.unwrap_err();
        let typed_err = err.downcast_ref::<StoreUserError>();

        assert_eq!(Some(&StoreUserError::InvalidHash), typed_err);

        Ok(())
    });
}

#[test]
fn get_value() {
    run_test(|store, _| {
        let key = "test_key";

        store.set(&key, &TEST_VALUE, None, "hash")?;

        let res = store.get(key)?;

        assert_eq!(Some(TEST_VALUE.to_vec()), res);

        Ok(())
    });
}
