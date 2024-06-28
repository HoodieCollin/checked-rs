use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::params::{BehaviorArg, NumberArg, NumberKind, Params};

pub fn define_guard(name: &syn::Ident, guard_name: &syn::Ident, params: &Params) -> TokenStream {
    let integer = params.integer;

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

pub fn impl_deref(name: &syn::Ident, params: &Params) -> TokenStream {
    let integer = params.integer;

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

pub fn impl_conversions(name: &syn::Ident, params: &Params) -> TokenStream {
    let integer = params.integer;
    let mut conversions = Vec::with_capacity(24);

    if params.is_u128_or_smaller() {
        conversions.push(quote! {
            impl From<#name> for u128 {
                #[inline(always)]
                fn from(val: #name ) -> Self {
                    val.into_primitive() as u128
                }
            }
        });
    }

    if matches!(params.integer, NumberKind::U128) {
        conversions.push(quote! {
            impl From<u128> for #name {
                #[inline(always)]
                fn from(val: u128) -> Self {
                    Self::from_primitive(val).expect("value should be within bounds")
                }
            }
        });
    }

    if params.is_usize_or_smaller() {
        conversions.push(quote! {
            impl From<#name> for usize {
                #[inline(always)]
                fn from(val: #name) -> Self {
                    val.into_primitive() as usize
                }
            }
        });
    }

    if params.is_usize_or_larger() {
        conversions.push(quote! {
            impl From<usize> for #name {
                #[inline(always)]
                fn from(val: usize) -> Self {
                    Self::from_primitive(val as #integer).expect("value should be within bounds")
                }
            }
        });
    }

    if params.is_u64_or_smaller() {
        conversions.push(quote! {
            impl From<#name> for u64 {
                #[inline(always)]
                fn from(val: #name) -> Self {
                    val.into_primitive() as u64
                }
            }
        });
    }

    if params.is_u64_or_larger() {
        conversions.push(quote! {
            impl From<u64> for #name {
                #[inline(always)]
                fn from(val: u64) -> Self {
                    Self::from_primitive(val as #integer).expect("value should be within bounds")
                }
            }
        });
    }

    if params.is_u32_or_smaller() {
        conversions.push(quote! {
            impl From<#name> for u32 {
                #[inline(always)]
                fn from(val: #name) -> Self {
                    val.into_primitive() as u32
                }
            }
        });
    }

    if params.is_u32_or_larger() {
        conversions.push(quote! {
            impl From<u32> for #name {
                #[inline(always)]
                fn from(val: u32) -> Self {
                    Self::from_primitive(val as #integer).expect("value should be within bounds")
                }
            }
        });
    }

    if params.is_u16_or_smaller() {
        conversions.push(quote! {
            impl From<#name> for u16 {
                #[inline(always)]
                fn from(val: #name) -> Self {
                    val.into_primitive() as u16
                }
            }
        });
    }

    if params.is_u16_or_larger() {
        conversions.push(quote! {
            impl From<u16> for #name {
                #[inline(always)]
                fn from(val: u16) -> Self {
                    Self::from_primitive(val as #integer).expect("value should be within bounds")
                }
            }
        });
    }

    if matches!(params.integer, NumberKind::U8) {
        conversions.push(quote! {
            impl From<#name> for u8 {
                #[inline(always)]
                fn from(val: #name) -> Self {
                    val.into_primitive() as u8
                }
            }
        });
    }

    if params.is_i128_or_smaller() {
        conversions.push(quote! {
            impl From<#name> for i128 {
                #[inline(always)]
                fn from(val: #name ) -> Self {
                    val.into_primitive() as i128
                }
            }
        });
    }

    if matches!(params.integer, NumberKind::U128) {
        conversions.push(quote! {
            impl From<u128> for #name {
                #[inline(always)]
                fn from(val: i128) -> Self {
                    Self::from_primitive(val).expect("value should be within bounds")
                }
            }
        });
    }

    if params.is_isize_or_smaller() {
        conversions.push(quote! {
            impl From<#name> for isize {
                #[inline(always)]
                fn from(val: #name) -> Self {
                    val.into_primitive() as isize
                }
            }
        });
    }

    if params.is_isize_or_larger() {
        conversions.push(quote! {
            impl From<usize> for #name {
                #[inline(always)]
                fn from(val: isize) -> Self {
                    Self::from_primitive(val as #integer).expect("value should be within bounds")
                }
            }
        });
    }

    if params.is_i64_or_smaller() {
        conversions.push(quote! {
            impl From<#name> for i64 {
                #[inline(always)]
                fn from(val: #name) -> Self {
                    val.into_primitive() as i64
                }
            }
        });
    }

    if params.is_i64_or_larger() {
        conversions.push(quote! {
            impl From<u64> for #name {
                #[inline(always)]
                fn from(val: i64) -> Self {
                    Self::from_primitive(val as #integer).expect("value should be within bounds")
                }
            }
        });
    }

    if params.is_i32_or_smaller() {
        conversions.push(quote! {
            impl From<#name> for i32 {
                #[inline(always)]
                fn from(val: #name) -> Self {
                    val.into_primitive() as i32
                }
            }
        });
    }

    if params.is_i32_or_larger() {
        conversions.push(quote! {
            impl From<u32> for #name {
                #[inline(always)]
                fn from(val: i32) -> Self {
                    Self::from_primitive(val as #integer).expect("value should be within bounds")
                }
            }
        });
    }

    if params.is_i16_or_smaller() {
        conversions.push(quote! {
            impl From<#name> for i16 {
                #[inline(always)]
                fn from(val: #name) -> Self {
                    val.into_primitive() as i16
                }
            }
        });
    }

    if params.is_i16_or_larger() {
        conversions.push(quote! {
            impl From<u16> for #name {
                #[inline(always)]
                fn from(val: i16) -> Self {
                    Self::from_primitive(val as #integer).expect("value should be within bounds")
                }
            }
        });
    }

    if matches!(params.integer, NumberKind::I8) {
        conversions.push(quote! {
            impl From<#name> for i8 {
                #[inline(always)]
                fn from(val: #name) -> Self {
                    val.into_primitive() as i8
                }
            }
        });
    }

    if params.is_signed() {
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

pub fn impl_other_eq(name: &syn::Ident, params: &Params) -> TokenStream {
    let integer = params.integer;

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

pub fn impl_other_compare(name: &syn::Ident, params: &Params) -> TokenStream {
    let integer = params.integer;

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
    params: &Params,
    trait_name: syn::Ident,
    method_name: syn::Ident,
    behavior: &BehaviorArg,
    explicit_bounds: Option<(NumberArg, NumberArg)>,
) -> TokenStream {
    let integer = params.integer;
    let assign_trait_name = format_ident!("{}Assign", trait_name);
    let assign_method_name = format_ident!("{}_assign", method_name);

    let op_params = if let Some((lower, upper)) = explicit_bounds {
        quote! {
            OpBehaviorParams::Simple {
                min: #lower,
                max: #upper,
            }
        }
    } else {
        quote! {
            self.op_behavior_params()
        }
    };

    quote! {
        impl std::ops::#trait_name for #name {
            type Output = #name;

            #[inline(always)]
            fn #method_name(self, rhs: #name) -> #name {
                unsafe {
                    Self::from_primitive_unchecked(#behavior::#method_name(
                        self.into_primitive(),
                        rhs.into_primitive(),
                        #op_params
                    ))
                }
            }
        }

        impl std::ops::#trait_name<#integer> for #name {
            type Output = #name;

            #[inline(always)]
            fn #method_name(self, rhs: #integer) -> #name {
                unsafe {
                    Self::from_primitive_unchecked(#behavior::#method_name(
                        self.into_primitive(),
                        rhs,
                        #op_params
                    ))
                }
            }
        }

        impl std::ops::#trait_name<#name> for #integer {
            type Output = #integer;

            #[inline(always)]
            fn #method_name(self, rhs: #name) -> #integer {
                Panicking::#method_name(self, rhs.into_primitive(), OpBehaviorParams::Simple {
                    min: #integer::MIN,
                    max: #integer::MAX,
                })
            }
        }

        impl std::ops::#trait_name<#name> for std::num::Saturating<#integer> {
            type Output = std::num::Saturating<#integer>;

            #[inline(always)]
            fn #method_name(self, rhs: #name) -> std::num::Saturating<#integer> {
                std::num::Saturating(Saturating::#method_name(self.0, rhs.into_primitive(), OpBehaviorParams::Simple {
                    min: #integer::MIN,
                    max: #integer::MAX,
                }))
            }
        }

        impl std::ops::#assign_trait_name for #name {
            #[inline(always)]
            fn #assign_method_name(&mut self, rhs: #name) {
                *self = unsafe {
                    Self::from_primitive_unchecked(#behavior::#method_name(
                        self.into_primitive(),
                        rhs.into_primitive(),
                        #op_params
                    ))
                };
            }
        }

        impl std::ops::#assign_trait_name<#integer> for #name {
            #[inline(always)]
            fn #assign_method_name(&mut self, rhs: #integer) {
                *self = unsafe {
                    Self::from_primitive_unchecked(#behavior::#method_name(
                        self.into_primitive(),
                        rhs,
                        #op_params
                    ))
                };
            }
        }

        impl std::ops::#assign_trait_name<#name> for #integer {
            #[inline(always)]
            fn #assign_method_name(&mut self, rhs: #name) {
                *self = Panicking::#method_name(
                    *self,
                    rhs.into_primitive(),
                    OpBehaviorParams::Simple {
                        min: #integer::MIN,
                        max: #integer::MAX,
                    }
                );
            }
        }

        impl std::ops::#assign_trait_name<#name> for std::num::Saturating<#integer> {
            #[inline(always)]
            fn #assign_method_name(&mut self, rhs: #name) {
                *self = std::num::Saturating(Saturating::#method_name(
                    self.0,
                    rhs.into_primitive(),
                    OpBehaviorParams::Simple {
                        min: #integer::MIN,
                        max: #integer::MAX,
                    }
                ));
            }
        }
    }
}
