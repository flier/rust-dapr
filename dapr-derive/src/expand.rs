use case::CaseExt;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::{Ident, ItemTrait, TraitItem};

use crate::args::Args;
use crate::parse::Item;

impl ToTokens for Item {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Item::Trait(item) => {
                item.to_tokens(tokens);

                let expanded = dapr_client(item);

                // println!("{}", quote!(#expanded));

                tokens.append_all(expanded)
            }
        }
    }
}

pub fn expand(_item: &mut Item, _args: Args) {}

fn dapr_client(item: &ItemTrait) -> TokenStream {
    let syn::ItemTrait { ident, items, .. } = item;
    let trait_name = &ident;
    let client_name = format_ident!("{}Client", trait_name);

    let impl_client = quote! {
        impl<T> #client_name<T> {
            pub fn into_inner(self) -> T {
                self.0
            }
        }
    };

    let impl_deref = quote! {
        impl<T> ::core::ops::Deref for #client_name<T> {
            type Target = T;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
    let impl_deref_mut = quote! {
        impl<T> ::core::ops::DerefMut for #client_name<T> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };

    let methods = items.iter().filter_map(|item| {
        if let TraitItem::Method(method) = item {
            Some(method)
        } else {
            None
        }
    });

    let arg_types = arg_types(trait_name, methods.clone());
    let result_types = result_types(trait_name, methods.clone());
    let on_invoke = on_invoke(trait_name, methods.clone());

    let get_topic_subscriptions = quote! {
        async fn get_topic_subscriptions(
            &self,
            _request: ::dapr::tonic::Request<()>,
        ) -> Result<::dapr::tonic::Response<::dapr::client::GetTopicSubscriptionsEnvelope>, ::dapr::tonic::Status> {
            let topics = self.0.topic_subscriptions().await.map_err(|err| {
                ::dapr::tonic::Status::new(::dapr::tonic::Code::Internal, err.to_string())
            })?;

            Ok(::dapr::tonic::Response::new(::dapr::client::GetTopicSubscriptionsEnvelope { topics }))
        }
    };
    let on_topic_event = quote! {
        async fn on_topic_event(
            &self,
            request: ::dapr::tonic::Request<::dapr::client::CloudEventEnvelope>,
        ) -> Result<::dapr::tonic::Response<()>, ::dapr::tonic::Status> {
            self.0
                .on_topic_event(request.into_inner())
                .await
                .map(::dapr::tonic::Response::new)
                .map_err(|err| {
                    ::dapr::tonic::Status::new(::dapr::tonic::Code::Internal, err.to_string())
                })
        }
    };
    let get_bindings_subscriptions = quote! {
        async fn get_bindings_subscriptions(
            &self,
            _request: ::dapr::tonic::Request<()>,
        ) -> Result<::dapr::tonic::Response<::dapr::client::GetBindingsSubscriptionsEnvelope>, ::dapr::tonic::Status> {
            let bindings = self.0.bindings_subscriptions().await.map_err(|err| {
                ::dapr::tonic::Status::new(::dapr::tonic::Code::Internal, err.to_string())
            })?;

            Ok(::dapr::tonic::Response::new(::dapr::client::GetBindingsSubscriptionsEnvelope { bindings }))
        }
    };
    let on_binding_event = quote! {
        async fn on_binding_event(
            &self,
            request: ::dapr::tonic::Request<::dapr::client::BindingEventEnvelope>,
        ) -> Result<::dapr::tonic::Response<::dapr::client::BindingResponseEnvelope>, ::dapr::tonic::Status> {
            self.0
                .on_binding_event(request.into_inner())
                .await
                .map(::dapr::tonic::Response::new)
                .map_err(|err| {
                    ::dapr::tonic::Status::new(::dapr::tonic::Code::Internal, err.to_string())
                })
        }
    };

    quote! {
        pub struct #client_name<T>(T);

        #impl_client
        #impl_deref
        #impl_deref_mut

        #arg_types
        #result_types

        #[::dapr::tonic::async_trait]
        impl<T: #trait_name> ::dapr::client::server::DaprClient for #client_name<T>
        where
            T: Send + Sync + 'static,
        {
            #on_invoke
            #get_topic_subscriptions
            #on_topic_event
            #get_bindings_subscriptions
            #on_binding_event
        }
    }
}

fn arg_types<'a>(
    trait_name: &Ident,
    methods: impl Iterator<Item = &'a syn::TraitItemMethod>,
) -> TokenStream {
    let types = methods.map(|method| {
        let name = arg_type_name(trait_name, method);
        let fields = method.sig.inputs.iter().filter_map(|arg| {
            if let syn::FnArg::Typed(syn::PatType { pat, ty, .. }) = arg {
                Some(quote! { pub #pat : #ty })
            } else {
                None
            }
        });

        quote! {
            #[derive(Default, ::serde::Deserialize)]
            struct #name { #(#fields),* }
        }
    });

    quote! {
        #(#types)*
    }
}

fn result_types<'a>(
    trait_name: &Ident,
    methods: impl Iterator<Item = &'a syn::TraitItemMethod>,
) -> TokenStream {
    let types = methods.map(|method| {
        let name = result_type_name(trait_name, method);
        let output = match method.sig.output {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, ref ty) => Some(ty),
        };

        quote! {
            #[derive(Default, ::serde::Serialize)]
            struct #name((#output));
        }
    });

    quote! {
        #(#types)*
    }
}

