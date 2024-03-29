use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::{abort, proc_macro_error};
use quote::{quote, ToTokens};
use syn::Fields;

#[proc_macro_attribute]
#[proc_macro_error]
pub fn rematch(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    parse(attr.into(), item.into()).to_token_stream().into()
}

enum Parsed {
    Enum {
        item: syn::ItemEnum,
        attrs_per_variant: Vec<Vec<Attribute>>,
    },
    Struct {
        attr: Attribute,
        item: syn::ItemStruct,
    },
}

#[derive(Clone, Debug)]
enum Attribute {
    Regex(TokenStream),
}

fn generate_fields(caps_ident: &Ident, fields: &Fields) -> TokenStream {
    match fields {
        Fields::Named(named) => {
            let fields = named
                .named
                .iter()
                .enumerate()
                .map(|(idx, field)| {
                    let ident = field
                        .ident
                        .as_ref()
                        .expect("named field should have a name");
                    let ident_str = ident.to_string();
                    let ty = &field.ty;
                    quote! {
                        #ident: #caps_ident.get(1 + #idx)
                        .ok_or_else(|| anyhow::anyhow!("Getting group {} failed", 1 + #idx))?
                        .as_str()
                        .parse::<#ty>()
                        .map_err(|e| anyhow::anyhow!("Field '{}' parsing error: {}", #ident_str, e))?,
                    }
                })
                .collect::<TokenStream>();
            quote! {
                {
                    #fields
                }
            }
        }
        Fields::Unnamed(unnamed) => {
            let fields = unnamed
                .unnamed
                .iter()
                .enumerate()
                .map(|(idx, field)| {
                    let ty = &field.ty;
                    quote! {
                        #caps_ident.get(1 + #idx)
                        .ok_or_else(|| anyhow::anyhow!("Getting group {} failed", 1 + #idx))?
                        .as_str()
                        .parse::<#ty>()
                        .map_err(|e| anyhow::anyhow!("Field {} parsing error: {}", #idx, e))?,
                    }
                })
                .collect::<TokenStream>();
            quote! {
                (
                    #fields
                )
            }
        }
        Fields::Unit => {
            quote!()
        }
    }
}

fn generate_fields_matching(
    re: &TokenStream,
    re_ident: &Ident,
    self_ident: TokenStream,
    fields: &Fields,
) -> TokenStream {
    let caps_ident = Ident::new("caps", Span::call_site());
    let fields = generate_fields(&caps_ident, fields);

    quote! {
        lazy_static::lazy_static! {
            static ref #re_ident: regex::Regex = regex::Regex::new(#re).unwrap();
        }

        if let Some(#caps_ident) = #re_ident.captures(s) {
            return Ok(#self_ident #fields);
        }
    }
}

fn generate_enum_matching<'a>(
    variants: impl IntoIterator<Item = &'a syn::Variant>,
    attrs_per_variant: &'a [Vec<Attribute>],
) -> TokenStream {
    variants
        .into_iter()
        .zip(attrs_per_variant.iter())
        .flat_map(|(variant, attrs)| {
            attrs.iter().enumerate().map(|(idx, attr)| {
                let Attribute::Regex(re) = attr;
                let re_ident =
                    Ident::new(&format!("RE_{}_{}", variant.ident, idx), Span::call_site());
                let variant_ident = &variant.ident;
                let self_ident = quote! { Self::#variant_ident };
                generate_fields_matching(re, &re_ident, self_ident, &variant.fields)
            })
        })
        .collect()
}

impl ToTokens for Parsed {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let (name, matching) = match self {
            Parsed::Enum {
                item,
                attrs_per_variant,
            } => {
                item.to_tokens(tokens);
                (
                    &item.ident,
                    generate_enum_matching(&item.variants, attrs_per_variant),
                )
            }
            Parsed::Struct {
                attr: Attribute::Regex(re),
                item,
            } => {
                item.to_tokens(tokens);
                let re_ident = Ident::new("RE", Span::call_site());
                (
                    &item.ident,
                    generate_fields_matching(re, &re_ident, quote! { Self }, &item.fields),
                )
            }
        };
        quote! {
            impl std::str::FromStr for #name {
                type Err = anyhow::Error;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    #matching
                    return Err(anyhow::anyhow!("Regex matching failed for: {:?}", s));
                }
            }
        }
        .to_tokens(tokens);
    }
}

fn parse(attr: TokenStream, item: TokenStream) -> Parsed {
    match syn::parse2::<syn::Item>(item) {
        Ok(syn::Item::Enum(mut e)) => {
            let mut attrs_per_variant = Vec::new();
            for v in &mut e.variants {
                let mut attrs = Vec::new();
                v.attrs.retain(|attr| {
                    if attr.path.get_ident().map(proc_macro2::Ident::to_string)
                        == Some("rematch".to_owned())
                    {
                        attrs.push(parse_rematch_attrs(attr.tokens.clone()));
                        false
                    } else {
                        true
                    }
                });
                attrs_per_variant.push(attrs);
            }
            Parsed::Enum {
                item: e,
                attrs_per_variant,
            }
        }
        Ok(syn::Item::Struct(s)) => Parsed::Struct {
            attr: parse_rematch_attrs(attr),
            item: s,
        },
        Ok(item) => abort!(item, "item is not an enum and not a struct"),
        _ => panic!("rematch can't be used on this item"), // FIXME: better error handling
    }
}

fn parse_rematch_attrs(raw_attrs: TokenStream) -> Attribute {
    match syn::parse2::<syn::Expr>(raw_attrs) {
        Ok(syn::Expr::Paren(paren_expr)) => Attribute::Regex(paren_expr.expr.to_token_stream()),
        Ok(syn::Expr::Lit(lit_expr)) => Attribute::Regex(lit_expr.lit.to_token_stream()),
        Ok(expr) => {
            abort!(expr, "Unknown attributes expression!");
        }
        _ => panic!("invalid attributes"), // FIXME: better error handling
    }
}
