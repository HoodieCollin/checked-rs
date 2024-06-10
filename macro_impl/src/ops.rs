use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, DeriveInput};

use crate::params::GenericParams;

pub fn derive_ops(input: DeriveInput) -> TokenStream {
    TokenStream::from_iter(vec![
        impl_default(&input),
        impl_deref(&input),
        impl_conversions(&input),
        impl_self_eq(&input),
        impl_self_cmp(&input),
        impl_other_eq(&input),
        impl_other_compare(&input),
        impl_binary_op(&input, format_ident!("Add"), format_ident!("add")),
        impl_binary_op(&input, format_ident!("Sub"), format_ident!("sub")),
        impl_binary_op(&input, format_ident!("Mul"), format_ident!("mul")),
        impl_binary_op(&input, format_ident!("Div"), format_ident!("div")),
        impl_binary_op(&input, format_ident!("Rem"), format_ident!("rem")),
        impl_binary_op(&input, format_ident!("BitAnd"), format_ident!("bitand")),
        impl_binary_op(&input, format_ident!("BitOr"), format_ident!("bitor")),
        impl_binary_op(&input, format_ident!("BitXor"), format_ident!("bitxor")),
        impl_binary_op(&input, format_ident!("Shl"), format_ident!("shl")),
        impl_binary_op(&input, format_ident!("Shr"), format_ident!("shr")),
    ])
}

fn impl_default(input: &syn::DeriveInput) -> TokenStream {
    let name = &input.ident;
    let base = GenericParams::from_input(input);
    let lower = base.lower_ident();
    let (impl_generics, ty_generics, where_clause) = base.split_for_impl();

    quote! {
        impl #impl_generics Default for #name #ty_generics #where_clause {
            #[inline(always)]
            fn default() -> Self {
                unsafe { Self::new_unchecked(private::from_u128(#lower)) }
            }
        }
    }
}

fn impl_deref(input: &syn::DeriveInput) -> TokenStream {
    let name = &input.ident;
    let base = GenericParams::from_input(input);
    let ident_uinteger = base.uinteger_ident();
    let (impl_generics, ty_generics, where_clause) = base.split_for_impl();

    if input
        .attrs
        .iter()
        .any(|a| a.path().is_ident("derive_deref_mut"))
    {
        quote! {
            impl #impl_generics std::ops::Deref for #name #ty_generics #where_clause {
                type Target = #ident_uinteger;

                #[inline(always)]
                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl #impl_generics std::ops::DerefMut for #name #ty_generics #where_clause {
                #[inline(always)]
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.0
                }
            }

            impl #impl_generics AsRef<#ident_uinteger> for #name #ty_generics #where_clause {
                fn as_ref(&self) -> & #ident_uinteger {
                    &self.0
                }
            }

            impl #impl_generics AsMut<#ident_uinteger> for #name #ty_generics #where_clause {
                fn as_mut(&mut self) -> &mut #ident_uinteger {
                    &mut self.0
                }
            }
        }
    } else {
        quote! {
            impl #impl_generics std::ops::Deref for #name #ty_generics #where_clause {
                type Target = #ident_uinteger;

                #[inline(always)]
                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl #impl_generics AsRef<#ident_uinteger> for #name #ty_generics #where_clause {
                fn as_ref(&self) -> & #ident_uinteger {
                    &self.0
                }
            }
        }
    }
}

