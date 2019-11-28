use proc_macro_error::abort;
use quote::quote;
use syn::{parse2, spanned::Spanned, Expr, Ident, Item, Stmt};

/// Replaces all `yeild_!{ expression }` with `co.yield_(#ident).await;` for
/// closures.
pub(crate) fn replace_yield_cls(body: &mut Expr) {
    match body {
        Expr::Block(block) => {
            let mut stmts = &mut block.block.stmts;
            parse_block_stmts(&mut stmts);
        }
        Expr::Async(block) => {
            let mut stmts = &mut block.block.stmts;
            parse_block_stmts(&mut stmts);
        }
        _ => {}
    }
}

/// Replaces all `yeild_!{ expression }` with `co.yield_(#ident).await;`.
pub(crate) fn parse_block_stmts(stmts: &mut [Stmt]) {
    for stmt in stmts.iter_mut() {
        match stmt {
            Stmt::Expr(Expr::ForLoop(loopy)) => {
                let loop_stmts = &mut loopy.body.stmts;
                let yield_idx = loop_stmts.iter().position(|loop_stmt| {
                    match loop_stmt {
                        Stmt::Item(Item::Macro(m)) => {
                            if let Some(id) = m.mac.path.get_ident() {
                                println!("{:?}", m);
                                id == "yield_"
                            } else {
                                false
                            }
                        }
                        _ => false,
                    }
                });
                if let Some(idx) = yield_idx {
                    let ident = match loop_stmts.get(idx) {
                        Some(Stmt::Item(Item::Macro(yd_expr))) => {
                            let ex: Ident =
                                syn::parse2(yd_expr.mac.tokens.clone()).expect("");
                            quote! { #ex }
                        }
                        _ => panic!("bug found index of yield in stmts but no yield"),
                    };
                    let co_call = quote! { co.yield_(#ident).await; };
                    let cc: Stmt = parse2(co_call).expect("parse of Stmt failed");
                    loop_stmts.remove(idx);
                    loop_stmts.insert(idx, cc);
                }
            }
            Stmt::Expr(Expr::While(loopy)) => {
                let loop_stmts = &mut loopy.body.stmts;
                let yield_idx = loop_stmts.iter().position(|loop_stmt| {
                    match loop_stmt {
                        Stmt::Item(Item::Macro(m)) => {
                            if let Some(id) = m.mac.path.get_ident() {
                                println!("{:?}", m);
                                id == "yield_"
                            } else {
                                false
                            }
                        }
                        _ => false,
                    }
                });
                if let Some(idx) = yield_idx {
                    let ident = match loop_stmts.get(idx) {
                        Some(Stmt::Item(Item::Macro(yd_expr))) => {
                            let ex: Ident =
                                syn::parse2(yd_expr.mac.tokens.clone()).expect("");
                            quote! { #ex }
                        }
                        _ => panic!("bug found index of yield in stmts but no yield"),
                    };
                    let co_call = quote! { co.yield_(#ident).await; };
                    let cc: Stmt = parse2(co_call).expect("parse of Stmt failed");
                    loop_stmts.remove(idx);
                    loop_stmts.insert(idx, cc);
                }
            }
            Stmt::Item(Item::Macro(m)) => {
                if let Some(id) = m.mac.path.get_ident() {
                    if id == "yield_" {
                        let ex: Ident = syn::parse2(m.mac.tokens.clone())
                            .or_else::<(), _>(|e| {
                                abort!(
                                    m.span(),
                                    format!("must yield explicit `()` {}", e)
                                )
                            })
                            .unwrap();
                        let co_call = quote! { co.yield_(#ex).await; };
                        let cc: Stmt = parse2(co_call).expect("parse of Stmt failed");
                        std::mem::replace(stmt, cc);
                    }
                } else {
                    abort!(m.span(), "must yield explicit `()`")
                }
            }
            _ => continue,
        }
    }
}
