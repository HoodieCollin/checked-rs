use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    params::UIntKind,
    variants::{ExactVariant, RangeVariant, Variants},
};

pub use crate::params::ClampParams;

/// Generate the implementation for a clamped enum. This macro generates the following:
/// - An inner type that wraps the enum's value
/// - An implementation of `EnumRepr` for the enum
/// - An implementation of `Deref` for the enum
/// - Implementations of various conversions for the enum
/// - Implementations of equality and comparison for the enum
/// - Implementations of various binary operations for the enum
pub fn clamped(attr: ClampParams, mut item: syn::Item) -> TokenStream {
    let variants = Variants::from_item(&attr, &mut item);
    let vis = &variants.vis;
    let name = &variants.name;
    let mod_name = &variants.mod_name;
    let inner_name = &variants.inner_name;

    let guard_name = format_ident!("{}Guard", &name);
    let decl_guard = declare_guard(name, &guard_name, &attr);

    let implementations = TokenStream::from_iter(vec![
        impl_enum_repr(name, inner_name, &guard_name, &attr, &variants),
        impl_deref(name, &attr),
        impl_conversions(name, &attr),
        impl_self_eq(name),
        impl_self_cmp(name),
        impl_other_eq(name, &attr),
        impl_other_compare(name, &attr),
        impl_binary_op(name, &attr, format_ident!("Add"), format_ident!("add")),
        impl_binary_op(name, &attr, format_ident!("Sub"), format_ident!("sub")),
        impl_binary_op(name, &attr, format_ident!("Mul"), format_ident!("mul")),
        impl_binary_op(name, &attr, format_ident!("Div"), format_ident!("div")),
        impl_binary_op(name, &attr, format_ident!("Rem"), format_ident!("rem")),
        impl_binary_op(
            name,
            &attr,
            format_ident!("BitAnd"),
            format_ident!("bitand"),
        ),
        impl_binary_op(name, &attr, format_ident!("BitOr"), format_ident!("bitor")),
        impl_binary_op(
            name,
            &attr,
            format_ident!("BitXor"),
            format_ident!("bitxor"),
        ),
        impl_binary_op(name, &attr, format_ident!("Shl"), format_ident!("shl")),
        impl_binary_op(name, &attr, format_ident!("Shr"), format_ident!("shr")),
    ]);

    quote! {
        mod #mod_name {
            use super::*;

            #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
            pub struct #inner_name<T>(pub(self) T);

            impl<T> std::fmt::Debug for #inner_name<T>
            where
                T: std::fmt::Debug,
            {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    self.0.fmt(f)
                }
            }

            #item

            #decl_guard

            #implementations
        }

        #vis use #mod_name::#name;
    }
}

fn declare_guard(name: &syn::Ident, guard_name: &syn::Ident, attr: &ClampParams) -> TokenStream {
    let uinteger = &attr.uinteger;
    let lower_limit = attr.lower_limit_token();
    let upper_limit = attr.upper_limit_token();

    quote! {
        pub struct #guard_name<'a>(#uinteger, &'a mut #name);

        impl<'a> std::ops::Deref for #guard_name<'a> {
            type Target = #uinteger;

            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<'a> std::ops::DerefMut for #guard_name<'a> {
            #[inline(always)]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl<'a> AsRef<#uinteger> for #guard_name<'a> {
            #[inline(always)]
            fn as_ref(&self) -> &#uinteger {
                &self.0
            }
        }

        impl<'a> AsMut<#uinteger> for #guard_name<'a> {
            #[inline(always)]
            fn as_mut(&mut self) -> &mut #uinteger {
                &mut self.0
            }
        }

        impl<'a> Drop for #guard_name<'a> {
            fn drop(&mut self) {
                #[cfg(debug_assertions)]
                {
                    eprintln!("A `Guard` was dropped without calling `commit` or `discard` first");
                }
            }
        }

        impl<'a> #guard_name<'a> {
            #[inline(always)]
            pub(self) fn new(val: &'a mut #name) -> Self {
                Self(val.into_uint(), val)
            }

            #[inline(always)]
            pub fn is_changed(&self) -> bool {
                let a = self.0;
                let b = self.1.into_uint();

                a != b
            }

            #[inline(always)]
            pub fn check(&self) -> ::anyhow::Result<()> {
                let val = <#name as InherentBehavior>::Behavior::add(0, self.0, #lower_limit, #upper_limit);

                if val == self.0 {
                    ::anyhow::Result::Ok(())
                } else {
                    ::anyhow::Result::Err(::anyhow::anyhow!("value out of bounds"))
                }
            }

            #[inline(always)]
            pub fn commit(self) -> ::anyhow::Result<(), Self> {
                let mut this = std::mem::ManuallyDrop::new(self);

                match this.check() {
                    ::anyhow::Result::Ok(_) => {
                        *this.1 = <#name as EnumRepr<#uinteger>>::from_uint(this.0).expect("value should be within bounds");
                        ::anyhow::Result::Ok(())
                    }
                    ::anyhow::Result::Err(_) => ::anyhow::Result::Err(std::mem::ManuallyDrop::into_inner(this)),
                }
            }

            #[inline(always)]
            pub fn discard(self) {
                std::mem::forget(self);
            }
        }
    }
}

