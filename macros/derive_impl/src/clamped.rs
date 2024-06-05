use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::variants::{ExactVariant, RangeVariant, Variants};

pub use crate::params::ClampParams;

pub fn clamped(attr: ClampParams, mut item: syn::Item) -> TokenStream {
    let variants = Variants::from_item(&attr, &mut item);
    let name = &variants.name;
    let assignable = variants.catchall.is_some();

    let implementations = TokenStream::from_iter(vec![
        impl_enum_repr(name, &attr, &variants),
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
            assignable,
        ),
        impl_binary_op(
            name,
            &attr,
            format_ident!("Sub"),
            format_ident!("sub"),
            assignable,
        ),
        impl_binary_op(
            name,
            &attr,
            format_ident!("Mul"),
            format_ident!("mul"),
            assignable,
        ),
        impl_binary_op(
            name,
            &attr,
            format_ident!("Div"),
            format_ident!("div"),
            assignable,
        ),
        impl_binary_op(
            name,
            &attr,
            format_ident!("Rem"),
            format_ident!("rem"),
            assignable,
        ),
        impl_binary_op(
            name,
            &attr,
            format_ident!("BitAnd"),
            format_ident!("bitand"),
            assignable,
        ),
        impl_binary_op(
            name,
            &attr,
            format_ident!("BitOr"),
            format_ident!("bitor"),
            assignable,
        ),
        impl_binary_op(
            name,
            &attr,
            format_ident!("BitXor"),
            format_ident!("bitxor"),
            assignable,
        ),
        impl_binary_op(
            name,
            &attr,
            format_ident!("Shl"),
            format_ident!("shl"),
            assignable,
        ),
        impl_binary_op(
            name,
            &attr,
            format_ident!("Shr"),
            format_ident!("shr"),
            assignable,
        ),
    ]);

    quote! {
        #item

        #implementations
    }
}

fn impl_enum_repr(name: &syn::Ident, attr: &ClampParams, variants: &Variants) -> TokenStream {
    let uinteger = &attr.uinteger;
    let behavior = &attr.behavior_val;
    let lower_limit = attr.lower_limit_token();
    let upper_limit = attr.upper_limit_token();

    let mut from_exact_cases = Vec::with_capacity(variants.exacts.len());
    let mut from_range_cases = Vec::with_capacity(variants.ranges.len());
    let mut as_uint_cases = Vec::with_capacity(variants.exacts.len());

    let from_catchall_case;

    // Generate exact match cases
    for ExactVariant { ident, value } in &variants.exacts {
        let value = syn::parse_str::<TokenStream>(&value.to_string()).unwrap();

        from_exact_cases.push(quote! {
            #value => Self::#ident(n),
        });

        as_uint_cases.push(quote! {
            Self::#ident(n) => n,
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

        from_range_cases.push(quote! {
            #(#range_tokens)* => Self::#ident(n),
        });

        as_uint_cases.push(quote! {
            Self::#ident(n) => n,
        });
    }

    if let Some(other) = &variants.catchall {
        from_catchall_case = quote! {
            _ => Self::#other(n),
        };

        as_uint_cases.push(quote! {
            Self::#other(n) => n,
        });
    } else {
        from_catchall_case = quote! {
            _ => ::anyhow::bail!("invalid value: {}", n)
        };
    }

    let default_value = syn::parse_str::<TokenStream>(&attr.default_val.to_string()).unwrap();

    quote! {
        impl crate::prelude::UIntegerLimits for #name {
            const MIN: u128 = #lower_limit;
            const MAX: u128 = #upper_limit;
        }

        impl crate::prelude::InherentBehavior for #name {
            type Behavior = #behavior;
        }

        impl crate::prelude::EnumRepr<#uinteger> for #name {
            fn from_uint(n: #uinteger) -> ::anyhow::Result<Self> {
                Ok(match n {
                    #(#from_exact_cases)*
                    #(#from_range_cases)*
                    #from_catchall_case
                })
            }

            fn as_uint(&self) -> &#uinteger {
                match &*self {
                    #(#as_uint_cases)*
                }
            }
        }

        impl Default for #name {
            fn default() -> Self {
                <Self as crate::prelude::EnumRepr<#uinteger>>::from_uint(#default_value).unwrap()
            }
        }
    }
}

