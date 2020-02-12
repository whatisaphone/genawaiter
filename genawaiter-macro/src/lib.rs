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
#[macro_export]
macro_rules! gen_rc {
    ($func:expr) => {
        ::genawaiter::rc::Gen::new(::genawaiter::rc_producer!({ $func }))
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
#[macro_export]
macro_rules! gen_stack {
    ($name:ident, $func:expr) => {
        let mut shelf = ::genawaiter::stack::Shelf::new();
        let mut generator = unsafe {
            ::genawaiter::stack::Gen::new(
                &mut shelf,
                ::genawaiter::stack_producer!({ $func }),
            )
        };
        let $name = &mut generator;
    };
    ($name:ident resume $func:expr) => {
        let mut shelf = ::genawaiter::stack::Shelf::new();
        let mut generator = unsafe {
            ::genawaiter::stack::Gen::new(
                &mut shelf,
                ::genawaiter::stack_producer_resume!({ $func }),
            )
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
#[macro_export]
macro_rules! gen_sync {
    ($func:expr) => {
        ::genawaiter::sync::Gen::new(::genawaiter::sync_producer!({ $func }))
    };
}
