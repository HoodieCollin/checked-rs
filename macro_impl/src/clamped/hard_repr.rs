use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    clamped::common_impl::{
        define_guard, impl_binary_op, impl_conversions, impl_deref, impl_other_compare,
        impl_other_eq, impl_self_cmp, impl_self_eq,
    },
    params::{attr_params::AttrParams, struct_item::StructItem, BehaviorArg},
};

pub fn define_mod(attr: AttrParams, mut item: syn::Item) -> TokenStream {
    let struct_item = StructItem::from_item(&attr, &mut item);
    let vis = &struct_item.vis;
    let name = &struct_item.name;
    let mod_name = &struct_item.mod_name;

    let guard_name = format_ident!("{}Guard", &name);
    let def_guard = define_guard(name, &guard_name, &attr);

    let implementations = TokenStream::from_iter(vec![
        impl_hard_repr(name, &guard_name, &attr),
        impl_deref(name, &attr),
        impl_conversions(name, &attr),
        impl_self_eq(name),
        impl_self_cmp(name),
        impl_other_eq(name, &attr),
        impl_other_compare(name, &attr),
        impl_binary_op(
            name,
            &attr,
            format_ident!("Add"),
            format_ident!("add"),
            attr.behavior_type(),
            None,
            None,
        ),
        impl_binary_op(
            name,
            &attr,
            format_ident!("Sub"),
            format_ident!("sub"),
            attr.behavior_type(),
            None,
            None,
        ),
        impl_binary_op(
            name,
            &attr,
            format_ident!("Mul"),
            format_ident!("mul"),
            attr.behavior_type(),
            None,
            None,
        ),
        impl_binary_op(
            name,
            &attr,
            format_ident!("Div"),
            format_ident!("div"),
            attr.behavior_type(),
            None,
            None,
        ),
        impl_binary_op(
            name,
            &attr,
            format_ident!("Rem"),
            format_ident!("rem"),
            attr.behavior_type(),
            None,
            None,
        ),
        impl_binary_op(
            name,
            &attr,
            format_ident!("BitAnd"),
            format_ident!("bitand"),
            attr.behavior_type(),
            None,
            None,
        ),
        impl_binary_op(
            name,
            &attr,
            format_ident!("BitOr"),
            format_ident!("bitor"),
            attr.behavior_type(),
            None,
            None,
        ),
        impl_binary_op(
            name,
            &attr,
            format_ident!("BitXor"),
            format_ident!("bitxor"),
            attr.behavior_type(),
            None,
            None,
        ),
        // impl_binary_op(name, &attr, format_ident!("Shl"), format_ident!("shl")),
        // impl_binary_op(name, &attr, format_ident!("Shr"), format_ident!("shr")),
    ]);

    quote! {
        #vis mod #mod_name {
            use super::*;

            #item

            #def_guard

            #implementations
        }

        #vis use #mod_name::#name;
    }
}

fn impl_hard_repr(name: &syn::Ident, guard_name: &syn::Ident, attr: &AttrParams) -> TokenStream {
    let integer = &attr.integer;
    let behavior = &attr.behavior_val;
    let lower_limit = attr.lower_limit_token();
    let upper_limit = attr.upper_limit_token();

    let mut methods = Vec::new();

    match attr.behavior_type() {
        BehaviorArg::Panicking(..) => {
            methods.push(quote! {
                #[inline(always)]
                pub fn new(value: #integer) -> Self {
                    match Self::from_primitive(value) {
                        Ok(v) => v,
                        Err(e) => panic!("{}", e),
                    }
                }
            });
        }
        BehaviorArg::Saturating(..) => {
            methods.push(quote! {
                #[inline(always)]
                pub fn new(value: #integer) -> Self {
                    if value < #lower_limit {
                        Self(Self::MIN)
                    } else if value > #upper_limit {
                        Self(Self::MAX)
                    } else {
                        Self::from_primitive(value).unwrap()
                    }
                }
            });
        }
    }

    let default_value = attr.default_val.into_literal_as_tokens(attr.kind());

    quote! {
        impl InherentLimits<#integer> for #name {
            const MIN: #integer = #lower_limit;
            const MAX: #integer = #upper_limit;
        }

        impl InherentBehavior for #name {
            type Behavior = #behavior;
        }

        unsafe impl ClampedInteger<#integer> for #name {
            #[inline(always)]
            fn from_primitive(n: #integer) -> ::anyhow::Result<Self> {
                Ok(Self(Self::validate(n)?))
            }

            #[inline(always)]
            fn as_primitive(&self) -> &#integer {
                &self.0
            }
        }

        unsafe impl HardClamp<#integer> for #name {}

        impl Default for #name {
            #[inline(always)]
            fn default() -> Self {
                <Self as ClampedInteger<#integer>>::from_primitive(#default_value).unwrap()
            }
        }

        impl #name {
            #(#methods)*

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
                if val < #lower_limit {
                    Err(ClampError::TooSmall { val, min: #lower_limit })
                } else if val > #upper_limit {
                    Err(ClampError::TooLarge { val, max: #upper_limit })
                } else {
                    Ok(val)
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
            pub fn modify<'a>(&'a mut self) -> #guard_name<'a> {
                #guard_name::new(self)
            }
        }
    }
}
