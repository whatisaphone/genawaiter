extern crate proc_macro;

use proc_macro_error::*;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    self,
    parse2,
    parse_macro_input,
    parse_str,
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    Block,
    Expr,
    FnArg,
    Ident,
    Item,
    ItemFn,
    Pat,
    Stmt,
    Type,
};

#[proc_macro_attribute]
#[proc_macro_error]
pub fn yielder_fn(args: TokenStream, input: TokenStream) -> TokenStream {
    // make sure it is a valid type
    let a = args.clone();
    let _ = parse_macro_input!(a as Type);
    let mut function = parse_macro_input!(input as ItemFn);

    let co_type = args.to_string();
    add_coroutine_arg(&mut function.sig.inputs, co_type);
    replace_yield(&mut *function.block);

    let tokens = quote! { #function };
    tokens.into()
}

fn add_coroutine_arg(punct: &mut Punctuated<FnArg, Comma>, co_ty: String) {
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
        let co_arg: FnArg = match parse_str::<FnArg>(&format!("co: Co<'_, {}>", co_ty))
        {
            Ok(s) => s,
            Err(err) => abort_call_site!(format!("invalid type for Co yield {}", err)),
        };
        punct.push_value(co_arg)
    }
}

fn replace_yield(blk: &mut Block) {
    let stmts = &mut blk.stmts;

    for stmt in stmts.iter_mut() {
        println!("{:#?}", stmt);
        match stmt {
            Stmt::Expr(Expr::ForLoop(loopy)) => {
                println!("{:#?}", loopy);
                let loop_stmts = &mut loopy.body.stmts;
                let yield_idx = loop_stmts.iter().position(|loop_stmt| {
                    match loop_stmt {
                        Stmt::Item(Item::Macro(m)) => {
                            if let Some(id) = m.mac.path.get_ident() {
                                println!("{:?}", m);
                                id == "yield_"
                            } else { false }
                        },
                        // Stmt::Item(Item::Macro(m)) => {
                        //     if let Some(id) = m.mac.path.get_ident() {
                        //         println!("{:?}", m);
                        //         id == "yield_"
                        //     } else { false }
                        // }
                        _ => false,
                    }
                });
                if let Some(idx) = yield_idx {
                    let ident = match loop_stmts.get(idx) {
                        Some(Stmt::Item(Item::Macro(yd_expr))) => {
                            let ex: Ident = syn::parse2(yd_expr.mac.tokens.clone()).expect("");
                            quote! { #ex }
                        }
                        // Some(Stmt::Semi(Expr::Yield(yd_expr), _)) => {
                        //     if let Some(ex) = yd_expr.expr.as_ref() {
                        //         quote! { #ex }
                        //     } else {
                        //         abort!(yd_expr.span(), "must yield explicit `()`")
                        //     }
                        // }
                        _ => panic!("bug found index of yield in stmts but no yield"),
                    };
                    let co_call = quote! { co.yield_(#ident).await; };
                    let cc: Stmt = parse2(co_call).expect("parse of Stmt failed");
                    loop_stmts.remove(idx);
                    loop_stmts.insert(idx, cc);
                }
            },
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
                        },
                        _ => false,
                    }
                });
                if let Some(idx) = yield_idx {
                    let ident = match loop_stmts.get(idx) {
                        Some(Stmt::Item(Item::Macro(yd_expr))) => {
                            let ex: Ident = syn::parse2(yd_expr.mac.tokens.clone()).expect("");
                            quote! { #ex }
                        }
                        // Some(Stmt::Semi(Expr::Yield(yd_expr), _)) => {
                        //     if let Some(ex) = yd_expr.expr.as_ref() {
                        //         quote! { #ex }
                        //     } else {
                        //         abort!(yd_expr.span(), "must yield explicit `()`")
                        //     }
                        // }
                        _ => panic!("bug found index of yield in stmts but no yield"),
                    };
                    let co_call = quote! { co.yield_(#ident).await; };
                    let cc: Stmt = parse2(co_call).expect("parse of Stmt failed");
                    loop_stmts.remove(idx);
                    loop_stmts.insert(idx, cc);
                }
            },
            Stmt::Item(Item::Macro(m)) => {
                if let Some(id) = m.mac.path.get_ident() {
                    if id == "yield_" {
                        let ex: Ident = syn::parse2(m.mac.tokens.clone())
                            .or_else::<(), _>(|e| {
                                abort!(m.span(), format!("must yield explicit `()` {}", e))
                            })
                            .unwrap();
                        let co_call = quote! { co.yield_(#ex).await; };
                        let cc: Stmt = parse2(co_call).expect("parse of Stmt failed");
                        std::mem::replace(stmt, cc);
                    }
                } else {
                    abort!(m.span(), "must yield explicit `()`")
                }
            },
            // Stmt::Expr(Expr::Yield(yd_expr)) => {
            //     if let Some(ex) = yd_expr.expr.as_ref() {

            //         let co_call = quote! { co.yield_(#ex).await; };
            //         let cc: Stmt = parse2(co_call).expect("parse of Stmt failed");
            //         std::mem::replace(stmt, cc);
            //     } else {
            //         abort!(yd_expr.span(), "must yield explicit `()`")
            //     }
            // }
            // Stmt::Semi(Expr::Yield(yd_expr), _) => {
            //     if let Some(ex) = yd_expr.expr.as_ref() {
            //         let co_call = quote! { co.yield_(#ex).await; };
            //         let cc: Stmt = parse2(co_call).expect("parse of Stmt failed");
            //         std::mem::replace(stmt, cc);
            //     } else {
            //         abort!(yd_expr.span(), "must yield explicit `()`")
            //     }
            // }
            _ => continue,
        }
    }
}

#[proc_macro_attribute]
pub fn yielder_cls(args: TokenStream, input: TokenStream) -> TokenStream {
    // make sure it is a valid type
    let a = args.clone();
    let _ = parse_macro_input!(a as Type);
    let mut function: Stmt = parse_macro_input!(input);

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
    let tokens = quote! { #function };
    tokens.into()
}

fn add_coroutine_arg_cls(punct: &mut Punctuated<Pat, Comma>, co_ty: String) {
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
        let arg = match parse_str::<FnArg>(&format!("co: Co<'_, {}>", co_ty)) {
            Ok(FnArg::Typed(x)) => x,
            _ => abort_call_site!("string Pat parse failed Co<...>"),
        };
        punct.push(Pat::Type(arg))
    }
}

fn replace_yield_cls(body: &mut Expr) {
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

fn parse_block_stmts(stmts: &mut [Stmt]) {
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
                            } else { false }
                        },
                        _ => false,
                    }
                });
                if let Some(idx) = yield_idx {
                    let ident = match loop_stmts.get(idx) {
                        Some(Stmt::Item(Item::Macro(yd_expr))) => {
                            let ex: Ident = syn::parse2(yd_expr.mac.tokens.clone()).expect("");
                            quote! { #ex }
                        },
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
                        },
                        _ => false,
                    }
                });
                if let Some(idx) = yield_idx {
                    let ident = match loop_stmts.get(idx) {
                        Some(Stmt::Item(Item::Macro(yd_expr))) => {
                            let ex: Ident = syn::parse2(yd_expr.mac.tokens.clone()).expect("");
                            quote! { #ex }
                        }
                        _ => panic!("bug found index of yield in stmts but no yield"),
                    };
                    let co_call = quote! { co.yield_(#ident).await; };
                    let cc: Stmt = parse2(co_call).expect("parse of Stmt failed");
                    loop_stmts.remove(idx);
                    loop_stmts.insert(idx, cc);
                }
            },
            Stmt::Item(Item::Macro(m)) => {
                if let Some(id) = m.mac.path.get_ident() {
                    if id == "yield_" {
                        let ex: Ident = syn::parse2(m.mac.tokens.clone())
                            .or_else::<(), _>(|e| {
                                abort!(m.span(), format!("must yield explicit `()` {}", e))
                            })
                            .unwrap();
                        let co_call = quote! { co.yield_(#ex).await; };
                        let cc: Stmt = parse2(co_call).expect("parse of Stmt failed");
                        std::mem::replace(stmt, cc);
                    }
                } else {
                    abort!(m.span(), "must yield explicit `()`")
                }
            },
            _ => continue,
        }
    }
}
