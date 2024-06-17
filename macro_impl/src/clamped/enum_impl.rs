use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    clamped::common_impl::{
        define_guard, impl_binary_op, impl_conversions, impl_deref, impl_other_compare,
        impl_other_eq, impl_self_cmp, impl_self_eq,
    },
    params::{
        attr_params::AttrParams,
        enum_variants::{ExactVariant, RangeVariant, Variants},
        NumberArg,
    },
};

pub fn define_mod(attr: AttrParams, mut item: syn::Item) -> TokenStream {
    let variants = Variants::from_item(&attr, &mut item);
    let vis = &variants.vis;
    let name = &variants.name;
    let mod_name = &variants.mod_name;
    let value_name = &variants.value_name;
    let def_inner = define_inner(value_name);

    let guard_name = format_ident!("{}Guard", &name);
    let def_guard = define_guard(name, &guard_name, &attr);

    let mut range_items = Vec::with_capacity(variants.ranges.len());

    let implementations = TokenStream::from_iter(vec![
        impl_enum_repr(
            name,
            value_name,
            &guard_name,
            &attr,
            &variants,
            &mut range_items,
        ),
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

            #(#range_items)*

            #item

            #def_inner

            #def_guard

            #implementations
        }

        #vis use #mod_name::#name;
    }
}

fn define_inner(value_name: &syn::Ident) -> TokenStream {
    quote! {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
        pub struct #value_name<T>(pub(self) T);

        impl<T> std::fmt::Debug for #value_name<T>
        where
            T: std::fmt::Debug,
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    }
}