fn impl_enum_repr(
    name: &syn::Ident,
    inner_name: &syn::Ident,
    guard_name: &syn::Ident,
    attr: &ClampParams,
    variants: &Variants,
) -> TokenStream {
    let uinteger = &attr.uinteger;
    let behavior = &attr.behavior_val;
    let lower_limit = attr.lower_limit_token();
    let upper_limit = attr.upper_limit_token();

    let mut factory_methods = Vec::with_capacity(variants.exacts.len());
    let mut is_exact_case_method = Vec::with_capacity(variants.exacts.len());
    let mut is_range_case_method = Vec::with_capacity(variants.ranges.len());
    let mut from_exact_cases = Vec::with_capacity(variants.exacts.len());
    let mut from_range_cases = Vec::with_capacity(variants.ranges.len());
    let mut as_uint_cases = Vec::with_capacity(variants.exacts.len());

    let mut is_catchall_case_method = None;
    let from_catchall_case;

    // Generate exact match cases
    for ExactVariant { ident, value } in &variants.exacts {
        let value = syn::parse_str::<TokenStream>(&value.to_string()).unwrap();

        let method_name = format_ident!("new_{}", ident.to_string().to_case(Case::Snake));

        factory_methods.push(quote! {
            #[inline(always)]
            pub fn #method_name() -> Self {
                Self::from_uint(#value).expect("value should be within bounds")
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
            #value => Self::#ident(#inner_name(n)),
        });

        as_uint_cases.push(quote! {
            Self::#ident(#inner_name(n)) => n,
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

        let method_name = format_ident!("is_{}", ident.to_string().to_case(Case::Snake));

        is_range_case_method.push(quote! {
            #[inline(always)]
            pub fn #method_name(&self) -> bool {
                matches!(self, Self::#ident(_))
            }
        });

        from_range_cases.push(quote! {
            #(#range_tokens)* => Self::#ident(#inner_name(n)),
        });

        as_uint_cases.push(quote! {
            Self::#ident(#inner_name(n)) => n,
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
            _ => Self::#other(#inner_name(n)),
        };

        as_uint_cases.push(quote! {
            Self::#other(#inner_name(n)) => n,
        });
    } else {
        from_catchall_case = quote! {
            _ => ::anyhow::bail!("invalid value: {}", n)
        };
    }

    let default_value = attr.default_val.into_literal_as_tokens();
    let methods = TokenStream::from_iter(
        factory_methods
            .into_iter()
            .chain(is_exact_case_method.into_iter())
            .chain(is_range_case_method.into_iter())
            .chain(is_catchall_case_method.into_iter()),
    );

    quote! {
        impl #name {
            #methods

            #[inline(always)]
            pub fn modify<'a>(&'a mut self) -> #guard_name<'a> {
                #guard_name::new(self)
            }
        }

        impl UIntegerLimits for #name {
            const MIN: u128 = #lower_limit;
            const MAX: u128 = #upper_limit;
        }

        impl InherentBehavior for #name {
            type Behavior = #behavior;
        }

        impl EnumRepr<#uinteger> for #name {
            #[inline(always)]
            fn from_uint(n: #uinteger) -> ::anyhow::Result<Self> {
                Ok(match n {
                    #(#from_exact_cases)*
                    #(#from_range_cases)*
                    #from_catchall_case
                })
            }

            #[inline(always)]
            fn as_uint(&self) -> &#uinteger {
                match &*self {
                    #(#as_uint_cases)*
                }
            }
        }

        impl Default for #name {
            #[inline(always)]
            fn default() -> Self {
                <Self as EnumRepr<#uinteger>>::from_uint(#default_value).unwrap()
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
            #[inline(always)]
            fn as_ref(&self) -> &#uinteger {
                self.as_uint()
            }
        }
    }
}

