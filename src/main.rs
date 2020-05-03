use std::fs;
use std::str;
use notea::{KVStore};
use std::path::Path;
use anyhow::Result;

static DB_PATH: &str = "target/my_store";

fn main() -> Result<()> {
    
    let file_name = ".gitignore";
    let file = &fs::read(&file_name).unwrap();

    let store = KVStore::init(Path::new(DB_PATH))?;

    store.set("custom", "my_key", file)?;

    let r = store.get("custom", "my_key")?;

    println!("{:x?}", r);

    Ok(())
}
