use notea::{KVStore};
use anyhow::Result;
use std::fs;
use rand::Rng; 
use rand::distributions::Alphanumeric;



fn gen_store_path() -> String {
    let rnd = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .collect::<String>(); 

    format!("{}_{}", "target/test_store", rnd)
}

fn run_test<T>(test: T) -> ()
    where T: FnOnce(&KVStore) -> Result<()> + std::panic::UnwindSafe
{
    let result = std::panic::catch_unwind(|| {
        let path = gen_store_path();
        let store = KVStore::init(&path).unwrap();
        let r = test(&store);
        store.inner_db().clear()?;
        r
    });

    assert!(result.is_ok())
}

#[test]
fn insert_value() {
    
    const VALUE: [u8; 16] = [0u8, 1u8, 2u8,3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8, 10u8, 11u8, 12u8, 13u8, 14u8, 15u8,];

    run_test(|store| {
        store.set("custom", "my_key", &VALUE)?;
        Ok(())
    })
}

#[test]
fn get_value() {
    const VALUE: [u8; 16] = [0u8, 1u8, 2u8,3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8, 10u8, 11u8, 12u8, 13u8, 14u8, 15u8,];
    let key = "custom";
    let tree_name = "tree_name";

    run_test(|store| {

        store.set(key, tree_name, &VALUE)?;
        
        let r= store.get(key, tree_name)?;

        assert_eq!(VALUE.to_vec(), r);

        Ok(())
    })
}

#[test]
fn create_db() -> Result<()> {
    let path = gen_store_path();
    print!("asdfaf");
    let s =KVStore::init(&path)?;
    s.inner_db().clear()?;
    Ok(())
}
