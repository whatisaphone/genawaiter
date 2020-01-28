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
