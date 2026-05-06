#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Block, ExprClosure, Result, Token, Type,
};

/// Creates a async closure
#[proc_macro]
//#[deprecated(since="0.4.0-beta.3", note="Use the async closure syntax instead.")]
pub fn async_closure(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ExprClosure);
    let body = input.body;
    let inputs = input.inputs;
    let mova = input.capture;
    let expanded = quote! {{
        use std::future::IntoFuture;
        #mova |#inputs| Box::pin(async move { #body })
    }};
    expanded.into()
}

struct If<T: Parse> {
    type_to_match: Type,
    input_type: Type,
    true_branch: T,
    false_branch: T,
}

impl<T: Parse> Parse for If<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let type_to_match: Type = input.parse()?;
        input.parse::<Token![,]>()?;
        let input_type: Type = input.parse()?;
        input.parse::<Token![,]>()?;
        let true_branch: T = input.parse()?;
        input.parse::<Token![,]>()?;
        let false_branch: T = input.parse()?;
        Ok(If {
            type_to_match,
            input_type,
            true_branch,
            false_branch,
        })
    }
}

/// Creates a compile time if statement
/// that takes checks if 2 types are the same
/// and if returns one of the branches
#[proc_macro]
pub fn block_if(input: TokenStream) -> TokenStream {
    let If {
        type_to_match,
        input_type,
        true_branch,
        false_branch,
    } = parse_macro_input!(input as If<Block>);
    let used_branch = if type_to_match.to_token_stream().to_string()
        == input_type.to_token_stream().to_string()
    {
        true_branch
    } else {
        false_branch
    }
    .stmts;
    let mut strm = TokenStream2::new();
    for stmt in used_branch {
        stmt.to_tokens(&mut strm)
    }
    strm.into()
}

/// Creates a compile time if statement
/// that takes checks if 2 types are the same
/// and if returns one of the branches
#[proc_macro]
pub fn type_if(input: TokenStream) -> TokenStream {
    let If {
        type_to_match,
        input_type,
        true_branch,
        false_branch,
    } = parse_macro_input!(input as If<Type>);
    let used_branch = if type_to_match.to_token_stream().to_string()
        == input_type.to_token_stream().to_string()
    {
        true_branch
    } else {
        false_branch
    };
    used_branch.into_token_stream().into()
}

/// Creates a compile time if statement
/// that takes checks if 2 types are the same
/// and if returns one of the branches
#[proc_macro]
pub fn expr_if(input: TokenStream) -> TokenStream {
    let If {
        type_to_match,
        input_type,
        true_branch,
        false_branch,
    } = parse_macro_input!(input as If<syn::Expr>);
    let used_branch = if type_to_match.to_token_stream().to_string()
        == input_type.to_token_stream().to_string()
    {
        true_branch
    } else {
        false_branch
    };
    used_branch.into_token_stream().into()
}
