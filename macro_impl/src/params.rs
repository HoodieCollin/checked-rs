use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use syn::{parse::Parse, parse_quote, spanned::Spanned};

/// Custom keywords used when parsing the `clamped` attribute.
mod kw {
    syn::custom_keyword!(default);
    syn::custom_keyword!(behavior);
    syn::custom_keyword!(lower);
    syn::custom_keyword!(upper);
    syn::custom_keyword!(Saturating);
    syn::custom_keyword!(Panicking);
    syn::custom_keyword!(MIN);
    syn::custom_keyword!(MAX);
}

/// Represents the size of unsigned integer.
#[derive(Debug, Clone)]
pub enum UIntKind {
    U8,
    U16,
    U32,
    U64,
    U128,
}

impl Parse for UIntKind {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;
        match ident.to_string().as_str() {
            "u8" => Ok(Self::U8),
            "u16" => Ok(Self::U16),
            "u32" => Ok(Self::U32),
            "u64" => Ok(Self::U64),
            "u128" => Ok(Self::U128),
            _ => abort!(ident, "expected unsigned integer type"),
        }
    }
}

impl ToTokens for UIntKind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let kind = match self {
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::U128 => "u128",
        };

        tokens.extend(syn::parse_str::<TokenStream>(kind).unwrap());
    }
}

/// Represents the `MIN` or `MAX` keyword.
#[derive(Clone)]
pub enum MinOrMax {
    Min(kw::MIN),
    Max(kw::MAX),
}

impl Parse for MinOrMax {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(kw::MIN) {
            Ok(Self::Min(input.parse()?))
        } else if input.peek(kw::MAX) {
            Ok(Self::Max(input.parse()?))
        } else {
            Err(input.error("expected `MIN` or `MAX`"))
        }
    }
}

impl ToTokens for MinOrMax {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Min(kw) => kw.to_tokens(tokens),
            Self::Max(kw) => kw.to_tokens(tokens),
        }
    }
}

/// Represents the `MIN` or `MAX` keyword.
#[derive(Clone)]
pub enum SemiOrComma {
    Semi(syn::Token![;]),
    Comma(syn::Token![,]),
}

impl Parse for SemiOrComma {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::Token![;]) {
            Ok(Self::Semi(input.parse()?))
        } else if input.peek(syn::Token![,]) {
            Ok(Self::Comma(input.parse()?))
        } else {
            Err(input.error("expected `;` or `,`"))
        }
    }
}

impl ToTokens for SemiOrComma {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Semi(kw) => kw.to_tokens(tokens),
            Self::Comma(kw) => kw.to_tokens(tokens),
        }
    }
}

/// Represents the unsigned integer argument. It can be a literal or a the MIN/MAX constant.
#[derive(Clone)]
pub enum UIntegerArg {
    Literal(syn::LitInt),
    Constant {
        kind: UIntKind,
        dbl_colon: syn::Token![::],
        ident: MinOrMax,
    },
}

impl Parse for UIntegerArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::LitInt) {
            Ok(Self::Literal(input.parse()?))
        } else {
            let kind = input.parse()?;
            let dbl_colon = input.parse()?;
            let ident: MinOrMax = input.parse()?;

            Ok(Self::Constant {
                kind,
                dbl_colon,
                ident,
            })
        }
    }
}

impl ToTokens for UIntegerArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Literal(lit) => lit.to_tokens(tokens),
            Self::Constant {
                kind,
                dbl_colon,
                ident,
            } => {
                let kind = kind.to_token_stream();
                let dbl_colon = dbl_colon.to_token_stream();
                let ident = ident.to_token_stream();

                tokens.extend(quote! {
                    #kind #dbl_colon #ident
                });
            }
        }
    }
}

impl UIntegerArg {
    /// Output the value as a bare literal number in a token stream.
    pub fn into_literal_as_tokens(&self) -> TokenStream {
        syn::parse_str(self.base10_parse::<u128>().unwrap().to_string().as_str()).unwrap()
    }

