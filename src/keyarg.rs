use syn::{
    parse::{Parse, ParseStream},
    token, Result,
};

use crate::ext::FnArgExt;

#[derive(Default)]
pub struct KeyArgs {
    pub(crate) optional: Vec<syn::FnArg>,
    pub(crate) required: Vec<syn::FnArg>,
}

impl KeyArgs {
    pub fn iter(&self) -> impl Iterator<Item = &syn::FnArg> {
        self.optional.iter().chain(self.required.iter())
    }
}

impl Parse for KeyArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut this = Self::default();
        while let Ok(arg) = input.parse::<syn::FnArg>() {
            match &*arg.unwrap_to_rhs_type() {
                syn::Type::Path(typ) if typ.path.is_ident("Option") => this.optional.push(arg),
                _ => this.required.push(arg),
            };

            input.parse::<token::Comma>().ok();
        }

        Ok(this)
    }
}
