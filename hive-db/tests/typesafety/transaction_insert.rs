use hive_db::{BincodeTransactional, HiveDb, Key};

const FLUSH_INTERVAL_MS: u64 = 60_000;
const CACHE_CAPACITY: u64 = 52_428_800;

fn main() {
    let db = HiveDb::open("dummy_path", FLUSH_INTERVAL_MS, CACHE_CAPACITY);

    let tree = db.open_tree("tree");

    let my_key: Key<u8> = Key::new("NiceNumber");

    tree.transaction(|tree| {
        let correct_data = 2;
        let previous: Option<bool> = tree.b_insert(&my_key, &correct_data)?;

        Ok(())
    })
    .unwrap();
}