    /// Parse the value as a base 10 number.
    pub fn base10_parse<N>(&self) -> syn::Result<N>
    where
        N: std::str::FromStr,
        N::Err: std::fmt::Display,
    {
        match self {
            Self::Literal(lit) => lit.base10_parse::<N>(),
            Self::Constant {
                kind,
                dbl_colon: _,
                ident,
            } => {
                let n = match ident {
                    MinOrMax::Min(..) => match kind {
                        UIntKind::U8 => u8::MIN.to_string(),
                        UIntKind::U16 => u16::MIN.to_string(),
                        UIntKind::U32 => u32::MIN.to_string(),
                        UIntKind::U64 => u64::MIN.to_string(),
                        UIntKind::U128 => u128::MIN.to_string(),
                    },
                    MinOrMax::Max(..) => match kind {
                        UIntKind::U8 => u8::MAX.to_string(),
                        UIntKind::U16 => u16::MAX.to_string(),
                        UIntKind::U32 => u32::MAX.to_string(),
                        UIntKind::U64 => u64::MAX.to_string(),
                        UIntKind::U128 => u128::MAX.to_string(),
                    },
                };

                match str::parse(&n) {
                    Ok(n) => Ok(n),
                    Err(e) => Err(syn::Error::new(ident.span(), e)),
                }
            }
        }
    }
}

/// Represents the behavior argument. It can be `Saturating` or `Panicking`.
#[derive(Clone)]
pub enum BehaviorArg {
    Saturating(kw::Saturating),
    Panicking(kw::Panicking),
}

impl Parse for BehaviorArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(kw::Saturating) {
            Ok(Self::Saturating(input.parse()?))
        } else if input.peek(kw::Panicking) {
            Ok(Self::Panicking(input.parse()?))
        } else {
            Err(input.error("expected `Saturating` or `Panicking`"))
        }
    }
}

impl ToTokens for BehaviorArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            Self::Saturating(..) => quote! {
                Saturating
            },
            Self::Panicking(..) => quote! {
                Panicking
            },
        });
    }
}

/// Represents the parameters of the `clamped` attribute.
/// Only the `uinteger` and `default` parameters are required.
/// The `uinteger` parameter must be first while the order of the rest is not important.
#[derive(Clone)]
pub struct ClampParams {
    pub uinteger: syn::TypePath,
    pub uinteger_semi: SemiOrComma,
    pub default_kw: kw::default,
    pub default_eq: syn::Token![=],
    pub default_val: UIntegerArg,
    pub default_semi: Option<SemiOrComma>,
    pub behavior_kw: kw::behavior,
    pub behavior_eq: syn::Token![=],
    pub behavior_val: BehaviorArg,
    pub behavior_semi: Option<SemiOrComma>,
    pub lower_kw: Option<kw::lower>,
    pub lower_eq: Option<syn::Token![=]>,
    pub lower_val: Option<UIntegerArg>,
    pub lower_semi: Option<SemiOrComma>,
    pub upper_kw: Option<kw::upper>,
    pub upper_eq: Option<syn::Token![=]>,
    pub upper_val: Option<UIntegerArg>,
    pub upper_semi: Option<SemiOrComma>,
}

impl Parse for ClampParams {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let uinteger = input.parse()?;
        let uinteger_semi = input.parse()?;
        let mut default_kw = None;
        let mut default_eq = None;
        let mut default_val = None;
        let mut default_semi = None;
        let mut behavior_kw = None;
        let mut behavior_eq = None;
        let mut behavior_val = None;
        let mut behavior_semi = None;
        let mut lower_kw = None;
        let mut lower_eq = None;
        let mut lower_val = None;
        let mut lower_semi = None;
        let mut upper_kw = None;
        let mut upper_eq = None;
        let mut upper_val = None;
        let mut upper_semi = None;

        let mut done = false;

