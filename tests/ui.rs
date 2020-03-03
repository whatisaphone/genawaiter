#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/rc_fail_not_awaiting_yield.rs");
    t.compile_fail("tests/ui/stack_fail_not_awaiting_yield.rs");
    t.compile_fail("tests/ui/sync_fail_not_awaiting_yield.rs");
}