fn impl_conversions(input: &syn::DeriveInput) -> TokenStream {
    let name = &input.ident;
    let base = GenericParams::from_input(input);

    let u8_generics = base.with_uinteger_ident(parse_quote!(u8));
    let (u8_impl_generics, u8_ty_generics, u8_where_clause) = u8_generics.split_for_impl();

    let u16_generics = base.with_uinteger_ident(parse_quote!(u16));
    let (u16_impl_generics, u16_ty_generics, u16_where_clause) = u16_generics.split_for_impl();

    let u32_generics = base.with_uinteger_ident(parse_quote!(u32));
    let (u32_impl_generics, u32_ty_generics, u32_where_clause) = u32_generics.split_for_impl();

    let u64_generics = base.with_uinteger_ident(parse_quote!(u64));
    let (u64_impl_generics, u64_ty_generics, u64_where_clause) = u64_generics.split_for_impl();

    let u128_generics = base.with_uinteger_ident(parse_quote!(u128));
    let (u128_impl_generics, u128_ty_generics, u128_where_clause) = u128_generics.split_for_impl();

    quote! {
        impl #u8_impl_generics From< #name #u8_ty_generics > for u8 #u8_where_clause {
            fn from(val: #name #u8_ty_generics ) -> Self {
                val.get_unchecked()
            }
        }

        impl #u16_impl_generics From< #name #u16_ty_generics > for u16 #u16_where_clause {
            fn from(val: #name #u16_ty_generics ) -> Self {
                val.get_unchecked()
            }
        }

        impl #u32_impl_generics From< #name #u32_ty_generics > for u32 #u32_where_clause {
            fn from(val: #name #u32_ty_generics ) -> Self {
                val.get_unchecked()
            }
        }

        impl #u64_impl_generics From< #name #u64_ty_generics > for u64 #u64_where_clause {
            fn from(val: #name #u64_ty_generics ) -> Self {
                val.get_unchecked()
            }
        }

        impl #u128_impl_generics From< #name #u128_ty_generics > for u128 #u128_where_clause {
            fn from(val: #name #u128_ty_generics ) -> Self {
                val.get_unchecked()
            }
        }
    }
}

fn impl_self_eq(input: &syn::DeriveInput) -> TokenStream {
    let name = &input.ident;
    let base = GenericParams::from_input(input);

    let impl_uinteger = base.uinteger();
    let impl_behavior = base.behavior();
    let impl_extras = base.extras();

    let ty_uinteger = base.uinteger_ident();
    let ty_behavior = base.behavior_ident();
    let ty_extras = base.extra_idents();

    let (impl_generics, ty_generics, where_clause) = base.split_for_impl();

    quote! {
        impl<
            #impl_uinteger,
            #impl_behavior,
            #(#impl_extras,)*
            const L1: u128,
            const U1: u128,
            const L2: u128,
            const U2: u128,
        > std::cmp::PartialEq<#name <#ty_uinteger, #ty_behavior, #(#ty_extras,)* L2, U2>> for #name <#ty_uinteger, #ty_behavior, #(#ty_extras,)* L1, U1> #where_clause
        {
            fn eq(&self, other: & #name <#ty_uinteger, #ty_behavior, #(#ty_extras,)* L2, U2>) -> bool {
                self.get_unchecked() == other.get_unchecked()
            }
        }

        impl #impl_generics std::cmp::Eq for #name #ty_generics #where_clause
        {
        }
    }
}

fn impl_self_cmp(input: &syn::DeriveInput) -> TokenStream {
    let name = &input.ident;
    let base = GenericParams::from_input(input);

    let impl_uinteger = base.uinteger();
    let impl_behavior = base.behavior();
    let impl_extras = base.extras();

    let ty_uinteger = base.uinteger_ident();
    let ty_behavior = base.behavior_ident();
    let ty_extras = base.extra_idents();

    let (impl_generics, ty_generics, where_clause) = base.split_for_impl();

    quote! {
        impl<
            #impl_uinteger,
            #impl_behavior,
            #(#impl_extras,)*
            const L1: u128,
            const U1: u128,
            const L2: u128,
            const U2: u128,
        > std::cmp::PartialOrd<#name <#ty_uinteger, #ty_behavior, #(#ty_extras,)* L2, U2>>
        for #name <#ty_uinteger, #ty_behavior, #(#ty_extras,)* L1, U1> #where_clause
        {
            #[inline(always)]
            fn partial_cmp(&self, rhs: & #name <#ty_uinteger, #ty_behavior, #(#ty_extras,)* L2, U2>) -> Option<std::cmp::Ordering> {
                self.get_unchecked().partial_cmp(&rhs.get_unchecked())
            }
        }

        impl #impl_generics std::cmp::Ord for #name #ty_generics #where_clause
        {
            #[inline(always)]
            fn cmp(&self, rhs: & #name #ty_generics) -> std::cmp::Ordering {
                self.get_unchecked().cmp(&rhs.get_unchecked())
            }
        }
    }
}