        while !done {
            let mut found_semi = false;

            if input.peek(kw::default) {
                if default_kw.is_some() {
                    return Err(input.error("duplicate `default` param"));
                }

                default_kw = Some(input.parse::<kw::default>()?);
                default_eq = Some(input.parse::<syn::Token![=]>()?);
                default_val = Some(input.parse::<UIntegerArg>()?);
                if !input.is_empty() {
                    default_semi = Some(input.parse::<SemiOrComma>()?);
                    found_semi = true;
                }
            } else if input.peek(kw::behavior) {
                if behavior_kw.is_some() {
                    return Err(input.error("duplicate `behavior` param"));
                }

                behavior_kw = Some(input.parse::<kw::behavior>()?);
                behavior_eq = Some(input.parse::<syn::Token![=]>()?);
                behavior_val = Some(input.parse::<BehaviorArg>()?);
                if !input.is_empty() {
                    behavior_semi = Some(input.parse::<SemiOrComma>()?);
                    found_semi = true;
                }
            } else if input.peek(kw::lower) {
                if lower_kw.is_some() {
                    return Err(input.error("duplicate `lower` param"));
                }

                lower_kw = Some(input.parse::<kw::lower>()?);
                lower_eq = Some(input.parse::<syn::Token![=]>()?);
                lower_val = Some(input.parse::<UIntegerArg>()?);
                if !input.is_empty() {
                    lower_semi = Some(input.parse::<SemiOrComma>()?);
                    found_semi = true;
                }
            } else if input.peek(kw::upper) {
                if upper_kw.is_some() {
                    return Err(input.error("duplicate `upper` param"));
                }

                upper_kw = Some(input.parse::<kw::upper>()?);
                upper_eq = Some(input.parse::<syn::Token![=]>()?);
                upper_val = Some(input.parse::<UIntegerArg>()?);
                if !input.is_empty() {
                    upper_semi = Some(input.parse::<SemiOrComma>()?);
                    found_semi = true;
                }
            }

            if !found_semi {
                if default_kw.is_none() {
                    return Err(input.error("`default` param is required"));
                }

                done = true;
            }
        }

        if behavior_kw.is_none() {
            behavior_kw = Some(parse_quote!(behavior));
            behavior_eq = Some(parse_quote!(=));
            behavior_val = Some(parse_quote!(Panicking));
        }

        let this = Self {
            uinteger,
            uinteger_semi,
            default_kw: default_kw.unwrap(),
            default_eq: default_eq.unwrap(),
            default_val: default_val.unwrap(),
            default_semi,
            behavior_kw: behavior_kw.unwrap(),
            behavior_eq: behavior_eq.unwrap(),
            behavior_val: behavior_val.unwrap(),
            behavior_semi,
            lower_kw,
            lower_eq,
            lower_val,
            lower_semi,
            upper_kw,
            upper_eq,
            upper_val,
            upper_semi,
        };

        if !this.is_u128_or_smaller() {
            abort!(this.uinteger, "expected unsigned integer type")
        }

        match this.kind() {
            UIntKind::U8 => {
                if this.default_val.base10_parse::<u8>().is_err() {
                    abort!(this.default_val, "expected u8 value")
                }
            }
            UIntKind::U16 => {
                if this.default_val.base10_parse::<u16>().is_err() {
                    abort!(this.default_val, "expected u16 value")
                }
            }
            UIntKind::U32 => {
                if this.default_val.base10_parse::<u32>().is_err() {
                    abort!(this.default_val, "expected u32 value")
                }
            }
            UIntKind::U64 => {
                if this.default_val.base10_parse::<u64>().is_err() {
                    abort!(this.default_val, "expected u64 value")
                }
            }
            UIntKind::U128 => {
                if this.default_val.base10_parse::<u128>().is_err() {
                    abort!(this.default_val, "expected u128 value")
                }
            }
        }

        if this.default_value() < this.lower_limit_value() {
            abort!(
                this.default_val,
                "default value is less than lower bound value"
            )
        }

        if this.default_value() > this.upper_limit_value() {
            abort!(this.default_val, "default value exceeds upper bound value")
        }

        Ok(this)
    }
}

