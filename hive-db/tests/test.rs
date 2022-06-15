#[test]
fn check_typesafety() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/typesafety/*.rs");
}
