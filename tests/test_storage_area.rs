use notea::{Store, Area, StoreKey};
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::Result;
use rand::Rng;
use rand::distributions::Alphanumeric;

const TEST_VALUE : [u8; 16] = [0u8, 1u8, 2u8,3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8, 10u8, 11u8, 12u8, 13u8, 14u8, 15u8,];



fn gen_store_path() -> String {
    let rnd = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .collect::<String>(); 

    format!("{}_{}", "target/test_store", rnd)
}

fn run_test<T>(test: T) -> ()
    where T: FnOnce(&Area) -> Result<()> + std::panic::UnwindSafe
{
    let result = std::panic::catch_unwind(|| {
        let path = gen_store_path();
        let store = Store::init(&path[..]).unwrap();
        let area = store.get_area("tree_name").unwrap();
        let r = test(&area);
        store.inner_db().clear()?;
        r
    });

    assert!(result.is_ok())
}

#[test]
fn simple_get() {
    run_test(|area| {
        let tree = area.inner_tree();

        let mut rng = rand::thread_rng();
        let id = rng.gen::<u32>();
        let rev = rng.gen::<u16>();
        let key = StoreKey::Blob {id:  id, rev: rev };

        tree.insert(key.to_be_bytes(), &TEST_VALUE)?;

        let value = area.get_document_revision(id, rev)?;

        assert_eq!(TEST_VALUE.to_vec(), value);

        Ok(())
    });
}

// #[test]
// fn insert_new() {
//     run_test(|store| {


//         store.safe_set(None, ts, "tree_name",  "key_name", &TEST_VALUE)?;

//         let tree = store.inner_db().open_tree("tree_name")?;
//         let value = tree.get("key_name_content")?;
//         let inserted_ts = tree.get("key_name_timestamp")?;

//         assert_eq!(Some(sled::IVec::from(&TEST_VALUE)), value);
//         assert_eq!(Some(sled::IVec::from(&ts.to_be_bytes())), inserted_ts);
//         Ok(())
//     })
// }

// #[test]
// fn correct_update() {

//     run_test(|store| {

//         // First set
//         let ts = store.safe_set(None,now(), "tree_name",  "key_name", &TEST_VALUE)?;

        
//         // Create second value
//         let second_val = TEST_VALUE.iter().cloned().rev().collect::<Vec<u8>>();
//         wait();
        
//         // Second set
//         let ts2 = store.safe_set(Some(ts), now(), "tree_name",  "key_name", &second_val[..])?;
  

//         let tree = store.inner_db().open_tree("tree_name")?;
//         let value = tree.get("key_name_content")?;
//         let inserted_ts = tree.get("key_name_timestamp")?;

//         assert_eq!(Some(sled::IVec::from(second_val)), value);
//         assert_eq!(Some(sled::IVec::from(&ts2.to_be_bytes())), inserted_ts);

//         Ok(())
//     })
// }

// #[test]
// fn delete() {

//     run_test(|store| {


//         let tree = store.inner_db().open_tree("tree_name")?;

//         tree.insert("aaa_content", &TEST_VALUE)?;
//         tree.insert("aaa_timestamp", &TEST_VALUE)?;

//         let ts = now();
//         tree.insert("aab_1", &TEST_VALUE)?;
//         tree.insert("aab_2", &TEST_VALUE)?;
//         tree.insert("aab_3", &TEST_VALUE)?;
//         tree.insert("aab_timestamp", &ts.to_be_bytes())?;

//         tree.insert("aac_content", &TEST_VALUE)?;
//         tree.insert("aac_timestamp", &TEST_VALUE)?;

//         // Run delete
//         store.remove(ts, "tree_name", "aab")?;
        

//         assert!(tree.contains_key("aaa_content").unwrap());
//         assert!(tree.contains_key("aaa_timestamp").unwrap());

//         assert!(!tree.contains_key("aab_1").unwrap());
//         assert!(!tree.contains_key("aab_2").unwrap());
//         assert!(!tree.contains_key("aab_3").unwrap());
//         assert!(!tree.contains_key("aab_timestamp").unwrap());

//         assert!(tree.contains_key("aac_content").unwrap());
//         assert!(tree.contains_key("aac_timestamp").unwrap());

//         Ok(())
//     })
// }


// #[test]
// fn create_db() -> Result<()> {
//     let path = gen_store_path();
//     let s =KVStore::init(&path)?;
//     s.inner_db().clear()?;
//     Ok(())
// }