impl ClampParams {
    /// Get the unsigned integer kind.
    pub fn kind(&self) -> UIntKind {
        self.uinteger
            .path
            .segments
            .iter()
            .last()
            .map(|s| match s.ident.to_string().as_str() {
                "u8" => UIntKind::U8,
                "u16" => UIntKind::U16,
                "u32" => UIntKind::U32,
                "u64" => UIntKind::U64,
                "u128" => UIntKind::U128,
                _ => abort!(self.uinteger, "expected unsigned integer type"),
            })
            .unwrap_or_else(|| abort!(self.uinteger, "expected unsigned integer type"))
    }

    /// Interpret the default value as `u128`.
    pub fn default_value(&self) -> u128 {
        if let Ok(n) = self.default_val.base10_parse() {
            n
        } else {
            abort!(self.default_val, "expected integer value")
        }
    }

    /// Get the behavior type.
    pub fn behavior_type(&self) -> &BehaviorArg {
        &self.behavior_val
    }

    /// Interpret the lower limit value as `u128`.
    pub fn lower_limit_value(&self) -> u128 {
        if let Some(val) = &self.lower_val {
            if let Ok(n) = val.base10_parse() {
                n
            } else {
                abort!(val, "expected integer value")
            }
        } else {
            0
        }
    }

    /// Output the lower limit value as a bare literal in a token stream.
    pub fn lower_limit_token(&self) -> TokenStream {
        syn::parse_str(&self.lower_limit_value().to_string()).unwrap()
    }

    /// Interpret the upper limit value as `u128`.
    pub fn upper_limit_value(&self) -> u128 {
        if let Some(val) = &self.upper_val {
            if let Ok(n) = val.base10_parse() {
                n
            } else {
                abort!(val, "expected integer value")
            }
        } else {
            match self.kind() {
                UIntKind::U8 => u8::MAX as u128,
                UIntKind::U16 => u16::MAX as u128,
                UIntKind::U32 => u32::MAX as u128,
                UIntKind::U64 => u64::MAX as u128,
                UIntKind::U128 => u128::MAX,
            }
        }
    }

    /// Output the upper limit value as a bare literal in a token stream.
    pub fn upper_limit_token(&self) -> TokenStream {
        syn::parse_str(&self.upper_limit_value().to_string()).unwrap()
    }

    /// Validate that an arbitrary value is within the lower and upper limit.
    pub fn abort_if_out_of_bounds<T: Spanned + ToTokens>(&self, ast: &T, value: u128) {
        if value < self.lower_limit_value() {
            abort!(
                ast,
                "{:?} value: {} is less than lower limit: {}",
                self.kind(),
                value,
                self.lower_limit_value()
            )
        }

        if value > self.upper_limit_value() {
            abort!(
                ast,
                "{:?} value: {} exceeds upper limit: {}",
                self.kind(),
                value,
                self.upper_limit_value()
            )
        }
    }

    /// Check if the unsigned integer kind is `u16` or smaller.
    pub fn is_u16_or_smaller(&self) -> bool {
        matches!(self.kind(), UIntKind::U8 | UIntKind::U16)
    }

    /// Check if the unsigned integer kind is `u16` or larger.
    pub fn is_u16_or_larger(&self) -> bool {
        matches!(
            self.kind(),
            UIntKind::U16 | UIntKind::U32 | UIntKind::U64 | UIntKind::U128
        )
    }

    /// Check if the unsigned integer kind is `u32` or smaller.
    pub fn is_u32_or_smaller(&self) -> bool {
        matches!(self.kind(), UIntKind::U8 | UIntKind::U16 | UIntKind::U32)
    }

    /// Check if the unsigned integer kind is `u32` or larger.
    pub fn is_u32_or_larger(&self) -> bool {
        matches!(self.kind(), UIntKind::U32 | UIntKind::U64 | UIntKind::U128)
    }

    /// Check if the unsigned integer kind is `u64` or smaller.
    pub fn is_u64_or_smaller(&self) -> bool {
        matches!(
            self.kind(),
            UIntKind::U8 | UIntKind::U16 | UIntKind::U32 | UIntKind::U64
        )
    }

    /// Check if the unsigned integer kind is `u64` or larger.
    pub fn is_u64_or_larger(&self) -> bool {
        matches!(self.kind(), UIntKind::U64 | UIntKind::U128)
    }

