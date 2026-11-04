use proc_macro2::{TokenTree, TokenStream, Delimiter};
use quote::{quote, TokenStreamExt};
use syn::parse::{Parse, ParseStream};
use syn::{bracketed, parenthesized, parse_macro_input, Error, Token, parse_quote};

struct RawHexValue {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

enum HexValue {
    Vec4(RawHexValue),
    Tuple(RawHexValue),
    Array(RawHexValue),
}

impl Parse for RawHexValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![#]>()?;
        let tt: TokenTree = input.parse()?;
        let s = tt.to_string();
        let l = s.len();

        if !matches!(l, 3 | 4 | 6 | 8) || !s.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(Error::new(
                tt.span(),
                "Expected 3, 4, 6, or 8 hex digits after '#'"
            ))
        }

        let p = u32::from_str_radix(&s, 16)
            .map_err(|e| Error::new(tt.span(), e))?;

        Ok(match l {
            3 => {
                let r = ((p >> 8) & 0xF) as f32 / 15.;
                let g = ((p >> 4) & 0xF) as f32 / 15.;
                let b = (p & 0xF) as f32 / 15.;
                Self { r, g, b, a: 1. }
            },
            4 => {
                let r = ((p >> 12) & 0xF) as f32 / 15.;
                let g = ((p >> 8) & 0xF) as f32 / 15.;
                let b = ((p >> 4) & 0xF) as f32 / 15.;
                let a = (p & 0xF) as f32 / 15.;
                Self { r, g, b, a }
            },
            6 => {
                let r = ((p >> 16) & 0xFF) as f32 / 255.;
                let g = ((p >> 8) & 0xFF) as f32 / 255.;
                let b = ( p & 0xFF) as f32 / 255.;
                Self { r, g, b, a: 1. }
            },
            8 => {
                let r = ((p >> 24) & 0xFF) as f32 / 255.;
                let g = ((p >> 16) & 0xFF) as f32 / 255.;
                let b = ((p >> 8) & 0xFF) as f32 / 255.;
                let a = ( p & 0xFF) as f32 / 255.;
                Self { r, g, b, a }
            },
            _ => unreachable!()
        })
    }
}

impl Parse for HexValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::token::Paren) {
            let content;
            parenthesized!(content in input);
            let val: RawHexValue = content.parse()?;
            Ok(Self::Tuple(val))
        } else if input.peek(syn::token::Bracket) {
            let content;
            bracketed!(content in input);
            let val: RawHexValue = content.parse()?;
            Ok(Self::Array(val))
        } else {
            let val: RawHexValue = input.parse()?;
            Ok(Self::Vec4(val))
        }
    }
}

#[proc_macro]
pub fn hex(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let hex = parse_macro_input!(ts as HexValue);
    match hex {
        HexValue::Vec4(RawHexValue { r, g, b, a }) => quote! { glam::Vec4::new(#r, #g, #b, #a) },
        HexValue::Tuple(RawHexValue { r, g, b, a }) => quote! { (#r, #g, #b, #a) },
        HexValue::Array(RawHexValue { r, g, b, a }) => quote! { [#r, #g, #b, #a] },
    }.into()
}

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

