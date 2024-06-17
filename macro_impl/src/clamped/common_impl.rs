use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::params::{attr_params::AttrParams, BehaviorArg, NumberArg, NumberKind};

pub fn define_guard(name: &syn::Ident, guard_name: &syn::Ident, attr: &AttrParams) -> TokenStream {
    let integer = &attr.integer;

    quote! {
        pub struct #guard_name<'a>(#integer, &'a mut #name);

        impl<'a> std::ops::Deref for #guard_name<'a> {
            type Target = #integer;

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

        impl<'a> AsRef<#integer> for #guard_name<'a> {
            #[inline(always)]
            fn as_ref(&self) -> &#integer {
                &self.0
            }
        }

        impl<'a> AsMut<#integer> for #guard_name<'a> {
            #[inline(always)]
            fn as_mut(&mut self) -> &mut #integer {
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
                Self(val.into_primitive(), val)
            }

            #[inline(always)]
            pub fn is_changed(&self) -> bool {
                let a = self.0;
                let b = self.1.into_primitive();

                a != b
            }

            #[inline(always)]
            pub fn check(&self) -> ::anyhow::Result<()> {
                #name::validate(self.0)?;
                Ok(())
            }

            #[inline(always)]
            pub fn commit(self) -> ::anyhow::Result<(), Self> {
                let mut this = std::mem::ManuallyDrop::new(self);

                match this.check() {
                    ::anyhow::Result::Ok(_) => {
                        *this.1 = <#name as ClampedInteger<#integer>>::from_primitive(this.0).expect("value should be within bounds");
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

pub fn impl_deref(name: &syn::Ident, attr: &AttrParams) -> TokenStream {
    let integer = &attr.integer;

    quote! {
        impl std::ops::Deref for #name {
            type Target = #integer;

            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                self.as_primitive()
            }
        }

        impl AsRef<#integer> for #name {
            #[inline(always)]
            fn as_ref(&self) -> &#integer {
                self.as_primitive()
            }
        }
    }
}

pub fn impl_conversions(name: &syn::Ident, attr: &AttrParams) -> TokenStream {
    let integer = &attr.integer;
    let mut conversions = Vec::with_capacity(24);

    if attr.is_u128_or_smaller() {
        conversions.push(quote! {
            impl From<#name> for u128 {
                #[inline(always)]
                fn from(val: #name ) -> Self {
                    val.into_primitive() as u128
                }
            }
        });
    }

    if matches!(attr.kind(), NumberKind::U128) {
        conversions.push(quote! {
            impl From<u128> for #name {
                #[inline(always)]
                fn from(val: u128) -> Self {
                    Self::from_primitive(val).expect("value should be within bounds")
                }
            }
        });
    }

    if attr.is_usize_or_smaller() {
        conversions.push(quote! {
            impl From<#name> for usize {
                #[inline(always)]
                fn from(val: #name) -> Self {
                    val.into_primitive() as usize
                }
            }
        });
    }

    if attr.is_usize_or_larger() {
        conversions.push(quote! {
            impl From<usize> for #name {
                #[inline(always)]
                fn from(val: usize) -> Self {
                    Self::from_primitive(val as #integer).expect("value should be within bounds")
                }
            }
        });
    }

    if attr.is_u64_or_smaller() {
        conversions.push(quote! {
            impl From<#name> for u64 {
                #[inline(always)]
                fn from(val: #name) -> Self {
                    val.into_primitive() as u64
                }
            }
        });
    }

    if attr.is_u64_or_larger() {
        conversions.push(quote! {
            impl From<u64> for #name {
                #[inline(always)]
                fn from(val: u64) -> Self {
                    Self::from_primitive(val as #integer).expect("value should be within bounds")
                }
            }
        });
    }

    if attr.is_u32_or_smaller() {
        conversions.push(quote! {
            impl From<#name> for u32 {
                #[inline(always)]
                fn from(val: #name) -> Self {
                    val.into_primitive() as u32
                }
            }
        });
    }

    if attr.is_u32_or_larger() {
        conversions.push(quote! {
            impl From<u32> for #name {
                #[inline(always)]
                fn from(val: u32) -> Self {
                    Self::from_primitive(val as #integer).expect("value should be within bounds")
                }
            }
        });
    }

    if attr.is_u16_or_smaller() {
        conversions.push(quote! {
            impl From<#name> for u16 {
                #[inline(always)]
                fn from(val: #name) -> Self {
                    val.into_primitive() as u16
                }
            }
        });
    }

    if attr.is_u16_or_larger() {
        conversions.push(quote! {
            impl From<u16> for #name {
                #[inline(always)]
                fn from(val: u16) -> Self {
                    Self::from_primitive(val as #integer).expect("value should be within bounds")
                }
            }
        });
    }

    if matches!(attr.kind(), NumberKind::U8) {
        conversions.push(quote! {
            impl From<#name> for u8 {
                #[inline(always)]
                fn from(val: #name) -> Self {
                    val.into_primitive() as u8
                }
            }
        });
    }

    if attr.is_i128_or_smaller() {
        conversions.push(quote! {
            impl From<#name> for i128 {
                #[inline(always)]
                fn from(val: #name ) -> Self {
                    val.into_primitive() as i128
                }
            }
        });
    }

    if matches!(attr.kind(), NumberKind::U128) {
        conversions.push(quote! {
            impl From<u128> for #name {
                #[inline(always)]
                fn from(val: i128) -> Self {
                    Self::from_primitive(val).expect("value should be within bounds")
                }
            }
        });
    }

    if attr.is_isize_or_smaller() {
        conversions.push(quote! {
            impl From<#name> for isize {
                #[inline(always)]
                fn from(val: #name) -> Self {
                    val.into_primitive() as isize
                }
            }
        });
    }

    if attr.is_isize_or_larger() {
        conversions.push(quote! {
            impl From<usize> for #name {
                #[inline(always)]
                fn from(val: isize) -> Self {
                    Self::from_primitive(val as #integer).expect("value should be within bounds")
                }
            }
        });
    }

    if attr.is_i64_or_smaller() {
        conversions.push(quote! {
            impl From<#name> for i64 {
                #[inline(always)]
                fn from(val: #name) -> Self {
                    val.into_primitive() as i64
                }
            }
        });
    }

    if attr.is_i64_or_larger() {
        conversions.push(quote! {
            impl From<u64> for #name {
                #[inline(always)]
                fn from(val: i64) -> Self {
                    Self::from_primitive(val as #integer).expect("value should be within bounds")
                }
            }
        });
    }

    if attr.is_i32_or_smaller() {
        conversions.push(quote! {
            impl From<#name> for i32 {
                #[inline(always)]
                fn from(val: #name) -> Self {
                    val.into_primitive() as i32
                }
            }
        });
    }

    if attr.is_i32_or_larger() {
        conversions.push(quote! {
            impl From<u32> for #name {
                #[inline(always)]
                fn from(val: i32) -> Self {
                    Self::from_primitive(val as #integer).expect("value should be within bounds")
                }
            }
        });
    }

    if attr.is_i16_or_smaller() {
        conversions.push(quote! {
            impl From<#name> for i16 {
                #[inline(always)]
                fn from(val: #name) -> Self {
                    val.into_primitive() as i16
                }
            }
        });
    }

    if attr.is_i16_or_larger() {
        conversions.push(quote! {
            impl From<u16> for #name {
                #[inline(always)]
                fn from(val: i16) -> Self {
                    Self::from_primitive(val as #integer).expect("value should be within bounds")
                }
            }
        });
    }

    if matches!(attr.kind(), NumberKind::I8) {
        conversions.push(quote! {
            impl From<#name> for i8 {
                #[inline(always)]
                fn from(val: #name) -> Self {
                    val.into_primitive() as i8
                }
            }
        });
    }

    if attr.is_signed() {
        conversions.push(quote! {
            impl From<i8> for #name {
                #[inline(always)]
                fn from(val: i8) -> Self {
                    Self::from_primitive(val as #integer).expect("value should be within bounds")
                }
            }
        });
    } else {
        conversions.push(quote! {
            impl From<u8> for #name {
                #[inline(always)]
                fn from(val: u8) -> Self {
                    Self::from_primitive(val as #integer).expect("value should be within bounds")
                }
            }
        });
    }

    quote! {
        #(#conversions)*

        impl std::str::FromStr for #name {
            type Err = ::anyhow::Error;

            #[inline(always)]
            fn from_str(s: &str) -> ::anyhow::Result<Self> {
                let n = s.parse::<#integer>()?;
                Self::from_primitive(n)
            }
        }
    }
}

