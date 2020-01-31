pub fn test_it() {}
/// Convenience macro to allow for the easy creation of rc generators.
///
/// ## Examples
///
/// ```
/// use genawaiter::gen_rc;
///
/// let g = gen_rc!({
/// let mut n = 1;
///     while n < 10 {
///         yield_!(n);
///         n += 2;
///     }
/// });
/// # let res = g.into_iter().collect::<Vec<_>>();
/// # assert_eq!(vec![1, 3, 5, 7, 9], res)
#[cfg(feature = "proc_macro")]
#[macro_export]
macro_rules! gen_rc {
    ($func:expr) => {
        $crate::rc::Gen::new($crate::rc_producer!({ $func }))
    };
}

/// Convenience macro to allow for the easy creation of stack generators.
///
/// ## Examples
///
/// ```
/// use genawaiter::gen_stack;
///
/// genawaiter::gen_stack!(gen, {
/// let mut n = 1;
///     while n < 10 {
///         yield_!(n);
///         n += 2;
///     }
/// });
/// # let res = gen.into_iter().collect::<Vec<_>>();
/// # assert_eq!(vec![1, 3, 5, 7, 9], res)
#[cfg(feature = "proc_macro")]
#[macro_export]
macro_rules! gen_stack {
    ($name:ident, $func:expr) => {
        let mut shelf = $crate::stack::Shelf::new();
        let mut generator = unsafe {
            $crate::stack::Gen::new(&mut shelf, $crate::stack_producer!({ $func }))
        };
        let $name = &mut generator;
    };
}

/// Convenience macro to allow for the easy creation of sync generators.
///
/// ## Examples
///
/// ```
/// use genawaiter::gen_sync;
///
/// let g = genawaiter::gen_sync!({
/// let mut n = 1;
///     while n < 10 {
///         yield_!(n);
///         n += 2;
///     }
/// });
/// # let res = g.into_iter().collect::<Vec<_>>();
/// # assert_eq!(vec![1, 3, 5, 7, 9], res)
#[cfg(feature = "proc_macro")]
#[macro_export]
macro_rules! gen_sync {
    ($func:expr) => {
        $crate::sync::Gen::new($crate::sync_producer!({ $func }))
    };
}