fn impl_other_eq(input: &syn::DeriveInput) -> TokenStream {
    let name = &input.ident;
    let base = GenericParams::from_input(input);
    let ident_uinteger = base.uinteger_ident();
    let (impl_generics, ty_generics, where_clause) = base.split_for_impl();

    let u8_generics = base.with_uinteger_ident(parse_quote!(u8));
    let (u8_impl_generics, u8_ty_generics, u8_where_clause) = u8_generics.split_for_impl();

    let u16_generics = base.with_uinteger_ident(parse_quote!(u16));
    let (u16_impl_generics, u16_ty_generics, u16_where_clause) = u16_generics.split_for_impl();

    let u32_generics = base.with_uinteger_ident(parse_quote!(u32));
    let (u32_impl_generics, u32_ty_generics, u32_where_clause) = u32_generics.split_for_impl();

    let u64_generics = base.with_uinteger_ident(parse_quote!(u64));
    let (u64_impl_generics, u64_ty_generics, u64_where_clause) = u64_generics.split_for_impl();

    let u128_generics = base.with_uinteger_ident(parse_quote!(u128));
    let (u128_impl_generics, u128_ty_generics, u128_where_clause) = u128_generics.split_for_impl();

    quote! {
        impl #impl_generics std::cmp::PartialEq< #ident_uinteger > for #name #ty_generics #where_clause
        {
            fn eq(&self, other: & #ident_uinteger ) -> bool {
                self.get_unchecked() == *other
            }
        }

        impl #u8_impl_generics std::cmp::PartialEq< #name #u8_ty_generics > for u8 #u8_where_clause
        {
            fn eq(&self, other: & #name #u8_ty_generics) -> bool {
                *self == other.get_unchecked()
            }
        }

        impl #u16_impl_generics std::cmp::PartialEq< #name #u16_ty_generics > for u16 #u16_where_clause
        {
            fn eq(&self, other: & #name #u16_ty_generics) -> bool {
                *self == other.get_unchecked()
            }
        }

        impl #u32_impl_generics std::cmp::PartialEq< #name #u32_ty_generics > for u32 #u32_where_clause
        {
            fn eq(&self, other: & #name #u32_ty_generics) -> bool {
                *self == other.get_unchecked()
            }
        }

        impl #u64_impl_generics std::cmp::PartialEq< #name #u64_ty_generics > for u64 #u64_where_clause
        {
            fn eq(&self, other: & #name #u64_ty_generics) -> bool {
                *self == other.get_unchecked()
            }
        }

        impl #u128_impl_generics std::cmp::PartialEq< #name #u128_ty_generics > for u128 #u128_where_clause
        {
            fn eq(&self, other: & #name #u128_ty_generics) -> bool {
                *self == other.get_unchecked()
            }
        }
    }
}

