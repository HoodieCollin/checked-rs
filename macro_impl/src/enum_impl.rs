use std::collections::HashSet;

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::parse_quote;

use crate::{
    common_impl::{
        define_guard, impl_binary_op, impl_conversions, impl_deref, impl_other_compare,
        impl_other_eq, impl_self_cmp, impl_self_eq,
    },
    hard_impl,
    item::enum_item::{ClampedEnumItem, ClampedEnumVariant, ClampedEnumVariantField},
    params::{BehaviorArg, NumberKind, NumberValueRange, NumberValueRangeSet, Params},
};

pub fn define_mod(
    params: &Params,
    parsed_variants: &syn::punctuated::Punctuated<ClampedEnumVariant, syn::Token![,]>,
) -> syn::Result<TokenStream> {
    let integer = &params.integer;
    let behavior_val = &params.behavior_val;

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
    let def_value_item = define_value_item(&derive_attr, &value_ident, params.integer);

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
            behavior_val,
            None,
            None,
        ),
        impl_binary_op(
            ident,
            params,
            format_ident!("Sub"),
            format_ident!("sub"),
            behavior_val,
            None,
            None,
        ),
        impl_binary_op(
            ident,
            params,
            format_ident!("Mul"),
            format_ident!("mul"),
            behavior_val,
            None,
            None,
        ),
        impl_binary_op(
            ident,
            params,
            format_ident!("Div"),
            format_ident!("div"),
            behavior_val,
            None,
            None,
        ),
        impl_binary_op(
            ident,
            params,
            format_ident!("Rem"),
            format_ident!("rem"),
            behavior_val,
            None,
            None,
        ),
        impl_binary_op(
            ident,
            params,
            format_ident!("BitAnd"),
            format_ident!("bitand"),
            behavior_val,
            None,
            None,
        ),
        impl_binary_op(
            ident,
            params,
            format_ident!("BitOr"),
            format_ident!("bitor"),
            behavior_val,
            None,
            None,
        ),
        impl_binary_op(
            ident,
            params,
            format_ident!("BitXor"),
            format_ident!("bitxor"),
            behavior_val,
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
                    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
                    pub struct #other_ident;

                    impl ExactValues<#integer> for #other_ident {
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

                let (lower_limit, upper_limit) = variant.field.limits(
                    kind,
                    params.lower_limit.clone(),
                    params.upper_limit.clone(),
                );

                let lower_limit_val = lower_limit.into_value(kind);
                let upper_limit_val = upper_limit.into_value(kind);

                let mut literal_args = Vec::with_capacity(values.len());
                let mut literal_values = Vec::with_capacity(values.len());
                let mut range_set = NumberValueRangeSet::new_with_step_fns();

                for range in values {
                    literal_args.push(range.clone());

                    let range =
                        range.to_value_range(kind, Some(lower_limit_val), Some(upper_limit_val))?;

                    range_set.insert(range.to_std_inclusive_range(None, None)?);

                    literal_values.push(range);
                }

                range_items.push(hard_impl::define_mod(
                    &Params {
                        integer: kind,
                        derived_traits: params.derived_traits.clone(),
                        vis: parse_quote!(pub),
                        ident: other_ident.clone(),
                        as_soft_or_hard: Some(parse_quote!(as Hard)),
                        default_val: default_val.cloned(),
                        behavior_val: behavior_val.clone(),
                        lower_limit: Some(lower_limit),
                        upper_limit: Some(upper_limit),
                        full_coverage: range_set
                            .gaps(
                                &NumberValueRange::from_values(
                                    Some(lower_limit_val),
                                    Some(upper_limit_val),
                                    kind,
                                )?
                                .to_std_inclusive_range(None, None)?,
                            )
                            .next()
                            .is_none(),
                    },
                    &literal_values,
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

                from_range_cases.push(quote! {
                    #(#literal_args)|* => #ident::#variant_ident(#value_ident::new_valid(val)),
                });

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
                let mut variant_lower_limit = None;
                let mut variant_upper_limit = None;

                if let Some(range) = value_range {
                    variant_lower_limit = range.0.start.clone().map(|arg| arg.into_value(kind));
                    variant_upper_limit = range.0.end.clone().map(|arg| arg.into_value(kind));
                }

                let mut exacts = HashSet::with_capacity(nested_variants.len());
                let mut ranges = NumberValueRangeSet::new_with_step_fns();

                nested_enum_items.push(define_mod(
                    &Params {
                        integer: *integer,
                        derived_traits: params.derived_traits.clone(),
                        vis: parse_quote!(pub),
                        ident: other_ident.clone(),
                        as_soft_or_hard: None,
                        default_val: default_val.cloned(),
                        behavior_val: behavior_val.clone(),
                        lower_limit: variant_lower_limit.map(|val| val.into_number_arg()),
                        upper_limit: variant_upper_limit.map(|val| val.into_number_arg()),
                        full_coverage: ClampedEnumItem::check_coverage(
                            Some(&mut exacts),
                            Some(&mut ranges),
                            variant_lower_limit,
                            variant_upper_limit,
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

                if !ranges.is_empty() {
                    let literal_ranges = ranges
                        .iter()
                        .map(|range| {
                            let start = range.start();
                            let end = range.end();

                            quote! {
                                #start..=#end
                            }
                        })
                        .collect::<Vec<_>>();

                    from_range_cases.push(quote! {
                        #(#literal_ranges)|* => #ident::#variant_ident(#value_ident::new_valid(val)),
                    });
                }

                as_primitive_cases.push(quote! {
                    #ident::#variant_ident(val) => val.as_primitive(),
                });
            }
        }
    }

    let lower_limit = &params.lower_limit;
    let upper_limit = &params.upper_limit;

    let constructor_method;

    match behavior_val {
        BehaviorArg::Panicking(..) => {
            constructor_method = quote! {
                #[inline(always)]
                pub const fn new_valid(value: #integer) -> Self {
                    match const_validate(value) {
                        Ok(v) => Self(v),
                        Err(e) => panic!("{}", e),
                    }
                }
            };
        }
        BehaviorArg::Saturating(..) => {
            constructor_method = quote! {
                #[inline(always)]
                pub const fn new_valid(value: #integer) -> Self {
                    if value < #lower_limit {
                        Self(Self::MIN_INT)
                    } else if value > #upper_limit {
                        Self(Self::MAX_INT)
                    } else {
                        Self(value)
                    }
                }
            };
        }
    }

    let default_impl = if let Some(val) = &params.default_val {
        quote! {
            impl Default for #ident {
                #[inline(always)]
                fn default() -> Self {
                    <Self as ClampedInteger<#integer>>::from_primitive(#val).unwrap()
                }
            }
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
            const fn const_from_primitive(n: #integer) -> #ident {
                match n {
                    #(#from_exact_cases)*
                    #(#from_range_cases)*
                    // #const_from_catch_all_case
                }
            }

            impl InherentLimits<#integer> for #ident {
                const MIN_INT: #integer = #lower_limit;
                const MAX_INT: #integer = #upper_limit;
                const MIN: #ident = const_from_primitive(#lower_limit);
                const MAX: #ident = const_from_primitive(#upper_limit);
            }

            impl InherentBehavior for #ident {
                type Behavior = #behavior_val;
            }

            unsafe impl ClampedInteger<#integer> for #ident {
                #[inline(always)]
                fn from_primitive(n: #integer) -> ::anyhow::Result<Self> {
                    Ok(match n {
                        #(#from_exact_cases)*
                        #(#from_range_cases)*
                        // #from_catchall_case
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

            #default_impl

            impl #ident {
                #constructor_method

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
    derive_attr: &TokenStream,
    value_item_ident: &syn::Ident,
    integer: NumberKind,
) -> TokenStream {
    quote! {
        #derive_attr
        pub struct #value_item_ident<T: ExactValues<#integer>>(pub(self) #integer, std::marker::PhantomData<T>);

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

        impl<T: ExactValues<#integer>> ClampedInteger<#integer> for #value_item_ident<T> {
            #[inline(always)]
            fn from_primitive(val: #integer) -> anyhow::Result<Self> {
                if T::contains(val) {
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
    }
}
