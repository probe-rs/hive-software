use hive_db::{BincodeDb, HiveDb, Key};

const FLUSH_INTERVAL_MS: u64 = 60_000;
const CACHE_CAPACITY: u64 = 52_428_800;

fn main() {
    let db = HiveDb::open("dummy_path", FLUSH_INTERVAL_MS, CACHE_CAPACITY);

    let tree = db.open_tree("tree");

    let my_key: Key<u8> = Key::new("NiceNumber");

    let wrong_type: Option<u64> = tree.b_get(&my_key).unwrap();
}
