use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::TraitItem;

use crate::stub::output_type;

pub fn dapr_mock(item: &syn::ItemTrait) -> TokenStream {
    let mock_struct = create_mock_struct(item);
    let mock_trait = impl_mock_trait(item);

    quote! {
        #mock_struct
        #mock_trait
    }
}

fn create_mock_struct(item: &syn::ItemTrait) -> TokenStream {
    let syn::ItemTrait { ident, items, .. } = item;
    let trait_name = &ident;
    let mock_name = format_ident!("{}Mock", trait_name);

    let methods = items.iter().filter_map(|item| {
        if let TraitItem::Method(method) = item {
            Some(method)
        } else {
            None
        }
    });

    let expect_methods = methods.map(|method| {
        let method_name = method.sig.ident.to_string();
        let expect_method = format_ident!("expect_{}", method_name);
        let args = method.sig.inputs.iter().flat_map(|input| {
            if let syn::FnArg::Typed(syn::PatType { ty, .. }) = input {
                Some(ty)
            } else {
                None
            }
        });
        let output = match output_type(&method.sig.output) {
            syn::ReturnType::Type(_, ty) => Some(quote! { => #ty }),
            syn::ReturnType::Default => None,
        };

        quote! {
            #expect_method(#method_name) #(#args),* #output;
        }
    });

    quote! {
        use ::dapr::simulacrum::*;

        create_mock_struct! {
            struct #mock_name : {
                #(#expect_methods)*
            }
        }

        unsafe impl Send for #mock_name {}
    }
}

fn impl_mock_trait(item: &syn::ItemTrait) -> TokenStream {
    let syn::ItemTrait {
        ident,
        generics,
        ref items,
        ..
    } = item;
    let mock_name = format_ident!("{}Mock", ident);
    let (impl_generics, ty_generics, _where_clause) = generics.split_for_impl();

    let items = items.iter().map(|item| {
        if let syn::TraitItem::Method(ref method) = item {
            let method_name = method.sig.ident.to_string();
            let output = output_type(&method.sig.output);
            let method_sig = syn::Signature {
                output,
                ..method.sig.clone()
            };
            let args = method.sig.inputs.iter().flat_map(|input| {
                if let syn::FnArg::Typed(ty) = input {
                    Some(ty)
                } else {
                    None
                }
            });
            let output = match output_type(&method.sig.output) {
                syn::ReturnType::Type(_, ty) => Some(quote! { -> #ty }),
                syn::ReturnType::Default => None,
            };

            quote! {
                async #method_sig {
                    was_called!(self, #method_name, ( #(#args),* ) #output)
                }
            }
        } else {
            quote! { #item }
        }
    });

    quote! {
        #[::dapr::async_trait]
        impl #impl_generics #ident #ty_generics for #mock_name {
            #(#items)*
        }
    }
}
