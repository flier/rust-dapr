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
    if std::env::var_os("DUMP_SERVICE").is_some() {
        println!("{}", rustfmt(expanded.to_string()));
    }
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
    if std::env::var_os("DUMP_STUB").is_some() {
        println!("{}", rustfmt(expanded.to_string()));
    }
    TokenStream::from(expanded)
}

fn rustfmt(input: String) -> String {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new("rustfmt")
        .args(&["--emit", "stdout", "--color", "auto", "--edition", "2018"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("`rustfmt` command failed to start");
    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    stdin
        .write_all(input.as_bytes())
        .expect("Failed to write to stdin");
    let output = child.wait_with_output().expect("Failed to read stdout");
    String::from_utf8(output.stdout).unwrap()
}
