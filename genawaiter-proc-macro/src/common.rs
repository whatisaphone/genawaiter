use std::collections::VecDeque;

use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::abort;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse2,
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    visit::{self, Visit},
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
    pub fnd_stmts: Vec<Option<Stmt>>,
    pub stmt_rep: Vec<Stmt>,
    pub fnd_exprs: Vec<Option<Expr>>,
    pub keep_exprs: Vec<Option<Expr>>,
    pub expr_rep: Vec<Expr>,
}

impl YieldMatchMacro {
    pub fn new() -> Self {
        Self {
            parent: None,
            fnd_stmts: vec![],
            stmt_rep: vec![],
            fnd_exprs: vec![],
            keep_exprs: vec![],
            expr_rep: vec![],
        }
    }
}

impl Visit<'_> for YieldMatchMacro {
    fn visit_expr(&mut self, expr: &Expr) {
        self.fnd_exprs.push(Some(expr.clone()));
        visit::visit_expr(self, expr);
    }
    fn visit_stmt(&mut self, node: &Stmt) {
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

        if let Some(&Some(Expr::Macro(mac))) = self.fnd_exprs.last().as_ref() {
            if mac.mac.path.segments.iter().any(|seg| seg.ident == "yield_") {
                println!("POP");

                let _ = self.fnd_exprs.pop();

                let tkns: TokenStream2 =
                    syn::parse2(node.tokens.clone()).expect("parse of TokensStream failed");
                let ident = quote! { #tkns };
                let co_call = quote! { co.yield_(#ident).await };
                let cc: Expr = parse2(co_call).expect("parse of Expr failed");
                
                if let Some(Some(Expr::MethodCall(mut call))) = self.fnd_exprs.pop() {
                    self.keep_exprs.push(Some(Expr::MethodCall(call.clone())));
                    call.receiver = Box::new(cc);
                    println!("CALLLLL {:#?}", call);
                    self.expr_rep.push(Expr::MethodCall(call));
                }
            } else {
                self.keep_exprs.push(None);
            }
        } else {
            self.keep_exprs.push(None);
        }

        visit::visit_macro(self, node);
    }
}

pub struct YieldReplace {
    pub fnd_stmts: VecDeque<Option<Stmt>>,
    pub stmt_rep: VecDeque<Stmt>,
    pub fnd_expr: VecDeque<Option<Expr>>,
    pub expr_rep: VecDeque<Expr>,
}

impl YieldReplace {
    pub fn new(found: YieldMatchMacro) -> Self {
        Self {
            fnd_stmts: found.fnd_stmts.into_iter().collect(),
            stmt_rep: found.stmt_rep.into_iter().collect(),
            fnd_expr: found.keep_exprs.into_iter().collect(),
            expr_rep: found.expr_rep.into_iter().collect(),
        }
    }
}

impl VisitMut for YieldReplace {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        if let Some(&None) = self.fnd_stmts.get(0) {
            self.fnd_stmts.pop_front();
        }

        if let Some(&None) = self.fnd_expr.get(0) {
            self.fnd_expr.pop_front();
        } else if let Some(&Some(old)) = self.fnd_expr.get(0).as_ref() {
            if old == expr {
                println!("{:?}\n{:#?}", self.expr_rep, expr);
                if let Some(new_expr) = self.expr_rep.remove(0) {
                    *expr = new_expr;
                }
            }
        }

        visit_mut::visit_expr_mut(self, expr);
    }
    fn visit_stmt_mut(&mut self, node: &mut Stmt) {
        match node {
            Stmt::Local(_local) => {
                if let Some(&None) = self.fnd_stmts.get(0) {
                    self.fnd_stmts.pop_front();
                }
            }
            Stmt::Item(_item) => {
                if let Some(&None) = self.fnd_stmts.get(0) {
                    self.fnd_stmts.pop_front();
                }
            }
            Stmt::Expr(_expr) => {
                if let Some(&None) = self.fnd_stmts.get(0) {
                    self.fnd_stmts.pop_front();
                }
            }
            Stmt::Semi(_expr, _semi) => {
                if let Some(&None) = self.fnd_stmts.get(0) {
                    self.fnd_stmts.pop_front();
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
            if let Some(yld) = self.stmt_rep.pop_front() {
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
            Err(input.error("use `in` keyword ex. 'yield_cls!{ u8 in || yield!(10) }'"))
        }
    }
}