fn impl_conversions(name: &syn::Ident, attr: &ClampParams) -> TokenStream {
    let uinteger = &attr.uinteger;
    let mut conversions = Vec::with_capacity(10);

    if attr.is_u128_or_smaller() {
        conversions.push(quote! {
            impl From<#name> for u128 {
                #[inline(always)]
                fn from(val: #name ) -> Self {
                    val.into_uint() as u128
                }
            }
        });
    }

    if matches!(attr.kind(), UIntKind::U128) {
        conversions.push(quote! {
            impl From<u128> for #name {
                #[inline(always)]
                fn from(val: u128) -> Self {
                    Self::from_uint(val).expect("value should be within bounds")
                }
            }
        });
    }

    if attr.is_u64_or_smaller() {
        conversions.push(quote! {
            impl From<#name> for u64 {
                #[inline(always)]
                fn from(val: #name) -> Self {
                    val.into_uint() as u64
                }
            }
        });
    }

    if attr.is_u64_or_larger() {
        conversions.push(quote! {
            impl From<u64> for #name {
                #[inline(always)]
                fn from(val: u64) -> Self {
                    Self::from_uint(val as #uinteger).expect("value should be within bounds")
                }
            }
        });
    }

    if attr.is_u32_or_smaller() {
        conversions.push(quote! {
            impl From<#name> for u32 {
                #[inline(always)]
                fn from(val: #name) -> Self {
                    val.into_uint() as u32
                }
            }
        });
    }

    if attr.is_u32_or_larger() {
        conversions.push(quote! {
            impl From<u32> for #name {
                #[inline(always)]
                fn from(val: u32) -> Self {
                    Self::from_uint(val as #uinteger).expect("value should be within bounds")
                }
            }
        });
    }

    if attr.is_u16_or_smaller() {
        conversions.push(quote! {
            impl From<#name> for u16 {
                #[inline(always)]
                fn from(val: #name) -> Self {
                    val.into_uint() as u16
                }
            }
        });
    }

    if attr.is_u16_or_larger() {
        conversions.push(quote! {
            impl From<u16> for #name {
                #[inline(always)]
                fn from(val: u16) -> Self {
                    Self::from_uint(val as #uinteger).expect("value should be within bounds")
                }
            }
        });
    }

    if matches!(attr.kind(), UIntKind::U8) {
        conversions.push(quote! {
            impl From<#name> for u8 {
                #[inline(always)]
                fn from(val: #name) -> Self {
                    val.into_uint() as u8
                }
            }
        });
    }

    quote! {
        #(#conversions)*

        impl From<u8> for #name {
            #[inline(always)]
            fn from(val: u8) -> Self {
                Self::from_uint(val as #uinteger).expect("value should be within bounds")
            }
        }
    }
}