fn impl_deref(name: &syn::Ident, attr: &ClampParams) -> TokenStream {
    let uinteger = &attr.uinteger;

    quote! {
        impl std::ops::Deref for #name {
            type Target = #uinteger;

            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                self.as_uint()
            }
        }

        impl AsRef<#uinteger> for #name {
            fn as_ref(&self) -> &#uinteger {
                self.as_uint()
            }
        }
    }
}

fn impl_conversions(name: &syn::Ident, attr: &ClampParams) -> TokenStream {
    let mut conversions = Vec::with_capacity(5);

    if attr.is_u128_or_smaller() {
        conversions.push(quote! {
            impl From<#name> for u128 {
                fn from(val: #name ) -> Self {
                    val.into_uint() as u128
                }
            }
        });
    }

    if attr.is_u64_or_smaller() {
        conversions.push(quote! {
            impl From<#name> for u64 {
                fn from(val: #name  ) -> Self {
                    val.into_uint() as u64
                }
            }
        });
    }

    if attr.is_u32_or_smaller() {
        conversions.push(quote! {
            impl From<#name> for u32 {
                fn from(val: #name  ) -> Self {
                    val.into_uint() as u32
                }
            }
        });
    }

    if attr.is_u16_or_smaller() {
        conversions.push(quote! {
            impl From<#name> for u16 {
                fn from(val: #name  ) -> Self {
                    val.into_uint() as u16
                }
            }
        });
    }

    if attr.is_u8() {
        conversions.push(quote! {
            impl From<#name> for u8 {
                fn from(val: #name  ) -> Self {
                    val.into_uint() as u8
                }
            }
        });
    }

    quote! {
        #(#conversions)*
    }
}

fn impl_self_eq(name: &syn::Ident) -> TokenStream {
    quote! {
        impl std::cmp::PartialEq<#name> for #name
        {
            fn eq(&self, other: &#name ) -> bool {
                self.into_uint() == other.into_uint()
            }
        }

        impl std::cmp::Eq for #name
        {
        }
    }
}

fn impl_self_cmp(name: &syn::Ident) -> TokenStream {
    quote! {
        impl std::cmp::PartialOrd<#name> for #name
        {
            #[inline(always)]
            fn partial_cmp(&self, rhs: &#name ) -> Option<std::cmp::Ordering> {
                self.into_uint().partial_cmp(&rhs.into_uint())
            }
        }

        impl std::cmp::Ord for #name
        {
            #[inline(always)]
            fn cmp(&self, rhs: &#name) -> std::cmp::Ordering {
                self.into_uint().cmp(&rhs.into_uint())
            }
        }
    }
}

fn impl_other_eq(name: &syn::Ident, attr: &ClampParams) -> TokenStream {
    let uinteger = &attr.uinteger;

    quote! {
        impl std::cmp::PartialEq<#uinteger> for #name
        {
            fn eq(&self, other: &#uinteger ) -> bool {
                self.into_uint() as u128 == *other as u128
            }
        }

        impl std::cmp::PartialEq<#name> for u8
        {
            fn eq(&self, other: &#name) -> bool {
                *self as u128 == other.into_uint() as u128
            }
        }

        impl std::cmp::PartialEq<#name> for u16
        {
            fn eq(&self, other: &#name) -> bool {
                *self as u128 == other.into_uint() as u128
            }
        }

        impl std::cmp::PartialEq<#name> for u32
        {
            fn eq(&self, other: &#name) -> bool {
                *self as u128 == other.into_uint() as u128
            }
        }

        impl std::cmp::PartialEq<#name> for u64
        {
            fn eq(&self, other: &#name) -> bool {
                *self as u128 == other.into_uint() as u128
            }
        }

        impl std::cmp::PartialEq<#name> for u128
        {
            fn eq(&self, other: &#name) -> bool {
                *self as u128 == other.into_uint() as u128
            }
        }
    }
}

