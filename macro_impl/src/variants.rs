use std::collections::{HashMap, HashSet};

use convert_case::{Case, Casing};
use proc_macro_error::{abort, emit_error};
use quote::format_ident;
use syn::parse_quote;

use crate::params::{ClampParams, UIntegerArg};

#[derive(Debug)]
pub struct ExactVariant {
    pub ident: syn::Ident,
    pub value: u128,
}

impl PartialEq for ExactVariant {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for ExactVariant {}

impl std::hash::Hash for ExactVariant {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

#[derive(Debug)]
pub struct RangeVariant {
    pub ident: syn::Ident,
    pub start: Option<u128>,
    pub end: Option<u128>,
    pub half_open: bool,
}

pub struct Variants {
    pub vis: syn::Visibility,
    pub name: syn::Ident,
    pub mod_name: syn::Ident,
    pub inner_name: syn::Ident,
    pub exacts: HashSet<ExactVariant>,
    pub ranges: Vec<RangeVariant>,
    pub catchall: Option<syn::Ident>,
}

impl Variants {
    pub fn from_item(params: &ClampParams, item: &mut syn::Item) -> Self {
        let data;

        if let syn::Item::Enum(d) = item {
            data = d;
        } else {
            abort! {
                item,
                "Can only derive `Specific` for enums"
            }
        }

        let vis = data.vis.clone();
        let name = data.ident.clone();
        let mod_name = format_ident!("clamped_{}", name.to_string().to_case(Case::Snake));
        let inner_name = format_ident!("{}UInt", name);

        data.vis = parse_quote!(pub);

        let ty = &params.uinteger;

        let mut exacts = HashMap::new();
        let mut ranges = Vec::new();
        let mut catchall = None;

        for variant in &mut data.variants {
            match &variant.fields {
                syn::Fields::Unit => {}
                _ => {
                    abort! {
                        variant,
                        "Each variant must be a unit field"
                    }
                }
            }

            let mut to_remove = vec![];

            for (i, attr) in variant.attrs.iter_mut().enumerate() {
                let p;

                if let Some(path) = attr.path().get_ident() {
                    p = path;
                } else {
                    continue;
                }

                match p.to_string().as_str() {
                    "eq" => {
                        to_remove.push(i);

                        struct UIntegerList(pub Vec<UIntegerArg>);

                        impl syn::parse::Parse for UIntegerList {
                            fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                                Ok(Self(
                                    syn::punctuated::Punctuated::<UIntegerArg, syn::Token![,]>::parse_terminated(input)?
                                    .into_iter()
                                    .collect(),
                                ))
                            }
                        }

                        if let Ok(list) = attr.parse_args::<UIntegerList>() {
                            for val in list.0 {
                                let n;

                                match val.base10_parse::<u128>() {
                                    Ok(num) => n = num,
                                    Err(e) => {
                                        emit_error! {
                                            val,
                                            "The `#[eq]` attribute must be one or more positive integer literals";
                                            note = e;
                                        }

                                        continue;
                                    }
                                }

                                if let Some(prev) = exacts.insert(n, variant.ident.clone()) {
                                    emit_error! {
                                        attr,
                                        "The value `{}` is already used by variant `{}`",
                                        n,
                                        prev;
                                        hint = prev.span() => "Conflicting variant here";
                                    }
                                }

                                params.abort_if_out_of_bounds(attr, n);

                                variant.fields = syn::Fields::Unnamed(parse_quote! {
                                    (#inner_name<#ty>)
                                });
                            }
                        } else {
                            emit_error! {
                                attr,
                                "The `#[eq]` attribute must be one or more integer literals"
                            }
                        }
                    }
                    "range" => {
                        to_remove.push(i);

                        if let Ok(val) = attr.parse_args::<syn::ExprRange>() {
                            let half_open = match val.limits {
                                syn::RangeLimits::HalfOpen(_) => true,
                                syn::RangeLimits::Closed(_) => false,
                            };

                            fn parse_val(
                                val: &Option<impl AsRef<syn::Expr>>,
                            ) -> Result<Option<u128>, ()> {
                                let val = match &val {
                                    Some(v) => v.as_ref(),
                                    None => {
                                        return Ok(None);
                                    }
                                };

                                let val: UIntegerArg = parse_quote!(#val);

                                if let Ok(num) = val.base10_parse::<u128>() {
                                    Ok(Some(num))
                                } else {
                                    Err(())
                                }
                            }

                            let start = if let Ok(n) = parse_val(&val.start) {
                                n
                            } else {
                                emit_error! {
                                    val,
                                    "The range start must be positive integer literals"
                                }

                                continue;
                            };

                            let end = if let Ok(n) = parse_val(&val.end) {
                                n
                            } else {
                                emit_error! {
                                    val,
                                    "The range end must be positive integer literals"
                                }

                                continue;
                            };

                            if start.is_none() && end.is_none() {
                                emit_error! {
                                    val,
                                    "The range must have at least one bound. use `#[other]` for catchall"
                                }

                                continue;
                            }

                            if end.is_none() && half_open {
                                emit_error! {
                                    val,
                                    "The range must be closed if it has only one bound"
                                }

                                continue;
                            }

                            if let Some(start) = start {
                                params.abort_if_out_of_bounds(attr, start);
                            }

                            if let Some(end) = end {
                                params.abort_if_out_of_bounds(attr, end);
                            }

                            ranges.push((start, end, half_open, variant.ident.clone()));

                            variant.fields = syn::Fields::Unnamed(parse_quote! {
                                (#inner_name<#ty>)
                            });
                        } else {
                            emit_error! {
                                attr,
                                "The `#[range]` attribute must be a range literal"
                            }
                        }
                    }
                    "other" => {
                        to_remove.push(i);

                        if catchall.is_some() {
                            abort! {
                                attr,
                                "Only one `#[other]` attribute is allowed per enum"
                            }
                        }

                        catchall = Some(variant.ident.clone());

                        variant.fields = syn::Fields::Unnamed(parse_quote! {
                            (#inner_name<#ty>)
                        });
                    }
                    _ => {}
                }
            }

            for i in to_remove.into_iter().rev() {
                variant.attrs.remove(i);
            }
        }

        // check that all possible values between `params.lower_limit_value()` and `params.upper_limit_value()` are covered
        let has_catchall = catchall.is_some();
        let lower_limit = params.lower_limit_value();
        let upper_limit = params.upper_limit_value();
        let mut covered = if !has_catchall {
            HashSet::with_capacity((upper_limit - lower_limit + 1) as usize)
        } else {
            HashSet::new()
        };

        let this = Self {
            vis,
            name,
            mod_name,
            inner_name,
            exacts: exacts
                .into_iter()
                .map(|(n, v)| {
                    if !has_catchall {
                        covered.insert(n);
                    }

                    ExactVariant { ident: v, value: n }
                })
                .collect(),
            ranges: ranges
                .into_iter()
                .map(|(s, e, h, v)| {
                    if !has_catchall {
                        match (s, e) {
                            (Some(s), Some(e)) => {
                                if h {
                                    for n in s..e {
                                        covered.insert(n);
                                    }
                                } else {
                                    for n in s..=e {
                                        covered.insert(n);
                                    }
                                }
                            }
                            (Some(s), None) => {
                                if h {
                                    for n in s..upper_limit {
                                        covered.insert(n);
                                    }
                                } else {
                                    for n in s..=upper_limit {
                                        covered.insert(n);
                                    }
                                }
                            }
                            (None, Some(e)) => {
                                if h {
                                    for n in lower_limit..e {
                                        covered.insert(n);
                                    }
                                } else {
                                    for n in lower_limit..=e {
                                        covered.insert(n);
                                    }
                                }
                            }
                            (None, None) => unreachable!("At least one bound must be present"),
                        }
                    }

                    RangeVariant {
                        ident: v,
                        start: s,
                        end: e,
                        half_open: h,
                    }
                })
                .collect(),
            catchall,
        };

        if !has_catchall {
            for n in lower_limit..=upper_limit {
                if !covered.contains(&n) {
                    emit_error! {
                        item,
                        "The value `{}` is not covered by any variant",
                        n;
                        hint = "Add a catchall variant with `#[other]` attribute";
                    }
                }
            }
        }

        this
    }
}
