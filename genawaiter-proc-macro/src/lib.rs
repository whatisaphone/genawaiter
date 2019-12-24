extern crate proc_macro;

use std::string::ToString;

use proc_macro::TokenStream;
use proc_macro_error::*;
use proc_macro_hack::proc_macro_hack;
use quote::{quote, ToTokens};
use syn::{self, parse_macro_input, visit::Visit, visit_mut::VisitMut};

mod visit;
use visit::{YieldClosure, YieldMatchMacro, YieldReplace};

mod stack {
    pub(crate) const CO_ARG: &str = "co: ::genawaiter::stack::Co<'_, ";
}

mod sync {
    pub(crate) const CO_ARG: &str = "co: ::genawaiter::sync::Co<";
}

mod rc {
    pub(crate) const CO_ARG: &str = "co: ::genawaiter::rc::Co<";
}

fn add_co_arg_closure(cls: &mut syn::ExprClosure, co_arg: &str) {
    let co_arg_found = cls.inputs.iter().any(|input| {
        match input {
            syn::Pat::Type(arg) => {
                match &*arg.ty {
                    syn::Type::Path(ty) => {
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
        let arg = match syn::parse_str::<syn::FnArg>(co_arg) {
            Ok(syn::FnArg::Typed(x)) => x,
            _ => proc_macro_error::abort_call_site!("string Pat parse failed Co<...>"),
        };
        cls.inputs.push(syn::Pat::Type(arg))
    }
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn stack_yield_fn(args: TokenStream, input: TokenStream) -> TokenStream {
    let a = args.clone();
    // make sure it is a valid type
    let _ = parse_macro_input!(a as syn::Type);
    let mut function = parse_macro_input!(input as syn::ItemFn);

    let mut y_found = YieldMatchMacro::new();
    y_found.visit_item_fn(&function);

    let co_type = Some(format!("{}{}>", stack::CO_ARG, args));
    let mut y_rep = YieldReplace::new(y_found, co_type);
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

    let mut y_replace = YieldReplace::new(ymc, None);
    y_replace.visit_expr_closure_mut(&mut yield_cls);

    let tokens = quote! { #yield_cls };
    tokens.into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn sync_yield_fn(args: TokenStream, input: TokenStream) -> TokenStream {
    let a = args.clone();
    // make sure it is a valid type
    let _ = parse_macro_input!(a as syn::Type);
    let mut function = parse_macro_input!(input as syn::ItemFn);

    let mut y_found = YieldMatchMacro::new();
    y_found.visit_item_fn(&function);

    let co_arg = Some(format!("{}{}>", sync::CO_ARG, args));
    let mut y_rep = YieldReplace::new(y_found, co_arg);
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

    let mut y_replace = YieldReplace::new(ymc, None);
    y_replace.visit_expr_closure_mut(&mut yield_cls);

    let tokens = quote! { #yield_cls };
    tokens.into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn rc_yield_fn(arg: TokenStream, input: TokenStream) -> TokenStream {
    let a = arg.clone();
    // make sure it is a valid type
    let _ = parse_macro_input!(a as syn::Type);
    let mut function = parse_macro_input!(input as syn::ItemFn);

    let mut y_found = YieldMatchMacro::new();
    y_found.visit_item_fn(&function);

    let co_arg = Some(format!("{}{}>", rc::CO_ARG, arg));
    let mut y_rep = YieldReplace::new(y_found, co_arg);
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

    let mut y_replace = YieldReplace::new(ymc, None);
    y_replace.visit_expr_closure_mut(&mut yield_cls);

    let tokens = quote! { #yield_cls };
    tokens.into()
}
