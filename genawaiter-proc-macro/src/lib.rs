extern crate proc_macro;

use std::string::ToString;

use proc_macro::TokenStream;
use proc_macro_error::*;
use proc_macro_hack::proc_macro_hack;
use quote::{quote, ToTokens};
use syn::{
    self,
    parse_macro_input,
    parse_str,
    punctuated::Punctuated,
    token::Comma,
    visit::Visit,
    visit_mut::VisitMut,
    ExprClosure,
    FnArg,
    Ident,
    ItemFn,
    Pat,
    Type,
};

mod visit;
use visit::{YieldClosure, YieldMatchMacro, YieldReplace};

#[proc_macro_attribute]
#[proc_macro_error]
pub fn stack_yield_fn(args: TokenStream, input: TokenStream) -> TokenStream {
    let a = args.clone();
    // make sure it is a valid type
    let _ = parse_macro_input!(a as Type);
    let mut function = parse_macro_input!(input as ItemFn);

    let mut y_found = YieldMatchMacro::new();
    y_found.visit_item_fn(&function);

    let co_arg = format!("{}{}>", stack::CO_ARG, args);
    add_coroutine_arg(&mut function, &co_arg);

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
    ymc.visit_expr_closure(&yield_cls);

    let ty = input.ty.to_token_stream().to_string();
    let co_arg = format!("{}{}>", stack::CO_ARG, ty);
    add_co_arg_closure(&mut yield_cls, &co_arg);

    let mut y_replace = YieldReplace::new(ymc);
    y_replace.visit_expr_closure_mut(&mut yield_cls);

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

    let mut y_found = YieldMatchMacro::new();
    y_found.visit_item_fn(&function);

    let co_arg = format!("{}{}>", sync::CO_ARG, args);
    add_coroutine_arg(&mut function, &co_arg);

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
    ymc.visit_expr_closure(&yield_cls);

    let ty = input.ty.to_token_stream().to_string();
    let co_arg = format!("{}{}>", sync::CO_ARG, ty);
    add_co_arg_closure(&mut yield_cls, &co_arg);

    let mut y_replace = YieldReplace::new(ymc);
    y_replace.visit_expr_closure_mut(&mut yield_cls);

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

    let mut y_found = YieldMatchMacro::new();
    y_found.visit_item_fn(&function);

    let co_arg = format!("{}{}>", rc::CO_ARG, args);
    add_coroutine_arg(&mut function, &co_arg);

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
    ymc.visit_expr_closure(&yield_cls);

    let ty = input.ty.to_token_stream().to_string();
    let co_arg = format!("{}{}>", rc::CO_ARG, ty);
    add_co_arg_closure(&mut yield_cls, &co_arg);

    let mut y_replace = YieldReplace::new(ymc);
    y_replace.visit_expr_closure_mut(&mut yield_cls);

    let tokens = quote! { #yield_cls };
    tokens.into()
}

mod stack {
    pub(crate) const CO_ARG: &str = "co: ::genawaiter::stack::Co<'_, ";
}

mod sync {
    pub(crate) const CO_ARG: &str = "co: ::genawaiter::sync::Co<";
}

mod rc {
    pub(crate) const CO_ARG: &str = "co: ::genawaiter::rc::Co<";
}

/// Mutates the input `Punctuated<FnArg, Comma>` to a lifetimeless `co:
/// Co<{type}>`.
pub(crate) fn add_coroutine_arg(func: &mut ItemFn, co_ty: &str) {
    let co_arg_found = func.sig.inputs.iter().any(|input| {
        match input {
            FnArg::Receiver(_) => false,
            FnArg::Typed(arg) => {
                match &*arg.ty {
                    Type::Path(ty) => {
                        ty.path.segments.iter().any(|seg| {
                            seg.ident
                                == parse_str::<Ident>("Co").expect("Ident parse failed")
                        })
                    }
                    _ => false,
                }
            }
        }
    });
    if !co_arg_found {
        let co_arg: FnArg = match parse_str::<FnArg>(co_ty) {
            Ok(s) => s,
            Err(err) => abort_call_site!(format!("invalid type for Co yield {}", err)),
        };
        func.sig.inputs.push_value(co_arg)
    }
}

fn add_co_arg_closure(cls: &mut ExprClosure, co_arg: &str) {
    let co_arg_found = cls.inputs.iter().any(|input| {
        match input {
            Pat::Type(arg) => {
                match &*arg.ty {
                    Type::Path(ty) => {
                        ty.path
                            .segments
                            .iter()
                            .any(|seg| seg.ident == "Co".to_string())
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    });
    if !co_arg_found {
        let arg = match parse_str::<FnArg>(co_arg) {
            Ok(FnArg::Typed(x)) => x,
            _ => proc_macro_error::abort_call_site!("string Pat parse failed Co<...>"),
        };
        cls.inputs.push(Pat::Type(arg))
    }
}
