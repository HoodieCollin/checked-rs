use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::{
    common_impl::{
        define_guard, impl_binary_op, impl_conversions, impl_deref, impl_other_compare,
        impl_other_eq, impl_self_cmp, impl_self_eq,
    },
    params::{NumberArg, NumberValueRange, Params},
};

pub fn define_mod(params: &Params, ranges: &Vec<NumberValueRange>) -> syn::Result<TokenStream> {
    let integer = &params.integer;

    let vis = &params.vis;
    let ident = &params.ident;
    let mod_ident = params.mod_ident();

    let guard_ident = params.guard_ident();
    let def_guard = define_guard(ident, &guard_ident, params);

    let implementations = TokenStream::from_iter(vec![
        impl_soft_repr(ident, &guard_ident, params, ranges)?,
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
            Some(NumberArg::new_min_constant(*integer)),
            Some(NumberArg::new_max_constant(*integer)),
        ),
        impl_binary_op(
            ident,
            params,
            format_ident!("Sub"),
            format_ident!("sub"),
            &params.behavior,
            Some(NumberArg::new_min_constant(*integer)),
            Some(NumberArg::new_max_constant(*integer)),
        ),
        impl_binary_op(
            ident,
            params,
            format_ident!("Mul"),
            format_ident!("mul"),
            &params.behavior,
            Some(NumberArg::new_min_constant(*integer)),
            Some(NumberArg::new_max_constant(*integer)),
        ),
        impl_binary_op(
            ident,
            params,
            format_ident!("Div"),
            format_ident!("div"),
            &params.behavior,
            Some(NumberArg::new_min_constant(*integer)),
            Some(NumberArg::new_max_constant(*integer)),
        ),
        impl_binary_op(
            ident,
            params,
            format_ident!("Rem"),
            format_ident!("rem"),
            &params.behavior,
            Some(NumberArg::new_min_constant(*integer)),
            Some(NumberArg::new_max_constant(*integer)),
        ),
        impl_binary_op(
            ident,
            params,
            format_ident!("BitAnd"),
            format_ident!("bitand"),
            &params.behavior,
            Some(NumberArg::new_min_constant(*integer)),
            Some(NumberArg::new_max_constant(*integer)),
        ),
        impl_binary_op(
            ident,
            params,
            format_ident!("BitOr"),
            format_ident!("bitor"),
            &params.behavior,
            Some(NumberArg::new_min_constant(*integer)),
            Some(NumberArg::new_max_constant(*integer)),
        ),
        impl_binary_op(
            ident,
            params,
            format_ident!("BitXor"),
            format_ident!("bitxor"),
            &params.behavior,
            Some(NumberArg::new_min_constant(*integer)),
            Some(NumberArg::new_max_constant(*integer)),
        ),
    ]);

    let derive_attr = params
        .derived_traits
        .as_ref()
        .map(|x| x.to_token_stream())
        .unwrap_or(TokenStream::new());

    Ok(quote! {
        #vis mod #mod_ident {
            use super::*;

            #derive_attr
            #[derive(Default)]
            pub struct #ident(#integer);

            #def_guard

            #implementations
        }

        #vis use #mod_ident::#ident;
    })
}

fn impl_soft_repr(
    ident: &syn::Ident,
    guard_ident: &syn::Ident,
    params: &Params,
    ranges: &Vec<NumberValueRange>,
) -> syn::Result<TokenStream> {
    let integer = params.integer;
    let behavior = &params.behavior;
    let lower_limit = params.lower_limit_token();
    let upper_limit = params.upper_limit_token();

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
            unsafe impl SoftClamp<#integer> for #ident {
                const VALID_RANGES: &'static [ValueRangeInclusive<#integer>] = &[
                    #(#valid_ranges)*
                ];
            }
        }
    };

    let default_impl = if let Some(val) = &params.default_val {
        quote! {
            impl Default for #ident {
                #[inline(always)]
                fn default() -> Self {
                    Self(#val)
                }
            }
        }
    } else {
        TokenStream::new()
    };

    Ok(quote! {
        impl InherentLimits<#integer> for #ident {
            const MIN_INT: #integer = #lower_limit;
            const MAX_INT: #integer = #upper_limit;
            const MIN: Self = Self(Self::MIN_INT);
            const MAX: Self = Self(Self::MAX_INT);
        }

        impl InherentBehavior for #ident {
            type Behavior = #behavior;
        }

        unsafe impl ClampedInteger<#integer> for #ident {
            #[inline(always)]
            fn from_primitive(n: #integer) -> ::anyhow::Result<Self> {
                Ok(Self(n))
            }

            #[inline(always)]
            fn as_primitive(&self) -> &#integer {
                &self.0
            }
        }

        #clamp_trait_impl

        #default_impl

        impl std::ops::DerefMut for #ident {
            #[inline(always)]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }


        impl AsMut<#integer> for #ident {
            #[inline(always)]
            fn as_mut(&mut self) -> &mut #integer {
                &mut self.0
            }
        }

        impl #ident {
            #[inline(always)]
            pub fn new(value: #integer) -> Self {
                Self(value)
            }

            #[inline(always)]
            pub fn rand() -> Self {
                loop {
                    if let Ok(v) = Self::validate(rand::random::<#integer>()) {
                        return Self::from_primitive(v).unwrap();
                    }
                }
            }

            #[inline(always)]
            pub fn validate(val: #integer) -> ::anyhow::Result<#integer, ClampError<#integer>> {
                let ranges = <#ident as SoftClamp<#integer>>::VALID_RANGES;

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
                    for range in ranges {
                        if range.contains(val) {
                            return Ok(val);
                        }
                    }

                    Err(ClampError::OutOfBounds)
                }
            }

            #[inline(always)]
            pub fn is_valid(&self) -> bool {
                Self::validate(self.0).is_ok()
            }

            #[inline(always)]
            pub fn set(&mut self, value: #integer) -> ::anyhow::Result<(), ClampError<#integer>> {
                self.0 = Self::validate(value)?;
                Ok(())
            }

            #[inline(always)]
            pub fn set_unchecked(&mut self, value: #integer) {
                self.0 = value;
            }

            #[inline(always)]
            pub fn get(&self) -> &#integer {
                &self.0
            }

            #[inline(always)]
            pub fn get_mut(&mut self) -> &mut #integer {
                &mut self.0
            }

            #[inline(always)]
            pub fn modify<'a>(&'a mut self) -> #guard_ident<'a> {
                #guard_ident::new(self)
            }
        }
    })
}
