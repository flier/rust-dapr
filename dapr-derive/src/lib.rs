extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

mod args;
mod expand;
mod parse;

use crate::args::Args;
use crate::expand::expand;
use crate::parse::Item;

#[proc_macro_attribute]
pub fn service(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as Args);
    let mut item = parse_macro_input!(input as Item);
    expand(&mut item, args);
    TokenStream::from(quote!(#item))
}
