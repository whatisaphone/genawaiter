use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::abort;
use quote::quote;
use syn::{
    parse2,
    spanned::Spanned,
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
                let co_call = quote! { yield_!(@emit => co, #tkns) };
                let cc: Expr = parse2(co_call).expect("parse of Expr failed");
                *expr = cc;
            }
        }

        visit_mut::visit_expr_mut(self, expr)
    }

    /// Aborts compilation if the `co: Co<...>` is found to be used in
    /// anyway other than by this macro.
    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        if let Some(n) = path.get_ident() {
            if n == "co" {
                abort!(path.span(), "you are not able alter the Co<...>")
            }
        }

        visit_mut::visit_path_mut(self, path)
    }
}
