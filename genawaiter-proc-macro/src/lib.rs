extern crate proc_macro;

use std::string::ToString;

use proc_macro::TokenStream;
use proc_macro_error::{abort, abort_call_site, proc_macro_error};
use proc_macro_hack::proc_macro_hack;
use quote::quote;
use syn::{
    self,
    parse_macro_input,
    parse_str,
    spanned::Spanned,
    visit_mut::VisitMut,
    ExprBlock,
    FnArg,
    Ident,
    ItemFn,
    Type,
};

mod visit;
use visit::YieldReplace;

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
    let mut input = parse_macro_input!(input as ExprBlock);

    YieldReplace.visit_expr_block_mut(&mut input);
    // for some reason parsing as a PatType (correct for closures) fails
    // the only way around is to destructure.
    let arg = match parse_str::<FnArg>(stack::CO_ARG) {
        Ok(FnArg::Typed(x)) => x,
        _ => proc_macro_error::abort_call_site!("string Pat parse failed Co<...>"),
    };

    let tokens = quote! { |#arg| async move #input };
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
    let mut input = parse_macro_input!(input as ExprBlock);

    YieldReplace.visit_expr_block_mut(&mut input);
    // for some reason parsing as a PatType (correct for closures) fails
    let arg = match parse_str::<FnArg>(sync::CO_ARG) {
        Ok(FnArg::Typed(x)) => x,
        _ => proc_macro_error::abort_call_site!("string Pat parse failed Co<...>"),
    };

    let tokens = quote! { |#arg| async move #input };
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
    let mut input = parse_macro_input!(input as ExprBlock);

    YieldReplace.visit_expr_block_mut(&mut input);
    // for some reason parsing as a PatType (correct for closures) fails
    let arg = match parse_str::<FnArg>(rc::CO_ARG) {
        Ok(FnArg::Typed(x)) => x,
        _ => proc_macro_error::abort_call_site!("string Pat parse failed Co<...>"),
    };

    let tokens = quote! { |#arg| async move #input };
    tokens.into()
}

mod stack {
    pub(crate) const CO_ARG_FN: &str =
        "__private_co_arg__: ::genawaiter::stack::Co<'_, ";
    pub(crate) const CO_ARG: &str =
        "__private_co_arg__: ::genawaiter::stack::Co<'_, _>";
}

mod sync {
    pub(crate) const CO_ARG_FN: &str = "__private_co_arg__: ::genawaiter::sync::Co<";
    pub(crate) const CO_ARG: &str = "__private_co_arg__: ::genawaiter::sync::Co<_, _>";
}

mod rc {
    pub(crate) const CO_ARG_FN: &str = "__private_co_arg__: ::genawaiter::rc::Co<";
    pub(crate) const CO_ARG: &str = "__private_co_arg__: ::genawaiter::rc::Co<_, _>";
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
