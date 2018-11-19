//! `procopt` is a `structopt`-like library for parsing proc-macro attribute arguments.
//!
//! Coming soon.

#![recursion_limit = "128"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use self::proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::collections::HashMap;
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Expr, Fields, Ident, ItemFn, ItemStruct, Token, Type};

struct Member {
    ty: Ident,
    required: bool,
}

#[proc_macro_attribute]
pub fn procopt(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);

    let struct_name = input.ident.clone();

    let fields = if let Fields::Named(named) = &input.fields {
        named
    } else {
        panic!("Expected named fields.")
    };

    let mut members = HashMap::new();
    let mut decls = Vec::new();

    let mut cases = Vec::new();

    for field in &fields.named {
        let ident = field.ident.clone().unwrap();

        cases.push(ident.clone());

        match field.ty.clone() {
            Type::Verbatim(syn::TypeVerbatim { tts }) => {
                let tts2 = tts.clone().into();
                let name = parse_macro_input!(tts2 as Ident);

                match name.to_string().as_str() {
                    "Option" => {
                        let group = tts.into_iter().next().unwrap();

                        let ty = match group {
                            proc_macro2::TokenTree::Group(group) => {
                                let stream = group.stream().into();
                                parse_macro_input!(stream as Ident)
                            }
                            _ => panic!("Expected group tt."),
                        };

                        decls.push(quote! { let #ident = None; });
                        members.insert(
                            ident.clone(),
                            Member {
                                ty,
                                required: false,
                            },
                        );
                    }
                    _ => {
                        decls.push(quote! { let #ident = None; });
                        members.insert(
                            ident.clone(),
                            Member {
                                ty: name,
                                required: true,
                            },
                        );
                    }
                }
            }
            Type::Array(_) => {}
            _ => {}
        }
    }

    TokenStream::from(quote!{
        #input

        impl syn::Parse for #struct_name {
            fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                #(#decls)*

                while input.peek(syn::Ident) && input.peek2(syn::Token![=]) {
                    let ident = input.parse::<syn::Ident>()?;
                    input.parse::<syn::Token![=]>()?;

                    match ident.to_string().as_str() {
                        #(#cases)*
                        _ => Err(syn::parse::Error::new(ident.span, format!("Unknown option `{}`", ident)))?,
                    }
                }
            }
        }
    })
}