pub fn impl_self_eq(name: &syn::Ident) -> TokenStream {
    quote! {
        impl std::cmp::PartialEq<#name> for #name
        {
            #[inline(always)]
            fn eq(&self, other: &#name ) -> bool {
                self.into_primitive() == other.into_primitive()
            }
        }

        impl std::cmp::Eq for #name
        {
        }
    }
}

pub fn impl_self_cmp(name: &syn::Ident) -> TokenStream {
    quote! {
        impl std::cmp::PartialOrd<#name> for #name
        {
            #[inline(always)]
            fn partial_cmp(&self, rhs: &#name ) -> Option<std::cmp::Ordering> {
                self.into_primitive().partial_cmp(&rhs.into_primitive())
            }
        }

        impl std::cmp::Ord for #name
        {
            #[inline(always)]
            fn cmp(&self, rhs: &#name) -> std::cmp::Ordering {
                self.into_primitive().cmp(&rhs.into_primitive())
            }
        }
    }
}

pub fn impl_other_eq(name: &syn::Ident, attr: &AttrParams) -> TokenStream {
    let integer = &attr.integer;

    quote! {
        impl std::cmp::PartialEq<#integer> for #name
        {
            #[inline(always)]
            fn eq(&self, other: &#integer ) -> bool {
                self.into_primitive() == *other
            }
        }

        impl std::cmp::PartialEq<#name> for #integer
        {
            #[inline(always)]
            fn eq(&self, other: &#name) -> bool {
                *self == other.into_primitive()
            }
        }
    }
}

