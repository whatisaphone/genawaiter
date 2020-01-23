use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse2,
    visit_mut::{self, VisitMut},
    Expr,
};

pub struct YieldReplace;

impl VisitMut for YieldReplace {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        if let Expr::Macro(m) = expr {
            if m.mac.path.segments.iter().any(|seg| seg.ident == "yield_") {
                let tkns: TokenStream2 = syn::parse2(m.mac.tokens.clone())
                    .expect("parse of TokensStream failed");

                let co_call = quote! { yield_!(@emit => __private_co_arg__, #tkns) };
                let cc: Expr = parse2(co_call).expect("parse of Expr failed");
                *expr = cc;
            }
        }

        visit_mut::visit_expr_mut(self, expr)
    }
}
