/// Yields a value from a generator.
///
/// This macro can only be used inside the `gen!` and `producer!` families of
/// macros.
///
/// It will suspend execution of the function until the generator is resumed. At
/// that time, it will evaluate to the resume argument, if given, otherwise it
/// will evaluate to `()`.
///
/// # Examples
///
/// [_See the module-level docs for examples._](.)
#[cfg(feature = "proc_macro")]
#[macro_export]
macro_rules! yield_ {
    ($val:expr) => {
        compile_error!(
            "`yield_!()` can only be used inside one of the genawaiter macros",
        )
    };
    (@__impl => $co:expr, $value:expr) => {
        $co.yield_($value).await
    };
}

// Internal use only. This is a copy of `futures::pin_mut!` so we can avoid
// pulling in a dependency for a two-liner.
#[cfg(feature = "futures03")]
macro_rules! pin_mut {
    ($x:ident) => {
        let mut $x = $x;
        #[allow(unused_mut)]
        let mut $x = unsafe { ::core::pin::Pin::new_unchecked(&mut $x) };
    };
}
