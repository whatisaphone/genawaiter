#[macro_export]
macro_rules! unsafe_generator {
    ($start:expr) => {{
        let mut gen = ::std::mem::MaybeUninit::uninit();
        unsafe {
            $crate::stack::generator::Gen::__macro_internal_popuate(&mut gen, $start);
            ::std::pin::Pin::new_unchecked(gen.as_mut_ptr().as_mut().unwrap())
        }
    }};
}