fn impl_enum_repr(
    name: &syn::Ident,
    value_name: &syn::Ident,
    guard_name: &syn::Ident,
    attr: &AttrParams,
    variants: &Variants,
    range_items: &mut Vec<TokenStream>,
) -> TokenStream {
    let integer = &attr.integer;
    let behavior = &attr.behavior_val;
    let lower_limit = attr.lower_limit_token();
    let upper_limit = attr.upper_limit_token();

    let mut factory_methods = Vec::with_capacity(variants.exacts.len());
    let mut is_exact_case_method = Vec::with_capacity(variants.exacts.len());
    let mut is_range_case_method = Vec::with_capacity(variants.ranges.len());
    let mut from_exact_cases = Vec::with_capacity(variants.exacts.len());
    let mut from_range_cases = Vec::with_capacity(variants.ranges.len());
    let mut as_primitive_cases = Vec::with_capacity(variants.exacts.len());

    let mut is_catchall_case_method = None;
    let from_catchall_case;

    // Generate exact match cases
    for ExactVariant { ident, value } in &variants.exacts {
        let value = syn::parse_str::<TokenStream>(&value.to_string()).unwrap();

        let method_name = format_ident!("new_{}", ident.to_string().to_case(Case::Snake));

        factory_methods.push(quote! {
            #[inline(always)]
            pub fn #method_name() -> Self {
                Self::from_primitive(#value).expect("value should be within bounds")
            }
        });

        let method_name = format_ident!("is_{}", ident.to_string().to_case(Case::Snake));

        is_exact_case_method.push(quote! {
            #[inline(always)]
            pub fn #method_name(&self) -> bool {
                matches!(self, Self::#ident(_))
            }
        });

        from_exact_cases.push(quote! {
            #value => Self::#ident(#value_name(n)),
        });

        as_primitive_cases.push(quote! {
            Self::#ident(#value_name(n)) => n,
        });
    }

    let mut range_tokens = Vec::with_capacity(3);

    for RangeVariant {
        ident,
        start,
        end,
        half_open,
    } in &variants.ranges
    {
        let kind = attr.kind();

        let range_item_name = format_ident!("{}Value", ident);
        let range_item_end = {
            let val = end.unwrap_or_else(|| NumberArg::new_max_constant(kind).into_value(kind));

            if !half_open {
                val - 1
            } else {
                val
            }
        };

        range_items.push(quote! {
            #[clamped(
                #integer as Hard,
                default = #start,
                behavior = #behavior,
                lower = #start,
                upper = #range_item_end,
            )]
            #[derive(Debug, Clone, Copy, Hash, serde::Serialize, serde::Deserialize)]
            pub struct #range_item_name;

            impl From<#range_item_name> for #name {
                fn from(n: #range_item_name) -> Self {
                    Self::#ident(n)
                }
            }

            impl From<#name> for Option<#range_item_name> {
                fn from(n: #name) -> Self {
                    match n {
                        #name::#ident(n) => Some(n),
                        _ => None,
                    }
                }
            }

            impl<'a: 'b, 'b> From<&'a #name> for Option<&'b #range_item_name> {
                fn from(n: &'a #name) -> Self {
                    match n {
                        #name::#ident(n) => Some(n),
                        _ => None,
                    }
                }
            }

            impl<'a: 'b, 'b> From<&'a mut #name> for Option<&'b mut #range_item_name> {
                fn from(n: &'a mut #name) -> Self {
                    match n {
                        #name::#ident(n) => Some(n),
                        _ => None,
                    }
                }
            }
        });

        range_tokens.clear();

        if let Some(start) = start {
            let start = syn::parse_str::<TokenStream>(&start.to_string()).unwrap();

            range_tokens.push(quote! {
                #start
            });
        }

        if *half_open {
            range_tokens.push(quote! {
                ..
            });
        } else {
            range_tokens.push(quote! {
                ..=
            });
        }

        if let Some(end) = end {
            let end = syn::parse_str::<TokenStream>(&end.to_string()).unwrap();

            range_tokens.push(quote! {
                #end
            });
        }

        let method_name = format_ident!("is_{}", ident.to_string().to_case(Case::Snake));

        is_range_case_method.push(quote! {
            #[inline(always)]
            pub fn #method_name(&self) -> bool {
                matches!(self, Self::#ident(_))
            }
        });

        from_range_cases.push(quote! {
            #(#range_tokens)* => Self::#ident(#range_item_name::new(n)),
        });

        as_primitive_cases.push(quote! {
            Self::#ident(n) => n.as_primitive(),
        });
    }

    if let Some(other) = &variants.catchall {
        let method_name = format_ident!("is_{}", other.to_string().to_lowercase());

        is_catchall_case_method = Some(quote! {
            #[inline(always)]
            pub fn #method_name(&self) -> bool {
                matches!(self, Self::#other(_))
            }
        });

        from_catchall_case = quote! {
            _ => Self::#other(#value_name(n)),
        };

        as_primitive_cases.push(quote! {
            Self::#other(#value_name(n)) => n,
        });
    } else {
        from_catchall_case = quote! {
            _ => ::anyhow::bail!("invalid value: {}", n)
        };
    }

    let default_value = attr.default_val.into_literal_as_tokens(attr.kind());
    let methods = TokenStream::from_iter(
        factory_methods
            .into_iter()
            .chain(is_exact_case_method.into_iter())
            .chain(is_range_case_method.into_iter())
            .chain(is_catchall_case_method.into_iter()),
    );

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
                Ok(match n {
                    #(#from_exact_cases)*
                    #(#from_range_cases)*
                    #from_catchall_case
                })
            }

            #[inline(always)]
            fn as_primitive(&self) -> &#integer {
                match &*self {
                    #(#as_primitive_cases)*
                }
            }
        }

        unsafe impl ClampedEnum<#integer> for #name {}

        impl Default for #name {
            #[inline(always)]
            fn default() -> Self {
                <Self as ClampedInteger<#integer>>::from_primitive(#default_value).unwrap()
            }
        }

        impl #name {
            #methods

            #[inline(always)]
            pub fn validate(value: #integer) -> ::anyhow::Result<()> {
                <Self as ClampedInteger<#integer>>::from_primitive(value)?;
                Ok(())
            }

            #[inline(always)]
            pub fn modify<'a>(&'a mut self) -> #guard_name<'a> {
                #guard_name::new(self)
            }
        }

    }
}