fn impl_self_eq(name: &syn::Ident) -> TokenStream {
    quote! {
        impl std::cmp::PartialEq<#name> for #name
        {
            #[inline(always)]
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
            #[inline(always)]
            fn eq(&self, other: &#uinteger ) -> bool {
                self.into_uint() as u128 == *other as u128
            }
        }

        impl std::cmp::PartialEq<#name> for u8
        {
            #[inline(always)]
            fn eq(&self, other: &#name) -> bool {
                *self as u128 == other.into_uint() as u128
            }
        }

        impl std::cmp::PartialEq<#name> for u16
        {
            #[inline(always)]
            fn eq(&self, other: &#name) -> bool {
                *self as u128 == other.into_uint() as u128
            }
        }

        impl std::cmp::PartialEq<#name> for u32
        {
            #[inline(always)]
            fn eq(&self, other: &#name) -> bool {
                *self as u128 == other.into_uint() as u128
            }
        }

        impl std::cmp::PartialEq<#name> for u64
        {
            #[inline(always)]
            fn eq(&self, other: &#name) -> bool {
                *self as u128 == other.into_uint() as u128
            }
        }

        impl std::cmp::PartialEq<#name> for u128
        {
            #[inline(always)]
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
            #[inline(always)]
            fn partial_cmp(&self, other: &#uinteger ) -> Option<std::cmp::Ordering> {
                (self.into_uint() as u128).partial_cmp(&(*other as u128))
            }
        }

        impl std::cmp::PartialOrd<#name> for u8
        {
            #[inline(always)]
            fn partial_cmp(&self, other: &#name) -> Option<std::cmp::Ordering> {
                (*self as u128).partial_cmp(&(other.into_uint() as u128))
            }
        }

        impl std::cmp::PartialOrd<#name> for u16
        {
            #[inline(always)]
            fn partial_cmp(&self, other: &#name ) -> Option<std::cmp::Ordering> {
                (*self as u128).partial_cmp(&(other.into_uint() as u128))
            }
        }

        impl std::cmp::PartialOrd<#name> for u32
        {
            #[inline(always)]
            fn partial_cmp(&self, other: &#name ) -> Option<std::cmp::Ordering> {
                (*self as u128).partial_cmp(&(other.into_uint() as u128))
            }
        }

        impl std::cmp::PartialOrd<#name> for u64
        {
            #[inline(always)]
            fn partial_cmp(&self, other: &#name ) -> Option<std::cmp::Ordering> {
                (*self as u128).partial_cmp(&(other.into_uint() as u128))
            }
        }

        impl std::cmp::PartialOrd< #name> for u128
        {
            #[inline(always)]
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
) -> TokenStream {
    let uinteger = &attr.uinteger;
    let lower = attr.lower_limit_token();
    let upper = attr.upper_limit_token();

    let assign_trait_name = format_ident!("{}Assign", trait_name);
    let assign_method_name = format_ident!("{}_assign", method_name);

    quote! {
        impl std::ops:: #trait_name for #name {
            type Output = #name;

            #[inline(always)]
            fn #method_name (self, rhs: #name) -> #name {
                Self::from_uint(<Self as InherentBehavior>::Behavior::#method_name(self.into_uint(), rhs.into_uint(), #lower, #upper)).expect("arithmetic operations should be infallible")
            }
        }

        impl std::ops:: #trait_name<#uinteger> for #name {
            type Output = #name;

            #[inline(always)]
            fn #method_name (self, rhs:  #uinteger) -> #name {
                Self::from_uint(<Self as InherentBehavior>::Behavior::#method_name(self.into_uint(), rhs, #lower, #upper)).expect("arithmetic operations should be infallible")
            }
        }

        impl std::ops:: #assign_trait_name for #name {
            #[inline(always)]
            fn #assign_method_name (&mut self, rhs: #name) {
                *self = Self::from_uint(
                    <Self as InherentBehavior>::Behavior::#method_name(self.into_uint(), rhs.into_uint(), #lower, #upper)
                ).expect("assignable operations should be infallible");
            }
        }

        impl std::ops:: #assign_trait_name<#uinteger> for #name {
            #[inline(always)]
            fn #assign_method_name (&mut self, rhs: #uinteger) {
                *self = Self::from_uint(
                    <Self as InherentBehavior>::Behavior::#method_name(self.into_uint(), rhs, #lower, #upper)
                ).expect("assignable operations should be infallible");
            }
        }
    }
}
