#[test]
fn check_macro() {
    let t = trybuild::TestCases::new();

    t.compile_fail("tests/macro/hive_test/should_fail/*.rs");
    t.pass("tests/macro/hive_test/should_pass/*.rs");

    t.compile_fail("tests/macro/hive/should_fail/*.rs");
    t.pass("tests/macro/hive/should_pass/*.rs");
}
