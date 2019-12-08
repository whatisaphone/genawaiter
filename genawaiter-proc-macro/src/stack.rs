use proc_macro_error::abort_call_site;
use syn::{parse_str, punctuated::Punctuated, token::Comma, FnArg, Ident, Pat, Type};

/// Mutates the input `Punctuated<FnArg, Comma>` to a `co: Co<'_, {type}>` with
/// lifetime.
pub(crate) fn add_coroutine_arg(punct: &mut Punctuated<FnArg, Comma>, co_ty: String) {
    let co_arg_found = punct.iter().any(|input| {
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
    });
    if !co_arg_found {
        let co_arg: FnArg = match parse_str::<FnArg>(&format!(
            "co: ::genawaiter::stack::Co<'_, {}>",
            co_ty
        )) {
            Ok(s) => s,
            Err(err) => abort_call_site!(format!("invalid type for Co yield {}", err)),
        };
        punct.push_value(co_arg)
    }
}

/// Mutates the input `Punctuated<Pat, Comma>` to `co: Co<'_ {type}>`
/// with lifetime for closures.
pub(crate) fn add_coroutine_arg_cls(punct: &mut Punctuated<Pat, Comma>, co_ty: String) {
    let co_arg_found = punct.iter().any(|input| {
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
    });
    if !co_arg_found {
        let arg = match parse_str::<FnArg>(&format!(
            "co: ::genawaiter::stack::Co<'_, {}>",
            co_ty
        )) {
            Ok(FnArg::Typed(x)) => x,
            _ => abort_call_site!("string Pat parse failed Co<...>"),
        };
        punct.push(Pat::Type(arg))
    }
}
