use std::collections::VecDeque;

use proc_macro_error::abort;
use quote::quote;
use syn::{
    parse2,
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    Block,
    Expr,
    ExprMacro,
    ForeignItemMacro,
    Ident,
    ImplItemMacro,
    Item,
    ItemMacro,
    ItemMacro2,
    PatMacro,
    Macro,
    Stmt,
    TraitItemMacro,
    TypeMacro,
};

pub struct YieldMatchMacro {
    pub parent: Option<Stmt>,
    pub collected: Vec<Option<Stmt>>,
    pub coll_replace: Vec<Stmt>,
}

impl YieldMatchMacro {
    pub fn new() -> Self {
        Self { parent: None, collected: vec![], coll_replace: vec![], }
    }
}

impl VisitMut for YieldMatchMacro {
    fn visit_stmt_mut(&mut self, node: &mut Stmt) {
        match node {
            Stmt::Local(l) => {
                self.parent = Some(node.clone());
                println!("LOCAL")
            },
            Stmt::Item(item) => {
                self.parent = Some(node.clone());
                println!("ITEM")
            },
            Stmt::Expr(expr) => {
                self.parent = Some(node.clone());
                println!("EXPR")
            },
            Stmt::Semi(expr, _semi) => {
                self.parent = Some(node.clone());
                println!("SEMI")
            },
        }
        visit_mut::visit_stmt_mut(self, node);
    }

    fn visit_macro_mut(&mut self, node: &mut Macro) {
        println!("EXPR MAC {:#?}", node);
        println!("EXPR MAC {:#?}", self.parent);
        let yield_found = node.path.segments.iter().any(|seg| seg.ident == "yield_");

        if yield_found && self.parent.is_some() {
            let ex: Ident = syn::parse2(node.tokens.clone()).expect("parse of Ident failed");
            let ident = quote! { #ex };

            let co_call = quote! { co.yield_(#ident).await; };
            let cc: Stmt = parse2(co_call).expect("parse of Stmt failed");
            self.coll_replace.push(cc);
            self.collected.push(self.parent.take())
        } else {
            self.collected.push(None)
        }

        visit_mut::visit_macro_mut(self, node);
    }
}

pub struct YieldReplace {
    pub collected: VecDeque<Option<Stmt>>,
    pub coll_replace: VecDeque<Stmt>,
}

impl YieldReplace {
    pub fn new(found: YieldMatchMacro) -> Self {
        Self {
            collected: found.collected.into_iter().collect(),
            coll_replace: found.coll_replace.into_iter().collect(), 
        }
    }
}

impl VisitMut for YieldReplace {
    fn visit_stmt_mut(&mut self, mut node: &mut Stmt) {
        match node {
            Stmt::Local(l) => {
                if self.collected.get(0).is_some() {
                    self.collected.pop_front();
                    println!("OHOHOHOHOHOHOHOHOH");
                } else {
                    self.collected.pop_front();
                }
                
                println!("LOCAL")
            },
            Stmt::Item(item) => {
                if let Some(&Some(_)) = self.collected.get(0) {
                    // self.collected.pop_front();
                    // if let Some(mut yld) = self.coll_replace.pop_front() {
                    //     println!("OHOHOHOHOHOHOHOHOH\n\n {:#?}", node);
                    //     *node = yld;
                    // }
                    println!("OHOHOHOHOHOHOHOHOH");
                } else {
                    self.collected.pop_front();
                }
                println!("ITEM")
            },
            Stmt::Expr(expr) => {
                if let Some(&Some(_)) = self.collected.get(0) {

                    // self.collected.pop_front();
                    // if let Some(mut yld) = self.coll_replace.pop_front() {
                    //     println!("OHOHOHOHOHOHOHOHOH\n\n {:#?}", node);
                    //     *node = yld;
                    // }
                    println!("OHOHOHOHOHOHOHOHOH\n\n{:#?}",self.coll_replace);
                } else {
                    self.collected.pop_front();
                }
                println!("EXPR")
            },
            Stmt::Semi(expr, _semi) => {
                if self.collected.get(0).is_some() {
                    self.collected.pop_front();
                    println!("OHOHOHOHOHOHOHOHOH");
                } else {
                    self.collected.pop_front();
                }
                println!("SEMI")
            },
        }
        visit_mut::visit_stmt_mut(self, node);
    }

    fn visit_block_mut(&mut self, node: &mut Block) {
        println!("{:?}\n{:?}", self.collected, node);
        let coll = self.collected.get(0).unwrap();
        if coll.as_ref() == node.stmts.get(0) {
            if let Some(mut yld) = self.coll_replace.pop_front() {
                println!("OHOHOHOHOHOHOHOHOH\n\n {:#?}", node);
                node.stmts[0] = yld;
            }
        }
        visit_mut::visit_block_mut(self, node);
    }

    fn visit_macro_mut(&mut self, node: &mut Macro) {
        // println!("EXPR MAC {:#?}", node);
        // println!("EXPR MAC {:#?}", self.coll_replace);
        // let yield_found = node.path.segments.iter().any(|seg| seg.ident == "yield_");

        // if yield_found && self.parent.is_some() {
        //     let ex: Ident = syn::parse2(node.tokens.clone()).expect("");
        //     let ident = quote! { #ex };

        //     let co_call = quote! { co.yield_(#ident).await; };
        //     let cc: Stmt = parse2(co_call).expect("parse of Stmt failed");
        //     self.coll_replace.push(cc);
        //     self.collected.push(self.parent.take().unwrap())
        // }

        visit_mut::visit_macro_mut(self, node);
    }
}

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

/// Replaces all `yield_!{ expression }` with `co.yield_(#ident).await;`.
pub(crate) fn parse_block_stmts(stmts: &mut [Stmt]) {
    for stmt in stmts.iter_mut() {
        match stmt {
            Stmt::Expr(Expr::ForLoop(loopy)) => {
                let loop_stmts = &mut loopy.body.stmts;
                let yield_idx = loop_stmts.iter().position(|loop_stmt| {
                    println!("{:?}", loop_stmts);
                    match loop_stmt {
                        Stmt::Item(Item::Macro(m)) => {
                            println!("{:?}", m);
                            m.mac.path.segments.iter().any(|seg| seg.ident == "yield_")
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

// fn visit_item_macro_mut(&mut self, node: &mut ItemMacro) {
//     println!("ITEM MAC");
//     visit_mut::visit_item_macro_mut(self, node);
// }

// fn visit_item_macro2_mut(&mut self, node: &mut ItemMacro2) {
//     println!("ITEM MAC2");
//     visit_mut::visit_item_macro2_mut(self, node);
// }

// fn visit_pat_macro_mut(&mut self, node: &mut PatMacro) {
//     println!("PAT MAC");
//     visit_mut::visit_pat_macro_mut(self, node);
// }
// fn visit_trait_item_macro_mut(&mut self, node: &mut TraitItemMacro) {
//     println!("TRAIT MAC");
//     visit_mut::visit_trait_item_macro_mut(self, node);
// }
// fn visit_type_macro_mut(&mut self, node: &mut TypeMacro) {
//     println!("TYPE MAC");
//     visit_mut::visit_type_macro_mut(self, node);
// }

// fn visit_foreign_item_macro_mut(&mut self, node: &mut ForeignItemMacro) {
//     println!("FOREIGN MAC");
//     visit_mut::visit_foreign_item_macro_mut(self, node);
// }

// fn visit_impl_item_macro_mut(&mut self, node: &mut ImplItemMacro) {
//     println!("IMPL MAC");
//     visit_mut::visit_impl_item_macro_mut(self, node);
// }
