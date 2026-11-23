use proc_macro2::{TokenTree, TokenStream, Delimiter};
use quote::{quote, TokenStreamExt};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, parse_quote};

struct When {
    ts: TokenStream,
}

impl Parse for When {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.step(|cursor| {
            let mut out = TokenStream::new();
            let mut rest = *cursor;
            while let Some((tt, next)) = rest.token_tree() {
                match tt {
                    TokenTree::Literal(l) => out.append_all(quote!{ crate::keymap::ctx(#l) }),
                    TokenTree::Ident(i) => {
                        if i == "always" {
                            out.append_all(quote! { crate::keymap::ALWAYS })
                        } else {
                            let s = i.to_string();
                            out.append_all(quote! { crate::keymap::ctx(#s) })
                        }
                    } ,
                    TokenTree::Group(g) => {
                        let body = g.stream();
                        let w: When = parse_quote!(#body);
                        let body = w.ts;
                        let w = match g.delimiter() {
                            Delimiter::Parenthesis => quote!( (#body) ),
                            Delimiter::Brace => quote!( {#body} ),
                            Delimiter::Bracket => quote!( [#body] ),
                            Delimiter::None => quote!(#body),
                        };
                        out.append_all(w);
                    }
                    _ => out.append(tt)
                }
                rest = next;
            };
            Ok((Self{ ts: out }, rest))
        })
    }
}

#[proc_macro]
pub fn when(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let w = parse_macro_input!(ts as When);

    w.ts.into()
}
