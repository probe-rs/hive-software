#[hive_macro::hive]
pub mod tests {
    mod a {
        mod b {
            #![allow(unused)]
            use hive_test;
            use std;
            use {
                std::sync,
                std::time,
                {quote, std::alloc, std::io},
            };
        }
    }
}

fn main() {}
