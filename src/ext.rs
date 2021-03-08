use proc_macro2::Ident;
use syn::{spanned::Spanned, Result};

use crate::keyarg::KeyArgs;

pub trait PatExt {
    fn unwrap_ident(&self) -> Ident;
}

impl PatExt for syn::Pat {
    fn unwrap_ident(&self) -> Ident {
        use syn::Pat;
        match self {
            Pat::Ident(ident) => ident.ident.clone(),
            _ => panic!("Attempted get ident for pattern with no ident."),
        }
    }
}

pub trait FnArgExt {
    fn unwrap_to_rhs_type(&self) -> Box<syn::Type>;
    fn unwrap_typed(&self) -> &syn::PatType;
    fn get_lhs_ident(&self) -> Option<syn::Ident>;
    fn has_keywords_macro_type(&self) -> bool;
    fn parse_keyword_arguments(&self) -> Result<KeyArgs>;
}

pub const MACRO_IDENT: &str = "keywords";

impl FnArgExt for syn::FnArg {
    fn get_lhs_ident(&self) -> Option<syn::Ident> {
        match &*self.unwrap_typed().pat {
            syn::Pat::Ident(pat) => Some(pat.ident.clone()),
            syn::Pat::Wild(_) => None,
            _ => panic!("Attempted to get ident on lhs of fn arg."),
        }
    }

    fn parse_keyword_arguments(&self) -> Result<KeyArgs> {
        match &*self.unwrap_typed().ty {
            syn::Type::Macro(syn::TypeMacro { mac }) => mac.parse_body::<KeyArgs>(),
            _ => Err(syn::Error::new(
                self.span(),
                format!("Expected a macro type for {}.", MACRO_IDENT),
            )),
        }
    }

    fn has_keywords_macro_type(&self) -> bool {
        match &*self.unwrap_typed().ty {
            syn::Type::Macro(syn::TypeMacro { mac }) => mac.path.is_ident(MACRO_IDENT),
            _ => false,
        }
    }

    fn unwrap_typed(&self) -> &syn::PatType {
        match self {
            syn::FnArg::Receiver(_) => panic!("Attempted to get type for `self`."),
            syn::FnArg::Typed(ty) => ty,
        }
    }

    fn unwrap_to_rhs_type(&self) -> Box<syn::Type> {
        match self {
            syn::FnArg::Receiver(_) => panic!("Attempted to get type for `self`."),
            syn::FnArg::Typed(syn::PatType { ty, .. }) => ty.clone(),
        }
    }
}

impl PatExt for syn::FnArg {
    fn unwrap_ident(&self) -> Ident {
        match self {
            syn::FnArg::Receiver(_) => panic!("Attempted to get ident for `self`."),
            syn::FnArg::Typed(syn::PatType { pat, .. }) => pat.unwrap_ident(),
        }
    }
}