pub fn impl_other_compare(name: &syn::Ident, attr: &AttrParams) -> TokenStream {
    let integer = &attr.integer;

    quote! {
        impl std::cmp::PartialOrd<#integer> for #name
        {
            #[inline(always)]
            fn partial_cmp(&self, other: &#integer ) -> Option<std::cmp::Ordering> {
                (self.into_primitive()).partial_cmp(other)
            }
        }

        impl std::cmp::PartialOrd<#name> for #integer
        {
            #[inline(always)]
            fn partial_cmp(&self, other: &#name) -> Option<std::cmp::Ordering> {
                self.partial_cmp(other.as_primitive())
            }
        }
    }
}

pub fn impl_binary_op(
    name: &syn::Ident,
    attr: &AttrParams,
    trait_name: syn::Ident,
    method_name: syn::Ident,
    behavior: &BehaviorArg,
    lower: Option<NumberArg>,
    upper: Option<NumberArg>,
) -> TokenStream {
    let kind = attr.kind();
    let integer = &attr.integer;

    let lower = lower
        .map(|n| n.into_literal_as_tokens(kind))
        .unwrap_or(attr.lower_limit_token());

    let upper = upper
        .map(|n| n.into_literal_as_tokens(kind))
        .unwrap_or(attr.upper_limit_token());

    let assign_trait_name = format_ident!("{}Assign", trait_name);
    let assign_method_name = format_ident!("{}_assign", method_name);

    quote! {
        impl std::ops::#trait_name for #name {
            type Output = #name;

            #[inline(always)]
            fn #method_name(self, rhs: #name) -> #name {
                Self::from_primitive(#behavior::#method_name(self.into_primitive(), rhs.into_primitive(), #lower, #upper)).expect("arithmetic operations should be infallible")
            }
        }

        impl std::ops::#trait_name<#integer> for #name {
            type Output = #name;

            #[inline(always)]
            fn #method_name(self, rhs: #integer) -> #name {
                Self::from_primitive(#behavior::#method_name(self.into_primitive(), rhs, #lower, #upper)).expect("arithmetic operations should be infallible")
            }
        }

        impl std::ops::#trait_name<#name> for #integer {
            type Output = #integer;

            #[inline(always)]
            fn #method_name(self, rhs: #name) -> #integer {
                Panicking::#method_name(self, rhs.into_primitive(), #integer::MIN, #integer::MAX)
            }
        }

        impl std::ops::#trait_name<#name> for std::num::Saturating<#integer> {
            type Output = std::num::Saturating<#integer>;

            #[inline(always)]
            fn #method_name(self, rhs: #name) -> std::num::Saturating<#integer> {
                std::num::Saturating(Saturating::#method_name(self.0, rhs.into_primitive(), #integer::MIN, #integer::MAX))
            }
        }

        impl std::ops::#assign_trait_name for #name {
            #[inline(always)]
            fn #assign_method_name(&mut self, rhs: #name) {
                *self = Self::from_primitive(
                    #behavior::#method_name(self.into_primitive(), rhs.into_primitive(), #lower, #upper)
                ).expect("assignable operations should be infallible");
            }
        }

        impl std::ops::#assign_trait_name<#integer> for #name {
            #[inline(always)]
            fn #assign_method_name(&mut self, rhs: #integer) {
                *self = Self::from_primitive(
                    #behavior::#method_name(self.into_primitive(), rhs, #lower, #upper)
                ).expect("assignable operations should be infallible");
            }
        }

        impl std::ops::#assign_trait_name<#name> for #integer {
            #[inline(always)]
            fn #assign_method_name(&mut self, rhs: #name) {
                *self = Panicking::#method_name(*self, rhs.into_primitive(), #integer::MIN, #integer::MAX);
            }
        }

        impl std::ops::#assign_trait_name<#name> for std::num::Saturating<#integer> {
            #[inline(always)]
            fn #assign_method_name(&mut self, rhs: #name) {
                *self = std::num::Saturating(Saturating::#method_name(self.0, rhs.into_primitive(), #integer::MIN, #integer::MAX));
            }
        }
    }
}