    /// Check if the unsigned integer kind is `u128` or smaller.
    pub fn is_u128_or_smaller(&self) -> bool {
        matches!(
            self.kind(),
            UIntKind::U8 | UIntKind::U16 | UIntKind::U32 | UIntKind::U64 | UIntKind::U128
        )
    }
}

/// Represents the expected generic parameters for the internal `CheckedRsOps` derive macro.
pub struct GenericParams {
    uinteger_original: Option<syn::Ident>,
    uinteger: syn::GenericParam,
    behavior: syn::GenericParam,
    lower: syn::GenericParam,
    upper: syn::GenericParam,
    extras: Vec<syn::GenericParam>,
    where_clause: Option<syn::WhereClause>,
}

impl Clone for GenericParams {
    fn clone(&self) -> Self {
        Self {
            uinteger_original: self.uinteger_original.clone(),
            uinteger: self.uinteger.clone(),
            behavior: self.behavior.clone(),
            lower: self.lower.clone(),
            upper: self.upper.clone(),
            extras: self.extras.clone(),
            where_clause: self.where_clause.clone(),
        }
    }
}

impl GenericParams {
    pub fn from_input(input: &syn::DeriveInput) -> Self {
        let count = input.generics.params.len();
        if count < 4 {
            abort!(input, "expected at least 4 generic parameters")
        }

        let mut iter = input.generics.params.iter();

        let uinteger = iter.next().cloned().unwrap();
        let behavior = iter.next().cloned().unwrap();

        let mut extras = Vec::with_capacity(count - 4);

        for _ in 0..count - 4 {
            extras.push(iter.next().cloned().unwrap());
        }

        let lower = iter.next().cloned().unwrap();
        let upper = iter.next().cloned().unwrap();

        Self {
            uinteger_original: None,
            uinteger,
            behavior,
            lower,
            upper,
            extras,
            where_clause: input.generics.where_clause.clone(),
        }
    }

    /// Replace the `uinteger` generic parameter with a new one.
    pub fn with_uinteger_ident(&self, uinteger: syn::Ident) -> Self {
        let mut this = self.clone();
        this.uinteger_original = Some(self.uinteger_ident().clone());
        this.uinteger = parse_quote!(#uinteger: UInteger);
        this
    }

    /// Get the current `uinteger` generic parameter.
    pub fn uinteger(&self) -> &syn::GenericParam {
        &self.uinteger
    }

    /// Get the current `uinteger` generic parameter identifier.
    pub fn uinteger_ident(&self) -> &syn::Ident {
        match &self.uinteger {
            syn::GenericParam::Type(ty) => &ty.ident,
            _ => abort!(self.uinteger, "expected type parameter"),
        }
    }

    /// Get the current `behavior` generic parameter.
    pub fn behavior(&self) -> &syn::GenericParam {
        &self.behavior
    }

    /// Get the current `behavior` generic parameter identifier.
    pub fn behavior_ident(&self) -> &syn::Ident {
        match &self.behavior {
            syn::GenericParam::Type(ty) => &ty.ident,
            _ => abort!(self.behavior, "expected type parameter"),
        }
    }

    /// Get the current `lower` generic parameter.
    pub fn lower(&self) -> &syn::GenericParam {
        &self.lower
    }

    /// Get the current `lower` generic parameter identifier.
    pub fn lower_ident(&self) -> &syn::Ident {
        match &self.lower {
            syn::GenericParam::Const(c) => &c.ident,
            _ => abort!(self.lower, "expected const parameter"),
        }
    }

    /// Get the current `upper` generic parameter.
    pub fn upper(&self) -> &syn::GenericParam {
        &self.upper
    }

    /// Get the current `upper` generic parameter identifier.
    pub fn upper_ident(&self) -> &syn::Ident {
        match &self.upper {
            syn::GenericParam::Const(c) => &c.ident,
            _ => abort!(self.upper, "expected const parameter"),
        }
    }

    /// Get the current extra generic parameters. If the `uinteger` parameter was replaced, it will be updated recursively in the extras.
    pub fn extras(&self) -> Vec<syn::GenericParam> {
        if let Some(ident_uinteger) = self.uinteger_original.as_ref() {
            let alt_ident_uinteger = self.uinteger_ident();

            use syn::visit_mut::{self, VisitMut};

            struct Replacer<'a> {
                pub ident_uinteger: &'a syn::Ident,
                pub alt_ident_uinteger: &'a syn::Ident,
            }

            impl VisitMut for Replacer<'_> {
                fn visit_type_path_mut(&mut self, node: &mut syn::TypePath) {
                    let ident_uinteger = self.ident_uinteger;
                    let alt_ident_uinteger = self.alt_ident_uinteger;

                    if node.path.is_ident(ident_uinteger) {
                        node.path = parse_quote!(#alt_ident_uinteger);
                    }

                    visit_mut::visit_type_path_mut(self, node);
                }
            }

            let mut replacer = Replacer {
                ident_uinteger,
                alt_ident_uinteger: &alt_ident_uinteger,
            };

            self.extras
                .iter()
                .cloned()
                .map(|mut p| {
                    replacer.visit_generic_param_mut(&mut p);
                    p
                })
                .collect()
        } else {
            self.extras.clone()
        }
    }

