use std::collections::VecDeque;

use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::abort;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse2,
    spanned::Spanned,
    visit::{self, Visit},
    visit_mut::{self, VisitMut},
    Block,
    Expr,
    ExprClosure,
    FnArg,
    Item,
    Macro,
    Result as SynResult,
    Stmt,
    Token,
    Type,
};

pub struct YieldMatchMacro {
    pub parent: Option<Stmt>,
    pub fnd_stmts: Vec<Option<Stmt>>,
    pub stmt_rep: Vec<Stmt>,
}

impl YieldMatchMacro {
    pub fn new() -> Self {
        Self {
            parent: None,
            fnd_stmts: vec![],
            stmt_rep: vec![],
        }
    }
}

impl Visit<'_> for YieldMatchMacro {
    fn visit_stmt(&mut self, node: &Stmt) {
        self.parent = Some(node.clone());

        visit::visit_stmt(self, node);
    }

    fn visit_macro(&mut self, node: &Macro) {
        let yield_found = node.path.segments.iter().any(|seg| seg.ident == "yield_");

        if yield_found && self.parent.is_some() {
            // this accepts any valid rust tokens allows Idents/Literals/Macros ect.
            let tkns: TokenStream2 =
                syn::parse2(node.tokens.clone()).expect("parse of TokensStream failed");
            let ident = quote! { #tkns };
            let co_call = quote! { co.yield_(#ident).await; };
            let cc: Stmt = parse2(co_call).expect("parse of Stmt failed");

            self.stmt_rep.push(cc);
            // we use this as a flag for YieldReplace to check and compare the collected
            // Stmt to the Stmt that YieldReplace is currently visiting.
            self.fnd_stmts.push(self.parent.take())
        } else {
            self.fnd_stmts.push(None)
        }

        visit::visit_macro(self, node);
    }
}

pub struct YieldReplace {
    pub fnd_stmts: VecDeque<Option<Stmt>>,
    pub stmt_rep: VecDeque<Stmt>,
}

impl YieldReplace {
    pub fn new(found: YieldMatchMacro) -> Self {
        Self {
            fnd_stmts: found.fnd_stmts.into_iter().collect(),
            stmt_rep: found.stmt_rep.into_iter().collect(),
        }
    }
}

impl VisitMut for YieldReplace {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        if let Some(&None) = self.fnd_stmts.get(0) {
            self.fnd_stmts.pop_front();
        }

        if let Expr::Macro(m) = expr {
            if m.mac.path.segments.iter().any(|seg| seg.ident == "yield_") {
                let tkns: TokenStream2 = syn::parse2(m.mac.tokens.clone())
                    .expect("parse of TokensStream failed");
                let ident = quote! { #tkns };
                let co_call = quote! { co.yield_(#ident).await };
                let cc: Expr = parse2(co_call).expect("parse of Expr failed");
                *expr = cc;
            }
        }

        visit_mut::visit_expr_mut(self, expr)
    }
    fn visit_stmt_mut(&mut self, node: &mut Stmt) {
        if let Some(&None) = self.fnd_stmts.get(0) {
            self.fnd_stmts.pop_front();
        }

        visit_mut::visit_stmt_mut(self, node)
    }

    fn visit_block_mut(&mut self, node: &mut Block) {
        for yd_stmt in node.stmts.iter_mut().filter(matching_yield_macro) {
            if let Some(await_stmt) = self.stmt_rep.pop_front() {
                match yd_stmt {
                    // for assignment `let foo = yield_!(55);`
                    Stmt::Local(box_loc) => {
                        if let Some(local) = &mut box_loc.init {
                            match &mut *local.1 {
                                Expr::Macro(_macro) => {
                                    match await_stmt {
                                        Stmt::Expr(inner) => *local.1 = inner,
                                        Stmt::Semi(inner, _) => *local.1 = inner,
                                        _ => {
                                            abort!(
                                                box_loc.span(),
                                                "`{}` is not a valid assignment"
                                            )
                                        }
                                    }
                                }
                                _ => panic!("BUG macro found then lost"),
                            }
                        } else {
                            panic!("BUG macro found then lost")
                        }
                    }
                    _ => {
                        *yd_stmt = await_stmt;
                    }
                }
            }
        }

        visit_mut::visit_block_mut(self, node)
    }
}

fn matching_yield_macro(yd_stmt: &&mut Stmt) -> bool {
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
                        m.mac.path.segments.iter().any(|seg| seg.ident == "yield_")
                    }
                    _ => false,
                }
            } else {
                false
            }
        }
        _ => false,
    }
}

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
            Err(input.error("use `in` keyword ex. 'yield_cls!{ u8 in || yield!(10) }'"))
        }
    }
}
