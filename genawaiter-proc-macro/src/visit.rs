use std::collections::VecDeque;

use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::abort;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse2,
    parse_str,
    token::{Async, Move},
    visit::{self, Visit},
    visit_mut::{self, VisitMut},
    Expr,
    ExprClosure,
    Macro,
    Result as SynResult,
    Stmt,
    Token,
    Type,
};

pub struct YieldReplace;

impl VisitMut for YieldReplace {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
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
}

pub struct YieldClosure {
    pub closure: ExprClosure,
}

impl Parse for YieldClosure {
    fn parse(input: ParseStream) -> SynResult<Self> {
        Ok(YieldClosure {
            closure: input.parse()?,
        })
    }
}
