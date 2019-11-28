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

/// Mutates the input `Punctuated<FnArg, Comma>` to a `co: Co<'_, {type}>` with lifetime.
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
        let co_arg: FnArg = match parse_str::<FnArg>(&format!("co: ::genawaiter::stack::Co<'_, {}>", co_ty))
        {
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
        let arg = match parse_str::<FnArg>(&format!("co: ::genawaiter::stack::Co<'_, {}>", co_ty)) {
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

// pub(crate) fn replace_yield_cls(body: &mut Expr) {
//     match body {
//         Expr::Block(block) => {
//             let mut stmts = &mut block.block.stmts;
//             parse_block_stmts(&mut stmts);
//         }
//         Expr::Async(block) => {
//             let mut stmts = &mut block.block.stmts;
//             parse_block_stmts(&mut stmts);
//         }
//         _ => {}
//     }
// }

// pub(crate) fn parse_block_stmts(stmts: &mut [Stmt]) {
//     for stmt in stmts.iter_mut() {
//         match stmt {
//             Stmt::Expr(Expr::ForLoop(loopy)) => {
//                 let loop_stmts = &mut loopy.body.stmts;
//                 let yield_idx = loop_stmts.iter().position(|loop_stmt| {
//                     match loop_stmt {
//                         Stmt::Item(Item::Macro(m)) => {
//                             if let Some(id) = m.mac.path.get_ident() {
//                                 println!("{:?}", m);
//                                 id == "yield_"
//                             } else { false }
//                         },
//                         _ => false,
//                     }
//                 });
//                 if let Some(idx) = yield_idx {
//                     let ident = match loop_stmts.get(idx) {
//                         Some(Stmt::Item(Item::Macro(yd_expr))) => {
//                             let ex: Ident = syn::parse2(yd_expr.mac.tokens.clone()).expect("");
//                             quote! { #ex }
//                         },
//                         _ => panic!("bug found index of yield in stmts but no yield"),
//                     };
//                     let co_call = quote! { co.yield_(#ident).await; };
//                     let cc: Stmt = parse2(co_call).expect("parse of Stmt failed");
//                     loop_stmts.remove(idx);
//                     loop_stmts.insert(idx, cc);
//                 }
//             }
//             Stmt::Expr(Expr::While(loopy)) => {
//                 let loop_stmts = &mut loopy.body.stmts;
//                 let yield_idx = loop_stmts.iter().position(|loop_stmt| {
//                     match loop_stmt {
//                         Stmt::Item(Item::Macro(m)) => {
//                             if let Some(id) = m.mac.path.get_ident() {
//                                 println!("{:?}", m);
//                                 id == "yield_"
//                             } else {
//                                 false
//                             }
//                         },
//                         _ => false,
//                     }
//                 });
//                 if let Some(idx) = yield_idx {
//                     let ident = match loop_stmts.get(idx) {
//                         Some(Stmt::Item(Item::Macro(yd_expr))) => {
//                             let ex: Ident = syn::parse2(yd_expr.mac.tokens.clone()).expect("");
//                             quote! { #ex }
//                         }
//                         _ => panic!("bug found index of yield in stmts but no yield"),
//                     };
//                     let co_call = quote! { co.yield_(#ident).await; };
//                     let cc: Stmt = parse2(co_call).expect("parse of Stmt failed");
//                     loop_stmts.remove(idx);
//                     loop_stmts.insert(idx, cc);
//                 }
//             },
//             Stmt::Item(Item::Macro(m)) => {
//                 if let Some(id) = m.mac.path.get_ident() {
//                     if id == "yield_" {
//                         let ex: Ident = syn::parse2(m.mac.tokens.clone())
//                             .or_else::<(), _>(|e| {
//                                 abort!(m.span(), format!("must yield explicit `()` {}", e))
//                             })
//                             .unwrap();
//                         let co_call = quote! { co.yield_(#ex).await; };
//                         let cc: Stmt = parse2(co_call).expect("parse of Stmt failed");
//                         std::mem::replace(stmt, cc);
//                     }
//                 } else {
//                     abort!(m.span(), "must yield explicit `()`")
//                 }
//             },
//             _ => continue,
//         }
//     }
// }