    /// Get the current extra generic parameter identifiers.
    pub fn extra_idents(&self) -> Vec<syn::Ident> {
        self.extras()
            .into_iter()
            .map(|p| match p {
                syn::GenericParam::Type(ty) => ty.ident,
                syn::GenericParam::Const(c) => c.ident,
                _ => abort!(p, "expected type or const parameter"),
            })
            .collect()
    }

    /// Get the tokens for the `impl` generics, the type generics, and the `where` clause.
    pub fn split_for_impl(&self) -> (syn::Generics, syn::Generics, Option<syn::WhereClause>) {
        let uinteger_param = self.uinteger();
        let behavior_param = self.behavior();
        let lower_param = self.lower();
        let upper_param = self.upper();
        let extra_params = self.extras();

        let impl_generics: syn::Generics = if self.uinteger_original.is_none() {
            parse_quote! {
                <
                    #uinteger_param,
                    #behavior_param,
                    #(#extra_params,)*
                    #lower_param,
                    #upper_param
                >
            }
        } else {
            parse_quote! {
                <
                    #behavior_param,
                    #(#extra_params,)*
                    #lower_param,
                    #upper_param
                >
            }
        };

        let uinteger_ident = self.uinteger_ident();
        let behavior_ident = self.behavior_ident();
        let lower_ident = self.lower_ident();
        let upper_ident = self.upper_ident();
        let extra_idents = self.extra_idents();

        let ty_generics: syn::Generics = parse_quote! {
            <
                #uinteger_ident,
                #behavior_ident,
                #(#extra_idents,)*
                #lower_ident,
                #upper_ident
            >
        };

        (impl_generics, ty_generics, self.where_clause.clone())
    }
}

#[cfg(test)]
mod tests {
    use quote::ToTokens;

    use super::*;

    #[test]
    fn extras_mapped() {
        let input = parse_quote! {
            struct Foo<T: UInteger, B: Behavior<T>, V, W: AsRef<T>, X, const L: u128, const U: u128> {
                t: T,
                b: B,
                v: V,
                w: W,
                x: X,
            }
        };

        let params = GenericParams::from_input(&input);

        // let extras = params
        //     .extras(Some(parse_quote!(u8)))
        //     .into_iter()
        //     .map(|p| p.to_token_stream().to_string())
        //     .collect::<Vec<_>>();

        // assert_eq!(extras.len(), 3);

        // println!("{:#?}", extras);

        let u8_params = params.with_uinteger_ident(parse_quote!(u8));

        let (impl_generics, ty_generics, where_clause) = u8_params.split_for_impl();

        println!(
            "impl_generics: {}",
            impl_generics.to_token_stream().to_string()
        );
        println!("ty_generics: {}", ty_generics.to_token_stream().to_string());
        println!(
            "where_clause: {:?}",
            where_clause.map(|w| w.to_token_stream().to_string())
        );
    }
}
