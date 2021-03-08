mod ext;
mod keyarg;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, token, Attribute, Ident, ImplItem, ItemImpl, Result,
};

use self::ext::*;
use self::keyarg::KeyArgs;

const CRATE_ROOT: &str = env!("CARGO_CRATE_NAME");

#[proc_macro_attribute]
pub fn block(
    _args: proc_macro::TokenStream,
    stream: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut block = parse_macro_input!(stream as ItemImpl);
    let mut builders = Vec::new();

    for item in &mut block.items {
        if let ImplItem::Method(method) = item {
            if let Some(i) = method
                .attrs
                .iter()
                .enumerate()
                .filter(|(_, a)| a.path.is_ident(CRATE_ROOT))
                .map(|(i, _)| i)
                .next()
            {
                method.attrs.remove(i);
            }

            if let Ok(keyargs) = syn::parse::<KwargsFn>(method.to_token_stream().into()) {
                builders.push(keyargs.builder().to_token_stream());

                *item = ImplItem::Verbatim(keyargs.to_token_stream());
            }
        }
    }

    let output = quote!(#(#builders)* #block).into();

    println!("{}", output);

    output
}

#[proc_macro_attribute]
pub fn r#fn(
    _args: proc_macro::TokenStream,
    stream: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let kwargs = parse_macro_input!(stream as KwargsFn);
    let builder = kwargs.builder();
    quote!(#builder #kwargs).into()
}

struct KwargsBuilder<'name, 'kwargs, 'generics, 'args, 'block, 'output, 'vis> {
    constness: Option<token::Const>,
    asyncness: Option<token::Async>,
    unsafety: Option<token::Unsafe>,
    name: &'name syn::Ident,
    kwargs: &'kwargs KeyArgs,
    generics: &'generics syn::Generics,
    args: &'args [syn::FnArg],
    block: &'block syn::Block,
    output: &'output syn::ReturnType,
    vis: &'vis Option<syn::Visibility>,
}

fn builder_ident(ident: &syn::Ident) -> syn::Ident {
    let i = heck::CamelCase::to_camel_case(&*format!("{}_builder", ident));
    syn::Ident::new(&i, proc_macro2::Span::call_site())
}

impl ToTokens for KwargsBuilder<'_, '_, '_, '_, '_, '_, '_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let constness = &self.constness;
        let asyncness = &self.asyncness;
        let unsafety = &self.unsafety;
        let args = &self.args;
        let kwargs = &self.kwargs;
        let block = &self.block;
        let output = &self.output;
        let vis = &self.vis;
        let args_idents = args
            .iter()
            .map(|arg| arg.unwrap_ident())
            .collect::<Vec<_>>();
        let kwargs_idents = kwargs
            .iter()
            .map(|arg| arg.unwrap_ident())
            .collect::<Vec<_>>();
        let optional_kwargs_idents = kwargs
            .optional
            .iter()
            .map(|arg| arg.unwrap_ident())
            .collect::<Vec<_>>();
        let kwargs_types = kwargs.iter().map(|arg| arg.unwrap_to_rhs_type());
        let type_params = self
            .generics
            .params
            .iter()
            .filter_map(|gp| match gp {
                syn::GenericParam::Type(param) => Some(param),
                _ => None,
            })
            .collect::<Vec<_>>();

        let kwargs_generics = kwargs
            .iter()
            .map(|arg| {
                let arg = arg.unwrap_typed();
                match &*arg.ty {
                    syn::Type::Path(pat) => {
                        if let Some(tp) = type_params.iter().find(|tp| pat.path.is_ident(&tp.ident))
                        {
                            syn::Generics {
                                lt_token: Some(Default::default()),
                                params: {
                                    let mut punct = syn::punctuated::Punctuated::new();

                                    punct.push(syn::GenericParam::Type((*tp).clone()));

                                    punct
                                },
                                gt_token: Some(Default::default()),
                                where_clause: None,
                            }
                        } else {
                            Default::default()
                        }
                    }
                    _ => Default::default(),
                }
            })
            .collect::<Vec<_>>();

        let optional_args_def = kwargs.optional.iter().map(|arg| {
            let ident = arg.unwrap_ident();
            let typ = arg.unwrap_to_rhs_type();
            quote!(#ident : #typ)
        });
        let required_args_def = kwargs.iter().map(|arg| {
            let ident = arg.unwrap_ident();
            let typ = arg.unwrap_to_rhs_type();
            quote!(#ident : core::option::Option<#typ>)
        });
        let required_args_init = kwargs.required.iter().map(|arg| {
            let ident = arg.unwrap_ident();
            quote!(let #ident = self.#ident.unwrap())
        });
        let builder_name = builder_ident(&self.name);

        tokens.append_all(quote! {
            #vis struct #builder_name {
                #(#args,)*
                #(#required_args_def,)*
                #(#optional_args_def,)*
            }

            impl #builder_name {

                #vis #constness fn new (#(#args),*) -> Self {
                    Self {
                        #(#args_idents,)*
                        #(#kwargs_idents: None,)*
                    }
                }

                #(
                #vis #constness fn #kwargs_idents #kwargs_generics (mut self, #kwargs_idents: #kwargs_types) -> Self {
                    self.#kwargs_idents = Some(#kwargs_idents);
                    self
                }
                )*

                #vis #constness #asyncness #unsafety fn call(self) #output {
                    #(let #args_idents = self.#args_idents;)*
                    #(let #optional_kwargs_idents = self.#optional_kwargs_idents;)*
                    #(#required_args_init;)*

                    #block
                }
            }
        });
    }
}

