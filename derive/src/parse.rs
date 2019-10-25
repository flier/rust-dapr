use syn::parse::{Parse, ParseStream, Result};
use syn::{Attribute, ItemTrait, Token};

pub enum Item {
    Trait(ItemTrait),
}

impl Parse for Item {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let mut lookahead = input.lookahead1();
        if lookahead.peek(Token![unsafe]) {
            let ahead = input.fork();
            ahead.parse::<Token![unsafe]>()?;
            lookahead = ahead.lookahead1();
        }
        if lookahead.peek(Token![pub]) || lookahead.peek(Token![trait]) {
            let mut item: ItemTrait = input.parse()?;
            item.attrs = attrs;
            Ok(Item::Trait(item))
        } else {
            Err(lookahead.error())
        }
    }
}