fn impl_other_compare(input: &syn::DeriveInput) -> TokenStream {
    let name = &input.ident;
    let base = GenericParams::from_input(input);
    let ident_uinteger = base.uinteger_ident();
    let (impl_generics, ty_generics, where_clause) = base.split_for_impl();

    let u8_generics = base.with_uinteger_ident(parse_quote!(u8));
    let (u8_impl_generics, u8_ty_generics, u8_where_clause) = u8_generics.split_for_impl();

    let u16_generics = base.with_uinteger_ident(parse_quote!(u16));
    let (u16_impl_generics, u16_ty_generics, u16_where_clause) = u16_generics.split_for_impl();

    let u32_generics = base.with_uinteger_ident(parse_quote!(u32));
    let (u32_impl_generics, u32_ty_generics, u32_where_clause) = u32_generics.split_for_impl();

    let u64_generics = base.with_uinteger_ident(parse_quote!(u64));
    let (u64_impl_generics, u64_ty_generics, u64_where_clause) = u64_generics.split_for_impl();

    let u128_generics = base.with_uinteger_ident(parse_quote!(u128));
    let (u128_impl_generics, u128_ty_generics, u128_where_clause) = u128_generics.split_for_impl();

    quote! {
        impl #impl_generics std::cmp::PartialOrd< #ident_uinteger > for #name #ty_generics #where_clause
        {
            fn partial_cmp(&self, other: & #ident_uinteger ) -> Option<std::cmp::Ordering> {
                self.get_unchecked().partial_cmp(other)
            }
        }

        impl #u8_impl_generics std::cmp::PartialOrd< #name #u8_ty_generics > for u8 #u8_where_clause
        {
            fn partial_cmp(&self, other: & #name #u8_ty_generics) -> Option<std::cmp::Ordering> {
                self.partial_cmp(&other.get_unchecked())
            }
        }

        impl #u16_impl_generics std::cmp::PartialOrd< #name #u16_ty_generics > for u16 #u16_where_clause
        {
            fn partial_cmp(&self, other: & #name #u16_ty_generics ) -> Option<std::cmp::Ordering> {
                self.partial_cmp(&other.get_unchecked())
            }
        }

        impl #u32_impl_generics std::cmp::PartialOrd< #name #u32_ty_generics > for u32 #u32_where_clause
        {
            fn partial_cmp(&self, other: & #name #u32_ty_generics ) -> Option<std::cmp::Ordering> {
                self.partial_cmp(&other.get_unchecked())
            }
        }

        impl #u64_impl_generics std::cmp::PartialOrd< #name #u64_ty_generics > for u64 #u64_where_clause
        {
            fn partial_cmp(&self, other: & #name #u64_ty_generics ) -> Option<std::cmp::Ordering> {
                self.partial_cmp(&other.get_unchecked())
            }
        }

        impl #u128_impl_generics std::cmp::PartialOrd< #name #u128_ty_generics> for u128 #u128_where_clause
        {
            fn partial_cmp(&self, other: & #name #u128_ty_generics) -> Option<std::cmp::Ordering> {
                self.partial_cmp(&other.get_unchecked())
            }
        }
    }
}

fn impl_binary_op(
    input: &syn::DeriveInput,
    trait_name: syn::Ident,
    method_name: syn::Ident,
) -> TokenStream {
    let name = &input.ident;
    let base = GenericParams::from_input(input);
    let (impl_generics, ty_generics, where_clause) = base.split_for_impl();

    let ident_uinteger = base.uinteger_ident();
    let ident_behavior = base.behavior_ident();
    let ident_lower = base.lower_ident();
    let ident_upper = base.upper_ident();

    let assign_trait_name = format_ident!("{}Assign", trait_name);
    let assign_method_name = format_ident!("{}_assign", method_name);

    quote! {
        impl #impl_generics std::ops:: #trait_name for #name #ty_generics #where_clause {
            type Output = #name #ty_generics;

            fn #method_name (self, rhs: #name #ty_generics) -> #name #ty_generics {
                unsafe { Self::new_unchecked(#ident_behavior :: #method_name (self.get_unchecked(), rhs.get_unchecked(), #ident_lower, #ident_upper)) }
            }
        }

        impl #impl_generics std::ops:: #assign_trait_name for #name #ty_generics #where_clause {
            fn #assign_method_name (&mut self, rhs: #name #ty_generics) {
                let lhs = self.get_unchecked();
                let rhs = rhs.get_unchecked();

                *self = unsafe { Self::new_unchecked(#ident_behavior :: #method_name (lhs, rhs, #ident_lower, #ident_upper))};
            }
        }

        impl #impl_generics std::ops:: #trait_name< #ident_uinteger > for #name #ty_generics #where_clause {
            type Output = #name #ty_generics;

            fn #method_name (self, rhs:  #ident_uinteger ) -> #name #ty_generics {
                unsafe { Self::new_unchecked(#ident_behavior :: #method_name (self.get_unchecked(), rhs, #ident_lower, #ident_upper)) }
            }
        }

        impl #impl_generics std::ops:: #assign_trait_name< #ident_uinteger > for #name #ty_generics #where_clause {
            fn #assign_method_name (&mut self, rhs:  #ident_uinteger ) {
                let lhs = self.get_unchecked();
                let rhs = rhs;

                *self = unsafe { Self::new_unchecked(#ident_behavior :: #method_name (lhs, rhs, #ident_lower, #ident_upper))};
            }
        }
    }
}
