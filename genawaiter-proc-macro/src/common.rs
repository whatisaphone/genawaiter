use std::collections::VecDeque;

use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::abort;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse2,
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    Block,
    Expr,
    ExprClosure,
    Item,
    Macro,
    Result as SynResult,
    Stmt,
    Token,
    Type,
};

pub struct YieldMatchMacro {
    pub parent: Option<Stmt>,
    pub collected: Vec<Option<Stmt>>,
    pub coll_replace: Vec<Stmt>,
}

impl YieldMatchMacro {
    pub fn new() -> Self {
        Self {
            parent: None,
            collected: vec![],
            coll_replace: vec![],
        }
    }
}

impl VisitMut for YieldMatchMacro {
    fn visit_stmt_mut(&mut self, node: &mut Stmt) {
        match node {
            Stmt::Local(_local) => {
                self.parent = Some(node.clone());
            }
            Stmt::Item(_item) => {
                self.parent = Some(node.clone());
            }
            Stmt::Expr(_expr) => {
                self.parent = Some(node.clone());
            }
            Stmt::Semi(_expr, _semi) => {
                self.parent = Some(node.clone());
            }
        }
        visit_mut::visit_stmt_mut(self, node);
    }

    fn visit_macro_mut(&mut self, node: &mut Macro) {
        let yield_found = node.path.segments.iter().any(|seg| seg.ident == "yield_");

        if yield_found && self.parent.is_some() {
            // this accepts any valid rust tokens allows Idents/Literals/Macros ect.
            let ex: TokenStream2 =
                syn::parse2(node.tokens.clone()).expect("parse of TokensStream failed");
            let ident = quote! { #ex };
            let co_call = quote! { co.yield_(#ident).await; };
            let cc: Stmt = parse2(co_call).expect("parse of Stmt failed");

            self.coll_replace.push(cc);
            // we use this as a flag for YieldReplace to check and compare the collected
            // Stmt to the Stmt that YieldReplace is currently visiting.
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
    fn visit_stmt_mut(&mut self, node: &mut Stmt) {
        match node {
            Stmt::Local(_local) => {
                if let Some(&None) = self.collected.get(0) {
                    self.collected.pop_front();
                }
            }
            Stmt::Item(_item) => {
                if let Some(&None) = self.collected.get(0) {
                    self.collected.pop_front();
                }
            }
            Stmt::Expr(_expr) => {
                if let Some(&None) = self.collected.get(0) {
                    self.collected.pop_front();
                }
            }
            Stmt::Semi(_expr, _semi) => {
                if let Some(&None) = self.collected.get(0) {
                    self.collected.pop_front();
                }
            }
        }
        visit_mut::visit_stmt_mut(self, node);
    }

    fn visit_block_mut(&mut self, node: &mut Block) {
        for yd_stmt in node.stmts.iter_mut().filter(|yd_stmt| {
            match yd_stmt {
                Stmt::Item(Item::Macro(m)) => {
                    m.mac.path.segments.iter().any(|seg| seg.ident == "yield_")
                }
                Stmt::Expr(Expr::Macro(m)) => {
                    m.mac.path.segments.iter().any(|seg| seg.ident == "yield_")
                }
                Stmt::Semi(Expr::Macro(m), _) => {
                    m.mac.path.segments.iter().any(|seg| seg.ident == "yield_")
                }
                Stmt::Local(box_loc) => {
                    if let Some(local) = &box_loc.init {
                        match &*local.1 {
                            Expr::Macro(m) => {
                                m.mac
                                    .path
                                    .segments
                                    .iter()
                                    .any(|seg| seg.ident == "yield_")
                            }
                            _ => false,
                        }
                    } else {
                        false
                    }
                }
                _ => false,
            }
        }) {
            if let Some(yld) = self.coll_replace.pop_front() {
                match yd_stmt {
                    // for assignment `let foo = yield_!(55);`
                    Stmt::Local(box_loc) => {
                        if let Some(local) = &mut box_loc.init {
                            match &mut *local.1 {
                                Expr::Macro(_macro) => {
                                    match yld {
                                        Stmt::Expr(inner) => *local.1 = inner,
                                        Stmt::Item(_inner) => {
                                            abort!(
                                                box_loc.span(),
                                                "`{}` is not a valid assignment"
                                            )
                                        }
                                        Stmt::Local(_inner) => {
                                            abort!(
                                                box_loc.span(),
                                                "`{}` is not a valid assignment"
                                            )
                                        }
                                        Stmt::Semi(inner, _) => *local.1 = inner,
                                    }
                                }
                                _ => panic!("bug macro found then lost"),
                            }
                        } else {
                            panic!("bug macro found then lost")
                        }
                    }
                    _ => {
                        *yd_stmt = yld;
                    }
                }
            }
        }

        visit_mut::visit_block_mut(self, node);
    }
}

#[derive(Debug)]
pub struct YieldClosure {
    pub ty: Type,
    pub closure: ExprClosure,
}

impl Parse for YieldClosure {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let ty = input.parse::<Type>()?;
        if input.parse::<Token![in]>().is_ok() {
            Ok(YieldClosure {
                ty,
                closure: input.parse()?,
            })
        } else {
            panic!("use `in` keyword ex. 'yield_cls!{ u8 in || yield!(10) }'")
        }
    }
}
