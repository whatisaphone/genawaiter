extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_error::*;
use proc_macro_hack::proc_macro_hack;
use quote::{quote, ToTokens};
use syn::{self, parse_macro_input, visit_mut::VisitMut, ItemFn, Type};

mod common;
mod rc;
mod stack;
mod sync;

use common::{YieldClosure, YieldMatchMacro, YieldReplace};

#[proc_macro_attribute]
#[proc_macro_error]
pub fn stack_yield_fn(args: TokenStream, input: TokenStream) -> TokenStream {
    let a = args.clone();
    // make sure it is a valid type
    let _ = parse_macro_input!(a as Type);
    let mut function = parse_macro_input!(input as ItemFn);

    let co_type = args.to_string();
    stack::add_coroutine_arg(&mut function.sig.inputs, co_type);

    let mut y_found = YieldMatchMacro::new();
    y_found.visit_item_fn_mut(&mut function);

    let mut y_rep = YieldReplace::new(y_found);
    y_rep.visit_item_fn_mut(&mut function);

    let tokens = quote! { #function };
    tokens.into()
}

#[proc_macro_hack]
#[proc_macro_error]
pub fn stack_yield_cls(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as YieldClosure);

    let mut yield_cls = input.closure;
    let mut ymc = YieldMatchMacro::new();
    ymc.visit_expr_closure_mut(&mut yield_cls);

    let mut y_replace = YieldReplace::new(ymc);
    y_replace.visit_expr_closure_mut(&mut yield_cls);

    stack::add_coroutine_arg_cls(
        &mut yield_cls.inputs,
        input.ty.to_token_stream().to_string(),
    );

    let tokens = quote! { #yield_cls };
    tokens.into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn sync_yield_fn(args: TokenStream, input: TokenStream) -> TokenStream {
    let a = args.clone();
    // make sure it is a valid type
    let _ = parse_macro_input!(a as Type);
    let mut function = parse_macro_input!(input as ItemFn);

    let co_type = args.to_string();
    sync::add_coroutine_arg(&mut function.sig.inputs, co_type);

    let mut y_found = YieldMatchMacro::new();
    y_found.visit_item_fn_mut(&mut function);

    let mut y_rep = YieldReplace::new(y_found);
    y_rep.visit_item_fn_mut(&mut function);

    let tokens = quote! { #function };
    tokens.into()
}

#[proc_macro_hack]
#[proc_macro_error]
pub fn sync_yield_cls(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as YieldClosure);

    let mut yield_cls = input.closure;
    let mut ymc = YieldMatchMacro::new();
    ymc.visit_expr_closure_mut(&mut yield_cls);

    let mut y_replace = YieldReplace::new(ymc);
    y_replace.visit_expr_closure_mut(&mut yield_cls);

    sync::add_coroutine_arg_cls(
        &mut yield_cls.inputs,
        input.ty.to_token_stream().to_string(),
    );

    let tokens = quote! { #yield_cls };
    tokens.into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn rc_yield_fn(args: TokenStream, input: TokenStream) -> TokenStream {
    let a = args.clone();
    // make sure it is a valid type
    let _ = parse_macro_input!(a as Type);
    let mut function = parse_macro_input!(input as ItemFn);

    let co_type = args.to_string();
    rc::add_coroutine_arg(&mut function.sig.inputs, co_type);

    let mut y_found = YieldMatchMacro::new();
    y_found.visit_item_fn_mut(&mut function);

    let mut y_rep = YieldReplace::new(y_found);
    y_rep.visit_item_fn_mut(&mut function);

    let tokens = quote! { #function };
    tokens.into()
}

#[proc_macro_hack]
#[proc_macro_error]
pub fn rc_yield_cls(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as YieldClosure);

    let mut yield_cls = input.closure;
    let mut ymc = YieldMatchMacro::new();
    ymc.visit_expr_closure_mut(&mut yield_cls);

    let mut y_replace = YieldReplace::new(ymc);
    y_replace.visit_expr_closure_mut(&mut yield_cls);

    rc::add_coroutine_arg_cls(
        &mut yield_cls.inputs,
        input.ty.to_token_stream().to_string(),
    );

    let tokens = quote! { #yield_cls };
    tokens.into()
}
