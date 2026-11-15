use proc_macro2::{TokenTree, TokenStream, Delimiter};
use quote::{quote, TokenStreamExt};
use syn::parse::{Parse, ParseStream};
use syn::{bracketed, parenthesized, parse_macro_input, Error, Token, parse_quote, Type, ImplItemFn, braced, ImplItem};
use syn::spanned::Spanned;

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

mod tracks {
    use super::*;
    pub(crate) struct TimedTrackDataInput {
        name: Type,
        second: Type,
        fns: Vec<ImplItemFn>,
    }

    impl Parse for TimedTrackDataInput {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let name: Type = input.parse()?;
            input.parse::<Token![:]>()?;
            let second: Type = input.parse()?;

            let content;
            braced!(content in input);

            let mut fns = Vec::new();
            while !content.is_empty() {
                match content.parse::<ImplItem>()? {
                    ImplItem::Fn(f) => fns.push(f),
                    item => {
                        return Err(Error::new(
                            item.span(),
                            "only function definitions are allowed here",
                        ))
                    }
                }
            }

            Ok(TimedTrackDataInput { name, second, fns })
        }
    }

    fn take(fns: &mut Vec<ImplItemFn>, name: &str) -> Option<ImplItemFn> {
        let pos = fns.iter().position(|f| f.sig.ident == name)?;
        Some(fns.remove(pos))
    }

    fn take_required(fns: &mut Vec<ImplItemFn>, name: &str, ctx: &Type) -> syn::Result<ImplItemFn> {
        take(fns, name).ok_or_else(|| {
            Error::new(ctx.span(), format!("missing required function `{name}`"))
        })
    }

    fn reject_rest(fns: &[ImplItemFn], expected: &str) -> syn::Result<()> {
        match fns.first() {
            Some(f) => Err(Error::new(
                f.sig.ident.span(),
                format!(
                    "unexpected function `{}`; expected only {expected}",
                    f.sig.ident
                ),
            )),
            None => Ok(()),
        }
    }

    /// Extract the `TimedData` functions: required `interpolate`, optional `extend`.
    fn take_timed(fns: &mut Vec<ImplItemFn>, ctx: &Type) -> syn::Result<TokenStream> {
        let interpolate = take_required(fns, "interpolate", ctx)?;
        let extend = take(fns, "extend");
        Ok(quote! {
            #interpolate
            #extend
        })
    }

    pub(crate) fn point_impl(input: TimedTrackDataInput) -> syn::Result<TokenStream> {
        let TimedTrackDataInput { name, second: state, mut fns } = input;

        let timed_fns = take_timed(&mut fns, &name)?;
        reject_rest(&fns, "`interpolate` and `extend`")?;

        Ok(quote! {
            impl crate::tracks::TimedData for #name {
                type Animator = crate::tracks::PointAnimator<Self>;
                type State = #state;

                #timed_fns
            }
        })
    }

    pub(crate) fn event_impl(input: TimedTrackDataInput) -> syn::Result<TokenStream> {
        let TimedTrackDataInput { name, second: point, mut fns } = input;

        // Event functions: required `get_points`, optional `clip`.
        let get_points = take_required(&mut fns, "get_points", &name)?;
        let clip = take(&mut fns, "clip");

        // TimedData functions: same as `point!`.
        let timed_fns = take_timed(&mut fns, &name)?;

        reject_rest(
            &fns,
            "`get_points`, `clip`, `interpolate` and `extend`",
        )?;

        Ok(quote! {
            impl crate::tracks::Event for #name {
                type Point = #point;

                #get_points
                #clip
            }

            impl crate::tracks::TimedData for #name {
                type Animator = crate::tracks::EventAnimator<Self>;
                type State = <<Self as crate::tracks::Event>::Point as crate::tracks::TimedData>::State;

                #timed_fns
            }
        })
    }

}

/// ```ignore
/// point! {
///     MyPoint: MyState {
///         fn interpolate(...) -> ... { ... }
///         fn extend(...) -> ... { ... } // optional
///     }
/// }
/// ```
#[proc_macro]
pub fn point(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as tracks::TimedTrackDataInput);
    tracks::point_impl(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// ```ignore
/// event! {
///     MyEvent: MyPoint {
///         fn get_points(...) -> ... { ... }
///         fn clip(...) -> ... { ... }        // optional
///         fn interpolate(...) -> ... { ... }
///         fn extend(...) -> ... { ... }      // optional
///     }
/// }
/// ```
#[proc_macro]
pub fn event(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as tracks::TimedTrackDataInput);
    tracks::event_impl(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}