struct KwargsFn {
    attrs: Vec<Attribute>,
    pub block: syn::Block,
    vis: Option<syn::Visibility>,
    pub constness: Option<token::Const>,
    pub asyncness: Option<token::Async>,
    pub unsafety: Option<token::Unsafe>,
    pub ident: Ident,
    pub generics: syn::Generics,
    pub args: Vec<syn::FnArg>,
    pub kwargs: KeyArgs,
    pub output: syn::ReturnType,
}

impl KwargsFn {
    pub fn builder(&self) -> KwargsBuilder {
        KwargsBuilder {
            constness: self.constness,
            asyncness: self.asyncness,
            unsafety: self.unsafety,
            args: &self.args,
            generics: &self.generics,
            kwargs: &self.kwargs,
            output: &self.output,
            block: &self.block,
            name: &self.ident,
            vis: &self.vis,
        }
    }
}

impl Parse for KwargsFn {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse::<syn::Visibility>().ok();
        let constness = input.parse().ok();
        let asyncness = input.parse().ok();
        let unsafety = input.parse().ok();
        input.parse::<token::Fn>()?;
        let ident = input.parse()?;
        let generics = input.parse()?;
        let mut args = Vec::new();
        let mut kwargs = KeyArgs::default();
        let fn_args;
        syn::parenthesized!(fn_args in input);

        while !fn_args.is_empty() {
            args.push(fn_args.parse::<syn::FnArg>()?);
            fn_args.parse::<token::Comma>().ok();
        }

        if let Some(arg) = args.pop() {
            if arg.get_lhs_ident().is_none() && arg.has_keywords_macro_type() {
                kwargs = arg.parse_keyword_arguments()?;
            } else {
                return Err(syn::Error::new(
                    fn_args.span(),
                    format!(
                        "Expected a `{}` macro as the last argument.",
                        ext::MACRO_IDENT
                    ),
                ));
            }
        }

        let output = input.parse()?;

        let block = input.parse()?;

        Ok(Self {
            args,
            asyncness,
            attrs,
            block,
            constness,
            generics,
            ident,
            kwargs,
            output,
            unsafety,
            vis,
        })
    }
}

impl ToTokens for KwargsFn {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let args = &self.args;
        let attrs = &self.attrs;
        let constness = &self.constness;
        let generics = &self.generics;
        let ident = &self.ident;
        let vis = &self.vis;

        let args_idents = args
            .iter()
            .map(|arg| arg.unwrap_ident())
            .collect::<Vec<_>>();
        let builder_name = builder_ident(ident);

        tokens.append_all(quote! {
            #(#attrs)*
            #vis
            #constness
            fn
            #ident
            #generics
            (#(#args),*)
                -> #builder_name
            {
                #builder_name::new(#(#args_idents,)*)
            }
        });
    }
}
