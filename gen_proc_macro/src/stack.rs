use proc_macro::TokenStream;
use proc_macro_error::abort_call_site;
use syn::{
    parse_str,
    punctuated::Punctuated,
    token::Comma,
    Expr,
    FnArg,
    Ident,
    Pat,
    Stmt,
    Type,
};

use crate::common::replace_yield_cls;

/// Mutates the input `Punctuated<FnArg, Comma>` to a `co: Co<'_, {type}>` with
/// lifetime.
pub(crate) fn add_coroutine_arg(punct: &mut Punctuated<FnArg, Comma>, co_ty: String) {
    if !punct.iter().any(|input| {
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
    }) {
        let co_arg: FnArg = match parse_str::<FnArg>(&format!(
            "co: ::genawaiter::stack::Co<'_, {}>",
            co_ty
        )) {
            Ok(s) => s,
            Err(err) => abort_call_site!(format!("invalid type for Co yield {}", err)),
        };
        punct.push_value(co_arg)
    }
}

/// Mutates the input `Punctuated<FnArg, Comma>` to `co: Co<'_ {type}>`
/// with lifetime for closures.
pub(crate) fn add_coroutine_arg_cls(punct: &mut Punctuated<Pat, Comma>, co_ty: String) {
    if !punct.iter().any(|input| {
        match input {
            Pat::Type(arg) => {
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
            _ => false,
        }
    }) {
        let arg = match parse_str::<FnArg>(&format!(
            "co: ::genawaiter::stack::Co<'_, {}>",
            co_ty
        )) {
            Ok(FnArg::Typed(x)) => x,
            _ => abort_call_site!("string Pat parse failed Co<...>"),
        };
        punct.push(Pat::Type(arg))
    }
}

/// Parses a `Stmt` to find closure then calls `add_coroutine_arg_cls`
/// and `replace_yield_cls`.
pub(crate) fn parse_cls(args: TokenStream, mut function: &mut Stmt) {
    let co_type = args.to_string();
    match &mut function {
        Stmt::Local(loc) => {
            if let Some((_eq, expr)) = loc.init.as_mut() {
                match &mut **expr {
                    Expr::Closure(closure) => {
                        add_coroutine_arg_cls(&mut closure.inputs, co_type.clone());
                        replace_yield_cls(&mut *closure.body);
                    }
                    Expr::Call(call) => {
                        for arg in call.args.iter_mut() {
                            match arg {
                                Expr::Closure(closure) => {
                                    add_coroutine_arg_cls(
                                        &mut closure.inputs,
                                        co_type.clone(),
                                    );
                                    replace_yield_cls(&mut *closure.body);
                                }
                                _ => {}
                            }
                        }
                    }
                    Expr::Unsafe(unsf) => {
                        for stmt in unsf.block.stmts.iter_mut() {
                            match stmt {
                                Stmt::Expr(Expr::Call(call)) => {
                                    for arg in call.args.iter_mut() {
                                        match arg {
                                            Expr::Closure(closure) => {
                                                add_coroutine_arg_cls(
                                                    &mut closure.inputs,
                                                    co_type.clone(),
                                                );
                                                replace_yield_cls(&mut *closure.body);
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}
