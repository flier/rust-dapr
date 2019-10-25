use proc_macro2::Span;
use syn::parse::{Error, Parse, ParseStream, Result};

#[derive(Copy, Clone)]
pub struct Args {}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        match try_parse(input) {
            Ok(args) if input.is_empty() => Ok(args),
            _ => Err(error()),
        }
    }
}

fn try_parse(_input: ParseStream) -> Result<Args> {
    Ok(Args {})
}

fn error() -> Error {
    let msg = "expected #[dapr::service]";
    Error::new(Span::call_site(), msg)
}
