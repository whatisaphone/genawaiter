extern crate proc_macro;

use proc_macro_error::*;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    self,
    parse_macro_input,
    ItemFn,
    Stmt,
    Type,
};

mod common;
mod stack;
mod sync;
mod rc;

#[proc_macro_attribute]
#[proc_macro_error]
pub fn yielder_fn(args: TokenStream, input: TokenStream) -> TokenStream {
    // make sure it is a valid type
    let a = args.clone();
    let _ = parse_macro_input!(a as Type);
    let mut function = parse_macro_input!(input as ItemFn);

    let co_type = args.to_string();
    stack::add_coroutine_arg(&mut function.sig.inputs, co_type);
    common::parse_block_stmts(&mut *function.block.stmts);

    let tokens = quote! { #function };
    tokens.into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn yielder_cls(args: TokenStream, input: TokenStream) -> TokenStream {
    // make sure it is a valid type
    let a = args.clone();
    let _ = parse_macro_input!(a as Type);
    let mut function: Stmt = parse_macro_input!(input);

    stack::parse_cls(args, &mut function);

    let tokens = quote! { #function };
    tokens.into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn yielder_fn_sync(args: TokenStream, input: TokenStream) -> TokenStream {
    // make sure it is a valid type
    let a = args.clone();
    let _ = parse_macro_input!(a as Type);
    let mut function = parse_macro_input!(input as ItemFn);

    let co_type = args.to_string();
    sync::add_coroutine_arg(&mut function.sig.inputs, co_type);
    common::parse_block_stmts(&mut *function.block.stmts);

    let tokens = quote! { #function };
    tokens.into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn yielder_cls_sync(args: TokenStream, input: TokenStream) -> TokenStream {
    // make sure it is a valid type
    let a = args.clone();
    let _ = parse_macro_input!(a as Type);
    let mut function: Stmt = parse_macro_input!(input);

    sync::parse_cls(args, &mut function);

    let tokens = quote! { #function };
    tokens.into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn yielder_fn_rc(args: TokenStream, input: TokenStream) -> TokenStream {
    // make sure it is a valid type
    let a = args.clone();
    let _ = parse_macro_input!(a as Type);
    let mut function = parse_macro_input!(input as ItemFn);

    let co_type = args.to_string();
    rc::add_coroutine_arg(&mut function.sig.inputs, co_type);
    common::parse_block_stmts(&mut *function.block.stmts);

    let tokens = quote! { #function };
    tokens.into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn yielder_cls_rc(args: TokenStream, input: TokenStream) -> TokenStream {
    // make sure it is a valid type
    let a = args.clone();
    let _ = parse_macro_input!(a as Type);
    let mut function: Stmt = parse_macro_input!(input);

    rc::parse_cls(args, &mut function);

    let tokens = quote! { #function };
    tokens.into()
}

#[cfg(test)]
mod test {
    use super::*;
    use genawaiter::{
        generator_mut,
        stack::{Co, Gen, Shelf},
        yield_,
    };
    
    #[yielder_fn(u8)]
    async fn odds() {
        for n in (1_u8..).step_by(2).take_while(|&n| n < 10) {
            yield_! { n };
        }
    }

    #[test]
    fn stack_fn() {
        generator_mut!(gen, odds);
        let res = gen.into_iter().collect::<Vec<_>>();
        assert_eq!(vec![1, 3, 5, 7, 9], res)
    }

    #[cfg_if(features = nightly)]
    #[test]
    fn stack_closure() {

    }
}
