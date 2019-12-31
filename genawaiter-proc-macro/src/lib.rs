extern crate proc_macro;

use std::string::ToString;

use proc_macro::TokenStream;
use proc_macro_error::{abort, abort_call_site, proc_macro_error};
use proc_macro_hack::proc_macro_hack;
use quote::quote;
use syn::{
    self,
    parse2,
    parse_macro_input,
    parse_str,
    spanned::Spanned,
    visit_mut::VisitMut,
    Expr,
    ExprClosure,
    FnArg,
    Ident,
    ItemFn,
    Pat,
    Token,
    Type,
};

mod visit;
use visit::{YieldClosure, YieldReplace};

/// Macro attribute to turn an `async fn` into a generator, yielding values on
/// the stack,
#[proc_macro_attribute]
#[proc_macro_error]
pub fn stack_producer_fn(args: TokenStream, input: TokenStream) -> TokenStream {
    let a = args.clone();
    // make sure it is a valid type
    let _ = parse_macro_input!(a as Type);
    let mut function = parse_macro_input!(input as ItemFn);

    let co_arg = format!("{}{}>", stack::CO_ARG_FN, args);
    add_coroutine_arg(&mut function, &co_arg);

    YieldReplace.visit_item_fn_mut(&mut function);

    let tokens = quote! { #function };
    tokens.into()
}

/// Function like `proc_macro` to easily and safely create generators from
/// closures on the stack.
#[proc_macro_hack]
#[proc_macro_error]
pub fn stack_producer(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as YieldClosure);

    add_move_async_closure(&mut input.closure);
    add_co_arg_closure(&mut input.closure, &stack::CO_ARG);
    YieldReplace.visit_expr_closure_mut(&mut input.closure);

    let closure = input.closure;
    let tokens = quote! { #closure };
    tokens.into()
}

/// Macro attribute to turn an `async fn` into a generator that can be
/// sent across threads.
#[proc_macro_attribute]
#[proc_macro_error]
pub fn sync_producer_fn(args: TokenStream, input: TokenStream) -> TokenStream {
    let a = args.clone();
    // make sure it is a valid type
    let _ = parse_macro_input!(a as Type);
    let mut function = parse_macro_input!(input as ItemFn);

    let co_arg = format!("{}{}>", sync::CO_ARG_FN, args);
    add_coroutine_arg(&mut function, &co_arg);

    YieldReplace.visit_item_fn_mut(&mut function);

    let tokens = quote! { #function };
    tokens.into()
}

/// Attribute `proc_macro` to easily create generators from
/// closures that are `Sync`.
#[proc_macro_hack]
#[proc_macro_error]
pub fn sync_producer(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as YieldClosure);

    add_move_async_closure(&mut input.closure);
    add_co_arg_closure(&mut input.closure, &sync::CO_ARG);
    YieldReplace.visit_expr_closure_mut(&mut input.closure);

    let closure = input.closure;
    let tokens = quote! { #closure };
    tokens.into()
}

/// Macro attribute to turn an `async fn` into a ref counted (`Rc`) generator.
#[proc_macro_attribute]
#[proc_macro_error]
pub fn rc_producer_fn(args: TokenStream, input: TokenStream) -> TokenStream {
    let a = args.clone();
    // make sure it is a valid type
    let _ = parse_macro_input!(a as Type);
    let mut function = parse_macro_input!(input as ItemFn);

    let co_arg = format!("{}{}>", rc::CO_ARG_FN, args);
    add_coroutine_arg(&mut function, &co_arg);

    YieldReplace.visit_item_fn_mut(&mut function);

    let tokens = quote! { #function };
    tokens.into()
}

/// Function like `proc_macro` to easily create generators from
/// closures that are `Rc`.
#[proc_macro_hack]
#[proc_macro_error]
pub fn rc_producer(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as YieldClosure);

    add_move_async_closure(&mut input.closure);
    add_co_arg_closure(&mut input.closure, &rc::CO_ARG);
    YieldReplace.visit_expr_closure_mut(&mut input.closure);

    let closure = input.closure;
    let tokens = quote! { #closure };
    tokens.into()
}

mod stack {
    pub(crate) const CO_ARG_FN: &str = "co: ::genawaiter::stack::Co<'_, ";
    pub(crate) const CO_ARG: &str = "co: ::genawaiter::stack::Co<'_, _>";
}

mod sync {
    pub(crate) const CO_ARG_FN: &str = "co: ::genawaiter::sync::Co<";
    pub(crate) const CO_ARG: &str = "co: ::genawaiter::sync::Co<_>";
}

mod rc {
    pub(crate) const CO_ARG_FN: &str = "co: ::genawaiter::rc::Co<";
    pub(crate) const CO_ARG: &str = "co: ::genawaiter::rc::Co<_>";
}

/// Mutates the input `Punctuated<FnArg, Comma>` to a lifetimeless `co:
/// Co<{type}>`.
fn add_coroutine_arg(func: &mut ItemFn, co_ty: &str) {
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
    } else {
        abort!(
            func.sig.span(),
            "arguments are not allowed when using proc_macro"
        )
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
    } else {
        abort!(
            cls.inputs.span(),
            "arguments are not allowed when using proc_macro"
        )
    }
}

fn add_move_async_closure(closure: &mut ExprClosure) {
    closure.asyncness = None;
    closure.capture = None;
    if let Expr::Async(expr) = &mut *closure.body {
        let mv = parse_str::<Token![move]>("move").expect("parse of `move` failed");
        expr.capture = Some(mv);
    } else {
        let non_async = closure.body.clone();
        let async_move_body =
            parse2::<syn::ExprAsync>(quote! { async move { #non_async } })
                .expect("async body parse failed");
        closure.body = Box::new(syn::Expr::Async(async_move_body));
    }
}
