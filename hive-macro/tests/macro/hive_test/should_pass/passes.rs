mod hive {
    #[hive_macro::hive]
    pub mod tests {
        use hive_test::{defines::DefineRegistry, inventory, HiveTargetInfo, TestChannelHandle};
        use probe_rs_test::Session;

        #[hive_macro::hive_test]
        fn my_fancy_test(
            _test_channel: &mut dyn TestChannelHandle,
            session: &mut Session,
            target_info: &HiveTargetInfo,
            _defines: &DefineRegistry,
        ) {
            let _session = session;
            let _target_info = target_info;

            // Doing important test
            let mut i = 0;
            i += 1;

            assert_eq!(i, 1);
        }

        mod a {
            use hive_test::{
                defines::DefineRegistry, inventory, HiveTargetInfo, TestChannelHandle,
            };
            use probe_rs_test::Session;

            #[hive_macro::hive_test]
            fn test(
                _test_channel: &mut dyn TestChannelHandle,
                _session: &mut Session,
                _target_info: &HiveTargetInfo,
                _defines: &DefineRegistry,
            ) {
            }

            mod aa {
                use hive_test::{
                    defines::DefineRegistry, inventory, HiveTargetInfo, TestChannelHandle,
                };
                use probe_rs_test::Session;

                #[hive_macro::hive_test]
                fn test(
                    _test_channel: &mut dyn TestChannelHandle,
                    _session: &mut Session,
                    _target_info: &HiveTargetInfo,
                    _defines: &DefineRegistry,
                ) {
                }

                mod aaa {
                    use hive_test::{
                        defines::DefineRegistry, inventory, HiveTargetInfo, TestChannelHandle,
                    };
                    use probe_rs_test::Session;

                    #[hive_macro::hive_test]
                    fn test(
                        _test_channel: &mut dyn TestChannelHandle,
                        _session: &mut Session,
                        _target_info: &HiveTargetInfo,
                        _defines: &DefineRegistry,
                    ) {
                    }
                }

                mod aab {
                    use hive_test::{
                        defines::DefineRegistry, inventory, HiveTargetInfo, TestChannelHandle,
                    };
                    use probe_rs_test::Session;

                    #[hive_macro::hive_test]
                    fn test(
                        _test_channel: &mut dyn TestChannelHandle,
                        _session: &mut Session,
                        _target_info: &HiveTargetInfo,
                        _defines: &DefineRegistry,
                    ) {
                    }
                }
            }

            mod ab {
                use hive_test::{
                    defines::DefineRegistry, inventory, HiveTargetInfo, TestChannelHandle,
                };
                use probe_rs_test::Session;

                #[hive_macro::hive_test]
                fn test(
                    _test_channel: &mut dyn TestChannelHandle,
                    _session: &mut Session,
                    _target_info: &HiveTargetInfo,
                    _defines: &DefineRegistry,
                ) {
                }
            }
        }

        mod b {
            use hive_test::{
                defines::DefineRegistry, inventory, HiveTargetInfo, TestChannelHandle,
            };
            use probe_rs_test::Session;

            #[hive_macro::hive_test]
            fn test(
                _test_channel: &mut dyn TestChannelHandle,
                _session: &mut Session,
                _target_info: &HiveTargetInfo,
                _defines: &DefineRegistry,
            ) {
            }
        }
    }
}

fn main() {}
