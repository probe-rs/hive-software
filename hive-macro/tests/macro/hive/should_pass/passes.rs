pub mod hive {
    #[hive_macro::hive]
    pub mod tests {
        use hive_test::{defines::DefineRegistry, HiveTargetInfo, TestChannelHandle};
        use probe_rs_test::Session;

        const TEST: u8 = 1;

        mod a {
            #![allow(unused)]
            use super::TEST;
        }
    }
}

fn main() {}
