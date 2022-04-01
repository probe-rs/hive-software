#[test]
fn check_macro() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/macro/should_fail/*.rs");
    t.pass("tests/macro/should_pass/*.rs");
}
