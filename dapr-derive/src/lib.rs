extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

mod client;
mod stub;

/// Implements server side interface for the Dapr service
#[proc_macro_attribute]
pub fn service(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as syn::ItemTrait);
    let client = client::dapr_client(&item);
    let expanded = quote! {
        #item
        #client
    };
    TokenStream::from(expanded)
}

/// Implements client site stub for the Dapr service
#[proc_macro_attribute]
pub fn stub(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as syn::ItemTrait);
    let stub = stub::dapr_stub(&item);
    let syn::ItemTrait {
        attrs,
        vis,
        ident,
        generics,
        colon_token,
        supertraits,
        mut items,
        ..
    } = item;
    let (_impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let items = items.iter_mut().map(|item| {
        if let syn::TraitItem::Method(ref mut method) = item {
            method.sig.output = stub::output_type(&method.sig.output);

            quote! { async #method }
        } else {
            quote! { #item }
        }
    });

    let expanded = quote! {
        #[::dapr::async_trait]
        #(#attrs)* #vis trait #ident #ty_generics #colon_token #supertraits #where_clause {
            #(#items)*
        }
        #stub
    };
    println!("{}", expanded);
    TokenStream::from(expanded)
}
