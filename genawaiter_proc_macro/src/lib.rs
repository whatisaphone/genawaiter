extern crate proc_macro;

use proc_macro_error::*;

use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input, visit_mut::VisitMut, ItemFn, Stmt, Type};

mod common;
mod rc;
mod stack;
mod sync;

use common::{YieldMatchMacro, YieldReplace};

#[proc_macro_attribute]
#[proc_macro_error]
pub fn yielder_fn(args: TokenStream, input: TokenStream) -> TokenStream {
    let a = args.clone();
    // make sure it is a valid type
    let _ = parse_macro_input!(a as Type);
    let mut function = parse_macro_input!(input as ItemFn);

    let co_type = args.to_string();
    stack::add_coroutine_arg(&mut function.sig.inputs, co_type);

    let mut y_found = YieldMatchMacro::new();
    y_found.visit_item_fn_mut(&mut function);
    println!("{:#?}", y_found.coll_replace);

    let mut y_rep = YieldReplace::new(y_found);
    y_rep.visit_item_fn_mut(&mut function);
    // common::parse_block_stmts(&mut *function.block.stmts);

    let tokens = quote! { #function };
    tokens.into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn yielder_cls(args: TokenStream, input: TokenStream) -> TokenStream {
    let a = args.clone();
    // make sure it is a valid type
    let _ = parse_macro_input!(a as Type);
    let mut function: Stmt = parse_macro_input!(input);

    stack::parse_cls(args, &mut function);

    let tokens = quote! { #function };
    tokens.into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn yielder_fn_sync(args: TokenStream, input: TokenStream) -> TokenStream {
    let a = args.clone();
    // make sure it is a valid type
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
    let a = args.clone();
    // make sure it is a valid type
    let _ = parse_macro_input!(a as Type);
    let mut function: Stmt = parse_macro_input!(input);

    sync::parse_cls(args, &mut function);

    let tokens = quote! { #function };
    tokens.into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn yielder_fn_rc(args: TokenStream, input: TokenStream) -> TokenStream {
    let a = args.clone();
    // make sure it is a valid type
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
    let a = args.clone();
    // make sure it is a valid type
    let _ = parse_macro_input!(a as Type);
    let mut function: Stmt = parse_macro_input!(input);

    rc::parse_cls(args, &mut function);

    let tokens = quote! { #function };
    tokens.into()
}
