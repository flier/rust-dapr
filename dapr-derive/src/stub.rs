use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, Ident, TraitItem};

use crate::client::{arg_type_name, arg_types, result_type_name, result_types};

pub fn dapr_stub(item: &syn::ItemTrait) -> TokenStream {
    let syn::ItemTrait { ident, items, .. } = item;
    let trait_name = &ident;
    let stub_name = format_ident!("{}Stub", trait_name);

    let methods = items.iter().filter_map(|item| {
        if let TraitItem::Method(method) = item {
            Some(method)
        } else {
            None
        }
    });

    let arg_types = arg_types(trait_name, methods.clone());
    let result_types = result_types(trait_name, methods.clone());
    let invoke_methods = methods.map(|method| invoke_method(trait_name, method));

    quote! {
        pub struct #stub_name<'a, T> {
            runtime: &'a mut ::dapr::runtime::Runtime<T>,
            app_id: String,
        }

        impl<'a, T> #stub_name<'a, T> {
            pub fn new<S: Into<String>>(runtime: &'a mut ::dapr::runtime::Runtime<T>, app_id: S) -> Self {
                #stub_name { runtime, app_id: app_id.into() }
            }
        }

        #arg_types
        #result_types

        #[::dapr::async_trait]
        impl<'a, T> #trait_name for #stub_name<'a, T>
        where
            T: ::dapr::tonic::client::GrpcService<tonic::body::BoxBody> + Send + Sync,
            T::Future: Send,
            T::ResponseBody: ::dapr::tonic::codegen::Body + ::dapr::tonic::codegen::HttpBody + Send + 'static,
            T::Error: Into<::dapr::tonic::codegen::StdError>,
            <T::ResponseBody as ::dapr::tonic::codegen::HttpBody>::Error: Into<::dapr::tonic::codegen::StdError> + Send,
            <T::ResponseBody as ::dapr::tonic::codegen::HttpBody>::Data: Into<::dapr::bytes::Bytes> + Send,
        {
            #(#invoke_methods)*
        }
    }
}

pub fn invoke_method(trait_name: &Ident, method: &syn::TraitItemMethod) -> TokenStream {
    let method_name = method.sig.ident.to_string();
    let method_output = output_type(&method.sig.output);
    let method_sig = syn::Signature {
        output: method_output,
        ..method.sig.clone()
    };
    let args = method.sig.inputs.iter().filter_map(|arg| {
        if let syn::FnArg::Typed(arg) = arg {
            Some(arg)
        } else {
            None
        }
    });

    let pack_args = {
        let name = arg_type_name(trait_name, method);
        let args = args.clone().map(|syn::PatType { pat, .. }| pat);

        quote! {
            let args = #name { #(#args),* };
            let args = ::dapr::json::pack(&args)?;
        }
    };

    let invoke_service = quote! {
        let (res, _metadata) = self.runtime.invoke_service(&self.app_id, #method_name, args).await?;
    };

    let unpack_res = {
        let name = result_type_name(trait_name, method);

        quote! {
            let #name(res) = res.as_ref().map(::dapr::json::unpack).transpose()?.unwrap_or_default();
            Ok(res)
        }
    };

    quote! {
        async #method_sig {
            #pack_args
            #invoke_service
            #unpack_res
        }
    }
}

pub fn async_trait(item: syn::ItemTrait) -> TokenStream {
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
            method.sig.output = output_type(&method.sig.output);

            quote! { async #method }
        } else {
            quote! { #item }
        }
    });

    quote! {
        #[::dapr::async_trait]
        #(#attrs)* #vis trait #ident #ty_generics #colon_token #supertraits #where_clause {
            #(#items)*
        }
    }
}

fn output_type(output: &syn::ReturnType) -> syn::ReturnType {
    match output {
        syn::ReturnType::Default => {
            parse_quote! {
                -> ::core::result::Result<(), ::dapr::Error>
            }
        }
        syn::ReturnType::Type(_, ref ty) => {
            parse_quote! {
                -> ::core::result::Result<#ty, ::dapr::Error>
            }
        }
    }
}
