use std::collections::HashSet;

use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::parse_quote;

use crate::{
    common_impl::{
        define_guard, impl_binary_op, impl_conversions, impl_deref, impl_other_compare,
        impl_other_eq, impl_self_cmp, impl_self_eq,
    },
    hard_impl,
    item::enum_item::{ClampedEnumItem, ClampedEnumVariant, ClampedEnumVariantField},
    params::{DerivedTraits, NumberArg, NumberArgRange, NumberKind, NumberValue, Params},
    range_seq::RangeSeq,
};

pub fn define_mod(
    params: &Params,
    parsed_variants: &syn::punctuated::Punctuated<ClampedEnumVariant, syn::Token![,]>,
) -> syn::Result<TokenStream> {
    let integer = &params.integer;
    let behavior = &params.behavior;

    let vis = &params.vis;
    let ident = &params.ident;
    let mod_ident = params.mod_ident();

    let guard_ident = params.guard_ident();
    let def_guard = define_guard(ident, &guard_ident, params);

    let derive_attr = params
        .derived_traits
        .as_ref()
        .map(|x| x.to_token_stream())
        .unwrap_or(TokenStream::new());

    let value_ident = params.value_ident();

    let implementations = TokenStream::from_iter(vec![
        impl_deref(ident, params),
        impl_conversions(ident, params),
        impl_self_eq(ident),
        impl_self_cmp(ident),
        impl_other_eq(ident, params),
        impl_other_compare(ident, params),
        impl_binary_op(
            ident,
            params,
            format_ident!("Add"),
            format_ident!("add"),
            behavior,
            None,
            None,
        ),
        impl_binary_op(
            ident,
            params,
            format_ident!("Sub"),
            format_ident!("sub"),
            behavior,
            None,
            None,
        ),
        impl_binary_op(
            ident,
            params,
            format_ident!("Mul"),
            format_ident!("mul"),
            behavior,
            None,
            None,
        ),
        impl_binary_op(
            ident,
            params,
            format_ident!("Div"),
            format_ident!("div"),
            behavior,
            None,
            None,
        ),
        impl_binary_op(
            ident,
            params,
            format_ident!("Rem"),
            format_ident!("rem"),
            behavior,
            None,
            None,
        ),
        impl_binary_op(
            ident,
            params,
            format_ident!("BitAnd"),
            format_ident!("bitand"),
            behavior,
            None,
            None,
        ),
        impl_binary_op(
            ident,
            params,
            format_ident!("BitOr"),
            format_ident!("bitor"),
            behavior,
            None,
            None,
        ),
        impl_binary_op(
            ident,
            params,
            format_ident!("BitXor"),
            format_ident!("bitxor"),
            behavior,
            None,
            None,
        ),
        // impl_binary_op(ident, params, format_ident!("Shl"), format_ident!("shl")),
        // impl_binary_op(ident, params, format_ident!("Shr"), format_ident!("shr")),
    ]);

    let mut exact_items = Vec::with_capacity(parsed_variants.len());
    let mut range_items = Vec::with_capacity(parsed_variants.len());
    let mut nested_enum_items = Vec::with_capacity(parsed_variants.len());
    let mut from_nested_enum_impls = Vec::with_capacity(parsed_variants.len());

    let mut variants = Vec::with_capacity(parsed_variants.len());

    let mut factory_methods = Vec::with_capacity(parsed_variants.len());
    let mut matches_methods = Vec::with_capacity(parsed_variants.len());
    let mut from_exact_cases = Vec::with_capacity(parsed_variants.len());
    let mut from_range_cases = Vec::with_capacity(parsed_variants.len());
    let mut from_nested_cases = Vec::with_capacity(parsed_variants.len());
    let mut as_primitive_cases = Vec::with_capacity(parsed_variants.len());

    let mut has_catchall = false;

    for variant in parsed_variants.iter() {
        let variant_ident = &variant.ident;
        let variant_as_snake_case = variant_ident.to_string().to_case(Case::Snake);

        let default_val = variant.default_val.as_ref();

        match &variant.field {
            ClampedEnumVariantField::Values { values, .. } => {
                let other_ident = params.other_ident(variant_ident);
                let literal_values = values.iter().collect::<Vec<_>>();

                let default_impl = if let Some(default_val) = default_val {
                    quote! {
                        impl Default for #value_ident<#other_ident> {
                            #[inline(always)]
                            fn default() -> Self {
                                Self::new(#default_val).unwrap()
                            }
                        }
                    }
                } else {
                    TokenStream::new()
                };

                exact_items.push(quote! {
                    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
                    pub struct #other_ident;

                    unsafe impl ExactValues<#integer> for #other_ident {
                        const VALUES: &'static [#integer] = &[#(#literal_values),*];
                    }

                    #default_impl
                });

                variants.push(quote! {
                    #variant_ident(#value_ident<#other_ident>),
                });

                let factory_ident = format_ident!("new_{}", &variant_as_snake_case);

                if values.len() == 1 {
                    let val = &values[0];

                    factory_methods.push(quote! {
                        #[inline(always)]
                        pub fn #factory_ident() -> Self {
                            #ident::#variant_ident(#value_ident::from_primitive(#val).unwrap())
                        }
                    });
                } else {
                    factory_methods.push(quote! {
                        #[inline(always)]
                        pub fn #factory_ident(val: #integer) -> anyhow::Result<Self> {
                            Ok(#ident::#variant_ident(#value_ident::from_primitive(val)?))
                        }
                    });
                }

                let matches_method_ident = format_ident!("is_{}", &variant_as_snake_case);

                matches_methods.push(quote! {
                    #[inline(always)]
                    pub fn #matches_method_ident(&self) -> bool {
                        matches!(self, #ident::#variant_ident(_))
                    }
                });

                from_exact_cases.push(quote! {
                    #(#literal_values)|* => #ident::#variant_ident(#value_ident(val, std::marker::PhantomData)),
                });

                as_primitive_cases.push(quote! {
                    #ident::#variant_ident(val) => val.as_primitive(),
                });
            }
            ClampedEnumVariantField::Ranges { values, .. } => {
                let kind = *integer;
                let other_ident = params.other_ident(variant_ident);

                let variant_limits = variant.field.limits(kind, None, None)?;

                let lower_limit_val = variant_limits.first_val(kind);
                let upper_limit_val = variant_limits.last_val(kind);

                let mut literal_args = Vec::with_capacity(values.len());
                let mut range_seq = RangeSeq::with_capacity(kind, values.len());
                let mut is_catchall = false;

                if values.len() == 1 {
                    let range = &values[0];

                    if range.is_full_range() {
                        is_catchall = true;
                        has_catchall = true;
                    }

                    literal_args.push(range.clone());

                    let range = range.to_value_range(kind)?;

                    range_seq.insert(range)?;
                } else {
                    for range in values {
                        if range.is_full_range() {
                            return Err(syn::Error::new(
                                Span::call_site(),
                                "Cannot have a catch-all range in a range that contains other ranges",
                            ));
                        }

                        literal_args.push(NumberArgRange::new_inclusive(
                            range.first_val(kind).into_number_arg(),
                            range.last_val(kind).into_number_arg(),
                        ));

                        let range = range.to_value_range(kind)?;

                        range_seq.insert(range)?;
                    }
                }

                range_items.push(hard_impl::define_mod(
                    &Params {
                        integer: kind,
                        derived_traits: params.derived_traits.clone(),
                        vis: parse_quote!(pub),
                        ident: other_ident.clone(),
                        as_soft_or_hard: Some(parse_quote!(as Hard)),
                        default_val: default_val.map(|arg| arg.into_value(kind)),
                        behavior: behavior.clone(),
                        lower_limit_val,
                        upper_limit_val,
                        full_coverage: !range_seq.has_gaps(),
                    },
                    &range_seq
                        .ranges()
                        .into_iter()
                        .map(|range| range.into())
                        .collect(),
                )?);

                variants.push(quote! {
                    #variant_ident(#other_ident),
                });

                let factory_ident = format_ident!("new_{}", &variant_as_snake_case);

                factory_methods.push(quote! {
                    #[inline(always)]
                    pub fn #factory_ident(val: #integer) -> anyhow::Result<Self> {
                        Ok(#ident::#variant_ident(#other_ident::from_primitive(val)?))
                    }
                });

                let matches_method_ident = format_ident!("is_{}", &variant_as_snake_case);

                matches_methods.push(quote! {
                    #[inline(always)]
                    pub fn #matches_method_ident(&self) -> bool {
                        matches!(self, #ident::#variant_ident(_))
                    }
                });

                if is_catchall {
                    from_range_cases.push(quote! {
                        _ => #ident::#variant_ident(unsafe { #other_ident::new_unchecked(val) }),
                    });
                } else {
                    from_range_cases.push(quote! {
                        #(#literal_args)|* => #ident::#variant_ident(unsafe { #other_ident::new_unchecked(val) }),
                    });
                }

                as_primitive_cases.push(quote! {
                    #ident::#variant_ident(val) => val.as_primitive(),
                });
            }
            ClampedEnumVariantField::ClampedEnum {
                value_range,
                variants: nested_variants,
                ..
            } => {
                let kind = *integer;
                let other_ident = params.other_ident(variant_ident);

                let variant_lower_limit = value_range
                    .as_ref()
                    .map(|range| range.0.first_val(kind))
                    .unwrap_or_else(|| NumberArg::new_min_constant(kind).into_value(kind));
                let variant_upper_limit = value_range
                    .as_ref()
                    .map(|range| range.0.last_val(kind))
                    .unwrap_or_else(|| NumberArg::new_max_constant(kind).into_value(kind));

                let mut exacts = HashSet::with_capacity(nested_variants.len());
                let mut range_seq = RangeSeq::with_capacity(kind, nested_variants.len());

                nested_enum_items.push(define_mod(
                    &Params {
                        integer: *integer,
                        derived_traits: params.derived_traits.clone(),
                        vis: parse_quote!(pub),
                        ident: other_ident.clone(),
                        as_soft_or_hard: None,
                        default_val: default_val.map(|arg| arg.into_value(kind)),
                        behavior: behavior.clone(),
                        lower_limit_val: variant_lower_limit,
                        upper_limit_val: variant_upper_limit,
                        full_coverage: ClampedEnumItem::check_coverage(
                            Some(&mut exacts),
                            Some(&mut range_seq),
                            Some(variant_lower_limit),
                            Some(variant_upper_limit),
                            kind,
                            nested_variants.iter(),
                        )?,
                    },
                    nested_variants,
                )?);

                from_nested_enum_impls.push(quote! {
                    impl From<#other_ident> for #ident {
                        #[inline(always)]
                        fn from(val: #other_ident) -> Self {
                            #ident::#variant_ident(val)
                        }
                    }
                });

                variants.push(quote! {
                    #variant_ident(#other_ident),
                });

                let factory_ident = format_ident!("new_{}", &variant_as_snake_case);

                factory_methods.push(quote! {
                    #[inline(always)]
                    pub fn #factory_ident(val: #integer) -> anyhow::Result<Self> {
                        Ok(#ident::#variant_ident(#other_ident::from_primitive(val)?))
                    }
                });

                let matches_method_ident = format_ident!("is_{}", &variant_as_snake_case);

                matches_methods.push(quote! {
                    #[inline(always)]
                    pub fn #matches_method_ident(&self) -> bool {
                        matches!(self, #ident::#variant_ident(_))
                    }
                });

                if !exacts.is_empty() {
                    let literal_values = exacts.iter().collect::<Vec<_>>();

                    from_nested_cases.push(quote! {
                        #(#literal_values)|* => #ident::#variant_ident(#value_ident::new(val).unwrap()),
                    });
                }

                if !range_seq.is_empty() {
                    let literal_ranges = range_seq
                        .ranges()
                        .into_iter()
                        .map(|range| {
                            let start = range.start();
                            let end = range.end();

                            quote! {
                                #start..=#end
                            }
                        })
                        .collect::<Vec<_>>();

                    from_range_cases.push(quote! {
                        #(#literal_ranges)|* => #ident::#variant_ident(unsafe { #other_ident::new_unchecked(val) }),
                    });
                }

                as_primitive_cases.push(quote! {
                    #ident::#variant_ident(val) => val.as_primitive(),
                });
            }
        }
    }

    let lower_limit = &params.lower_limit_val;
    let upper_limit = &params.upper_limit_val;

    let default_val = params.default_val.unwrap_or(*lower_limit);

    let def_value_item = define_value_item(
        &params.derived_traits,
        &value_ident,
        params.integer,
        lower_limit,
        upper_limit,
    );

    let catchall_case = if !has_catchall {
        quote! {
            _ => anyhow::bail!("value is not allowed"),
        }
    } else {
        TokenStream::new()
    };

    Ok(quote! {
        #vis mod #mod_ident {
            use super::*;

            #def_value_item

            #(#exact_items)*

            #(#range_items)*

            #(#nested_enum_items)*

            #(#from_nested_enum_impls)*

            #def_guard

            #derive_attr
            pub enum #ident {
                #(#variants)*
            }

            #implementations

            #[inline(always)]
            const fn const_from_primitive(val: #integer) -> #ident {
                match val {
                    #(#from_exact_cases)*
                    #(#from_range_cases)*
                    _ => panic!("value is not allowed"),
                }
            }

            impl InherentLimits<#integer> for #ident {
                const MIN_INT: #integer = #lower_limit;
                const MAX_INT: #integer = #upper_limit;
                const MIN: #ident = const_from_primitive(#lower_limit);
                const MAX: #ident = const_from_primitive(#upper_limit);
            }

            impl InherentBehavior for #ident {
                type Behavior = #behavior;
            }

            unsafe impl ClampedInteger<#integer> for #ident {
                #[inline(always)]
                fn from_primitive(val: #integer) -> ::anyhow::Result<Self> {
                    Ok(match val {
                        #(#from_exact_cases)*
                        #(#from_range_cases)*
                        #catchall_case
                    })
                }

                #[inline(always)]
                fn as_primitive(&self) -> &#integer {
                    match &*self {
                        #(#as_primitive_cases)*
                    }
                }
            }

            unsafe impl ClampedEnum<#integer> for #ident {}

            impl Default for #ident {
                #[inline(always)]
                fn default() -> Self {
                    <Self as ClampedInteger<#integer>>::from_primitive(#default_val).unwrap()
                }
            }

            impl #ident {
                pub const unsafe fn new_unchecked(val: #integer) -> Self {
                    const_from_primitive(val)
                }

                #(#factory_methods)*

                #(#matches_methods)*

                #[inline(always)]
                pub fn validate(value: #integer) -> ::anyhow::Result<()> {
                    <Self as ClampedInteger<#integer>>::from_primitive(value)?;
                    Ok(())
                }

                #[inline(always)]
                pub fn modify<'a>(&'a mut self) -> #guard_ident<'a> {
                    #guard_ident::new(self)
                }
            }
        }

        #vis use #mod_ident::#ident;
    })
}