fn on_invoke<'a>(
    trait_name: &Ident,
    methods: impl Iterator<Item = &'a syn::TraitItemMethod>,
) -> TokenStream {
    let methods = methods.map(|method| invoke_method(trait_name, method));

    quote! {
        async fn on_invoke(
            &self,
            request: ::dapr::tonic::Request<::dapr::client::InvokeEnvelope>,
        ) -> Result<::dapr::tonic::Response<::dapr::prost_types::Any>, ::dapr::tonic::Status> {
            let service = &self.0;

            match request.get_ref().method.as_str() {
                #(#methods,)*
                _ => Err(::dapr::tonic::Status::unimplemented("Not yet implemented"))
            }
        }
    }
}

fn invoke_method(trait_name: &Ident, method: &syn::TraitItemMethod) -> TokenStream {
    let method_name = &method.sig.ident;
    let args = method.sig.inputs.iter().filter_map(|arg| {
        if let syn::FnArg::Typed(arg) = arg {
            Some(arg)
        } else {
            None
        }
    });

    let unpack_args = {
        let name = arg_type_name(trait_name, method);
        let args = args.clone().map(|syn::PatType { pat, .. }| pat);

        quote! {
            let #name { #(#args),* } = if let Some(ref data) = request.get_ref().data {
                ::dapr::json::unpack(data).map_err(|err|
                    ::dapr::tonic::Status::new(::dapr::tonic::Code::Internal, err.to_string())
                )?
            } else {
                Default::default()
            };
        }
    };

    let call_method = {
        let names = args.clone().map(|syn::PatType { pat, .. }| pat);

        quote! {
            let res = service.#method_name(#(#names),*);
        }
    };

    let pack_res = {
        let name = result_type_name(trait_name, method);

        quote! {
            let res = #name(res);

            Ok(::dapr::tonic::Response::new(::dapr::json(&res).unwrap_or_default()))
        }
    };

    let method_pattern = method_name.to_string();

    quote! {
        #method_pattern => {
            #unpack_args
            #call_method
            #pack_res
        }
    }
}

fn arg_type_name(trait_name: &Ident, method: &syn::TraitItemMethod) -> Ident {
    format_ident!(
        "{}{}Args",
        trait_name,
        method.sig.ident.to_string().to_camel()
    )
}

fn result_type_name(trait_name: &Ident, method: &syn::TraitItemMethod) -> Ident {
    format_ident!(
        "{}{}Res",
        trait_name,
        method.sig.ident.to_string().to_camel()
    )
}
