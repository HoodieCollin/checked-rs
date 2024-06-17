use std::collections::{HashMap, HashSet};

use convert_case::{Case, Casing};
use proc_macro_error::{abort, emit_error};
use quote::format_ident;
use syn::parse_quote;

use crate::params::{NumberArg, NumberKind};

use super::{attr_params::AttrParams, NumberValue};

#[derive(Debug)]
pub struct ExactVariant {
    pub ident: syn::Ident,
    pub value: NumberValue,
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
    pub start: Option<NumberValue>,
    pub end: Option<NumberValue>,
    pub half_open: bool,
}

pub struct Variants {
    pub vis: syn::Visibility,
    pub name: syn::Ident,
    pub mod_name: syn::Ident,
    pub value_name: syn::Ident,
    pub exacts: HashSet<ExactVariant>,
    pub ranges: Vec<RangeVariant>,
    pub catchall: Option<syn::Ident>,
}

impl Variants {
    pub fn from_item(params: &AttrParams, item: &mut syn::Item) -> Self {
        let data;

        if let syn::Item::Enum(d) = item {
            data = d;
        } else {
            abort! {
                item,
                "Can only be applied `Specific` for enums"
            }
        }

        if params.as_soft_or_hard.is_some() {
            abort! {
                item,
                "The `as Soft` and `as Hard` parameters are not allowed on enums"
            }
        }

        let vis = data.vis.clone();
        let name = data.ident.clone();
        let mod_name = format_ident!("clamped_{}", name.to_string().to_case(Case::Snake));
        let value_name = format_ident!("{}Value", name);

        data.vis = parse_quote!(pub);

        let ty = &params.integer;

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

                        struct NumberArgList(pub Vec<NumberArg>);

                        impl syn::parse::Parse for NumberArgList {
                            fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                                Ok(Self(
                                    syn::punctuated::Punctuated::<NumberArg, syn::Token![,]>::parse_terminated(input)?
                                    .into_iter()
                                    .collect(),
                                ))
                            }
                        }

                        if let Ok(list) = attr.parse_args::<NumberArgList>() {
                            for val in list.0 {
                                let n = val.into_value(params.kind());

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
                                    (#value_name<#ty>)
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
                                kind: NumberKind,
                            ) -> Option<NumberValue> {
                                let val = match &val {
                                    Some(v) => v.as_ref(),
                                    None => return None,
                                };

                                let val: NumberArg = parse_quote!(#val);
                                Some(val.into_value(kind))
                            }

                            let start = parse_val(&val.start, params.kind());
                            let end = parse_val(&val.end, params.kind());

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

                            let wrapper_name = format_ident!("{}Value", &variant.ident);

                            variant.fields = syn::Fields::Unnamed(parse_quote! {
                                (#wrapper_name)
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
                            (#value_name<#ty>)
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
            HashSet::with_capacity((upper_limit.clone() - lower_limit + 1).into_usize())
        } else {
            HashSet::new()
        };

        let this = Self {
            vis,
            name,
            mod_name,
            value_name,
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
                                    for n in s.range(e) {
                                        covered.insert(n);
                                    }
                                } else {
                                    for n in s.range(e + 1) {
                                        covered.insert(n);
                                    }
                                }
                            }
                            (Some(s), None) => {
                                if h {
                                    let upper_limit = upper_limit;
                                    for n in s.range(upper_limit) {
                                        covered.insert(n);
                                    }
                                } else {
                                    let upper_limit = upper_limit;
                                    for n in s.range(upper_limit + 1) {
                                        covered.insert(n);
                                    }
                                }
                            }
                            (None, Some(e)) => {
                                if h {
                                    let lower_limit = lower_limit;
                                    for n in lower_limit.range(e) {
                                        covered.insert(n);
                                    }
                                } else {
                                    let lower_limit = lower_limit;
                                    for n in lower_limit.range(e + 1) {
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
            for n in lower_limit.range(upper_limit + 1) {
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
