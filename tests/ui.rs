#[rustversion::stable]
#[cfg(feature = "proc_macro")]
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/rc_fail_not_awaiting_yield.rs");
    t.compile_fail("tests/ui/stack_fail_not_awaiting_yield.rs");
    t.compile_fail("tests/ui/sync_fail_not_awaiting_yield.rs");

    t.compile_fail("tests/ui/fail_producer_with_argument.rs");
    t.compile_fail("tests/ui/stack_fail_when_co_is_static.rs");
}