fn impl_other_compare(name: &syn::Ident, attr: &ClampParams) -> TokenStream {
    let uinteger = &attr.uinteger;

    quote! {
        impl std::cmp::PartialOrd<#uinteger> for #name
        {
            fn partial_cmp(&self, other: &#uinteger ) -> Option<std::cmp::Ordering> {
                (self.into_uint() as u128).partial_cmp(&(*other as u128))
            }
        }

        impl std::cmp::PartialOrd<#name> for u8
        {
            fn partial_cmp(&self, other: &#name) -> Option<std::cmp::Ordering> {
                (*self as u128).partial_cmp(&(other.into_uint() as u128))
            }
        }

        impl std::cmp::PartialOrd<#name> for u16
        {
            fn partial_cmp(&self, other: &#name ) -> Option<std::cmp::Ordering> {
                (*self as u128).partial_cmp(&(other.into_uint() as u128))
            }
        }

        impl std::cmp::PartialOrd<#name> for u32
        {
            fn partial_cmp(&self, other: &#name ) -> Option<std::cmp::Ordering> {
                (*self as u128).partial_cmp(&(other.into_uint() as u128))
            }
        }

        impl std::cmp::PartialOrd<#name> for u64
        {
            fn partial_cmp(&self, other: &#name ) -> Option<std::cmp::Ordering> {
                (*self as u128).partial_cmp(&(other.into_uint() as u128))
            }
        }

        impl std::cmp::PartialOrd< #name> for u128
        {
            fn partial_cmp(&self, other: &#name) -> Option<std::cmp::Ordering> {
                (*self as u128).partial_cmp(&(other.into_uint() as u128))
            }
        }
    }
}

fn impl_binary_op(
    name: &syn::Ident,
    attr: &ClampParams,
    trait_name: syn::Ident,
    method_name: syn::Ident,
    assignable: bool,
) -> TokenStream {
    let uinteger = &attr.uinteger;
    let lower = attr.lower_limit_token();
    let upper = attr.upper_limit_token();

    let main = quote! {
        impl std::ops:: #trait_name for #name {
            type Output = #name;

            fn #method_name (self, rhs: #name) -> #name {
                Self::from_uint(<Self as crate::prelude::InherentBehavior>::Behavior::#method_name(self.into_uint(), rhs.into_uint(), #lower, #upper)).expect("arithmetic operations should be infallible")
            }
        }

        impl std::ops:: #trait_name<#uinteger> for #name {
            type Output = #name;

            fn #method_name (self, rhs:  #uinteger) -> #name {
                Self::from_uint(<Self as crate::prelude::InherentBehavior>::Behavior::#method_name(self.into_uint(), rhs, #lower, #upper)).expect("arithmetic operations should be infallible")
            }
        }

    };

    if !assignable {
        main
    } else {
        let assign_trait_name = format_ident!("{}Assign", trait_name);
        let assign_method_name = format_ident!("{}_assign", method_name);

        quote! {
            #main

            impl std::ops:: #assign_trait_name for #name {
                fn #assign_method_name (&mut self, rhs: #name) {
                    *self = Self::from_uint(
                        <Self as crate::prelude::InherentBehavior>::Behavior::#method_name(self.into_uint(), rhs.into_uint(), #lower, #upper)
                    ).expect("assignable operations should be infallible");
                }
            }

            impl std::ops:: #assign_trait_name<#uinteger> for #name {
                fn #assign_method_name (&mut self, rhs: #uinteger) {
                    *self = Self::from_uint(
                        <Self as crate::prelude::InherentBehavior>::Behavior::#method_name(self.into_uint(), rhs, #lower, #upper)
                    ).expect("assignable operations should be infallible");
                }
            }
        }
    }
}
