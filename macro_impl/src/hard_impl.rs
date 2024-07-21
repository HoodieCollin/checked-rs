use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::parse_quote;

use crate::{
    common_impl::{
        define_guard, impl_binary_op, impl_conversions, impl_deref, impl_other_compare,
        impl_other_eq, impl_self_cmp, impl_self_eq,
    },
    params::{NumberValueRange, Params},
};

pub fn define_mod(params: &Params, ranges: &Vec<NumberValueRange>) -> syn::Result<TokenStream> {
    let integer = &params.integer;

    let vis = &params.vis;
    let ident = &params.ident;
    let mod_ident = params.mod_ident();

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
            &params.behavior,
            None,
        ),
        impl_binary_op(
            ident,
            params,
            format_ident!("Sub"),
            format_ident!("sub"),
            &params.behavior,
            None,
        ),
        impl_binary_op(
            ident,
            params,
            format_ident!("Mul"),
            format_ident!("mul"),
            &params.behavior,
            None,
        ),
        impl_binary_op(
            ident,
            params,
            format_ident!("Div"),
            format_ident!("div"),
            &params.behavior,
            None,
        ),
        impl_binary_op(
            ident,
            params,
            format_ident!("Rem"),
            format_ident!("rem"),
            &params.behavior,
            None,
        ),
        impl_binary_op(
            ident,
            params,
            format_ident!("BitAnd"),
            format_ident!("bitand"),
            &params.behavior,
            None,
        ),
        impl_binary_op(
            ident,
            params,
            format_ident!("BitOr"),
            format_ident!("bitor"),
            &params.behavior,
            None,
        ),
        impl_binary_op(
            ident,
            params,
            format_ident!("BitXor"),
            format_ident!("bitxor"),
            &params.behavior,
            None,
        ),
    ]);

    let behavior = &params.behavior;
    let lower_limit = params.lower_limit_token();
    let upper_limit = params.upper_limit_token();
    let default_val = params.default_val_token();

    let guard_ident = params.guard_ident();
    let def_guard = define_guard(ident, &guard_ident, params);

    let mut traits = params
        .derived_traits
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
                            "Clone" | "Copy" => false,
                            _ => true,
                        }
                    })
                    .cloned(),
            );

            traits
        })
        .unwrap_or(Vec::with_capacity(2));

    traits.extend(vec![parse_quote!(Clone), parse_quote!(Copy)]);

    let clamp_trait_impl = {
        let mut valid_ranges = Vec::with_capacity(ranges.len());

        for value_range in ranges {
            let first_val = value_range.first_val();
            let last_val = value_range.last_val();

            valid_ranges.push(quote! {
                ValueRangeInclusive(#first_val..=#last_val),
            });
        }

        quote! {
            unsafe impl RangeValues<#integer> for #ident {
                const VALID_RANGES: &'static [ValueRangeInclusive<#integer>] = &[
                    #(#valid_ranges)*
                ];
            }

            unsafe impl HardClamp<#integer> for #ident {}
        }
    };

    Ok(quote! {
        #vis mod #mod_ident {
            use super::*;

            #[derive(#(#traits),*)]
            pub struct #ident(#integer);

            impl #ident {
                /// Creates a new instance or `None` if it would be invalid.
                #[inline(always)]
                pub fn new(val: #integer) -> Option<Self> {
                    match #ident::validate(val) {
                        Ok(v) => Some(Self(v)),
                        Err(..) => None,
                    }
                }

                #[inline(always)]
                pub const unsafe fn new_unchecked(val: #integer) -> Self {
                    Self(val)
                }

                #[inline(always)]
                pub(self) fn op_behavior_params(&self) -> OpBehaviorParams<#integer> {
                    let ranges = <#ident as RangeValues<#integer>>::VALID_RANGES;

                    if ranges.len() == 1 {
                        let range = &ranges[0];

                        OpBehaviorParams::Simple {
                            min: range.first_val(),
                            max: range.last_val(),
                        }
                    } else {
                        let min = ranges.first().unwrap().first_val();
                        let max = ranges.last().unwrap().last_val();

                        OpBehaviorParams::RangesOnly(ranges)
                    }
                }

                #[inline(always)]
                pub fn rand() -> Self {
                    loop {
                        if let Ok(v) = Self::from_primitive(rand::random::<#integer>()) {
                            return v;
                        }
                    }
                }

                #[inline(always)]
                pub fn validate(val: #integer) -> ::anyhow::Result<#integer, ClampError<#integer>> {
                    let ranges = <#ident as RangeValues<#integer>>::VALID_RANGES;

                    if ranges.len() == 1 {
                        let range = &ranges[0];
                        let min = range.first_val();
                        let max = range.last_val();

                        if val < min {
                            Err(ClampError::TooSmall { val, min })
                        } else if val > max {
                            Err(ClampError::TooLarge { val, max })
                        } else {
                            Ok(val)
                        }
                    } else {
                        for (i, range) in ranges.iter().enumerate() {
                            if range.contains(val) {
                                return Ok(val);
                            }

                            let min = range.first_val();

                            if i == 0 && val < min {
                                return Err(ClampError::TooSmall { val, min });
                            }

                            if i == ranges.len() - 1 {
                                let max = range.last_val();
                                return Err(ClampError::TooLarge { val, max });
                            }

                            let left_range = range;
                            let right_range = &ranges[i + 1];

                            let left_max = left_range.last_val();
                            let right_min = right_range.first_val();

                            if val > left_max && val < right_min {
                                return Err(ClampError::OutOfBounds {
                                    val,
                                    left_min: left_range.first_val(),
                                    left_max,
                                    right_min,
                                    right_max: right_range.last_val(),
                                });
                            }
                        }

                        unreachable!("all error cases should be covered by loop");
                    }
                }

                #[inline(always)]
                pub fn set(&mut self, value: #integer) -> ::anyhow::Result<(), ClampError<#integer>> {
                    self.0 = Self::validate(value)?;
                    Ok(())
                }

                #[inline(always)]
                pub unsafe fn set_unchecked(&mut self, value: #integer) {
                    self.0 = value;
                }

                #[inline(always)]
                pub fn get(&self) -> &#integer {
                    &self.0
                }

                #[inline(always)]
                pub unsafe fn get_mut(&mut self) -> &mut #integer {
                    &mut self.0
                }

                #[inline(always)]
                pub fn modify<'a>(&'a mut self) -> #guard_ident<'a> {
                    #guard_ident::new(self)
                }
            }

            impl InherentLimits<#integer> for #ident {
                const MIN_INT: #integer = #lower_limit;
                const MAX_INT: #integer = #upper_limit;
                const MIN: Self = Self(Self::MIN_INT);
                const MAX: Self = Self(Self::MAX_INT);

                #[inline(always)]
                fn is_zero(&self) -> bool {
                    self.0 == 0
                }

                #[inline(always)]
                fn is_negative(&self) -> bool {
                    self.0 < 0
                }

                #[inline(always)]
                fn is_positive(&self) -> bool {
                    self.0 > 0
                }
            }

            impl InherentBehavior for #ident {
                type Behavior = #behavior;
            }

            unsafe impl ClampedInteger<#integer> for #ident {
                #[inline(always)]
                fn from_primitive(n: #integer) -> ::anyhow::Result<Self> {
                    Ok(Self(Self::validate(n)?))
                }

                #[inline(always)]
                fn as_primitive(&self) -> &#integer {
                    &self.0
                }
            }

            #clamp_trait_impl

            impl Default for #ident {
                #[inline(always)]
                fn default() -> Self {
                    Self(#default_val)
                }
            }

            #implementations

            #def_guard
        }

        #vis use #mod_ident::#ident;
    })
}