fn define_value_item(
    derived_traits: &Option<DerivedTraits>,
    value_item_ident: &syn::Ident,
    integer: NumberKind,
    lower_limit: &NumberValue,
    upper_limit: &NumberValue,
) -> TokenStream {
    let mut traits = derived_traits
        .as_ref()
        .map(|x| {
            let mut traits = Vec::with_capacity(x.traits.len());

            traits.extend(
                x.traits
                    .iter()
                    .filter(|ty| {
                        let ty = ty
                            .path
                            .segments
                            .last()
                            .unwrap()
                            .to_token_stream()
                            .to_string();

                        match ty.as_str() {
                            "Clone" | "Copy" | "PartialEq" | "Eq" | "PartialOrd" | "Ord" => false,
                            _ => true,
                        }
                    })
                    .cloned(),
            );

            traits
        })
        .unwrap_or(Vec::with_capacity(6));

    traits.extend(vec![
        parse_quote!(Clone),
        parse_quote!(Copy),
        parse_quote!(PartialEq),
        parse_quote!(Eq),
        parse_quote!(PartialOrd),
        parse_quote!(Ord),
    ]);

    quote! {
        #[derive(#(#traits),*)]
        pub struct #value_item_ident<T: ExactValues<#integer>>(pub(self) #integer, pub(self) std::marker::PhantomData<T>);

        impl<T: ExactValues<#integer>> InherentLimits<#integer> for #value_item_ident<T> {
            const MIN_INT: #integer = #lower_limit;
            const MAX_INT: #integer = #upper_limit;
            const MIN: Self = Self(#lower_limit, std::marker::PhantomData);
            const MAX: Self = Self(#upper_limit, std::marker::PhantomData);
        }

        impl<T: ExactValues<#integer>> Default for #value_item_ident<T> {
            #[inline(always)]
            fn default() -> Self {
                Self(T::VALUES[0], std::marker::PhantomData)
            }
        }

        impl<T: ExactValues<#integer>> std::fmt::Debug for #value_item_ident<T> {
            #[inline(always)]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl<T: ExactValues<#integer>> std::fmt::Display for #value_item_ident<T> {
            #[inline(always)]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl<T: ExactValues<#integer>> std::ops::Deref for #value_item_ident<T> {
            type Target = #integer;

            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<T: ExactValues<#integer>> AsRef<#integer> for #value_item_ident<T> {
            #[inline(always)]
            fn as_ref(&self) -> &#integer {
                &self.0
            }
        }

        impl<T: ExactValues<#integer>> From<#value_item_ident<T>> for #integer {
            #[inline(always)]
            fn from(val: #value_item_ident<T>) -> Self {
                val.0
            }
        }

        unsafe impl<T: ExactValues<#integer>> ClampedInteger<#integer> for #value_item_ident<T> {
            #[inline(always)]
            fn from_primitive(val: #integer) -> anyhow::Result<Self> {
                if T::contains_value(val) {
                    Ok(Self(val, std::marker::PhantomData))
                } else {
                    Err(anyhow::anyhow!("value is not allowed"))
                }
            }

            #[inline(always)]
            fn as_primitive(&self) -> &#integer {
                &self.0
            }
        }

        impl<T: ExactValues<#integer>> #value_item_ident<T> {
            #[inline(always)]
            pub const unsafe fn new_unchecked(val: #integer) -> Self {
                Self(val, std::marker::PhantomData)
            }
        }
    }
}
