use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::Parse;

use super::{NumberArg, NumberKind, NumberValue, NumberValueRange};

#[derive(Clone)]
pub struct NumberArgRange {
    pub start: Option<NumberArg>,
    pub dot_dot: Option<syn::Token![..]>,
    pub dot_dot_eq: Option<syn::Token![..=]>,
    pub end: Option<NumberArg>,
}

impl Parse for NumberArgRange {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut start = None;
        let mut dot_dot = None;
        let mut dot_dot_eq = None;
        let mut end = None;

        let lookahead = input.lookahead1();

        if lookahead.peek(syn::Token![..=]) {
            dot_dot_eq = Some(input.parse()?);

            if !input.is_empty() {
                end = Some(input.parse()?);
            }
        } else if lookahead.peek(syn::Token![..]) {
            dot_dot = Some(input.parse()?);

            if !input.is_empty() {
                end = Some(input.parse()?);
            }
        } else if lookahead.peek(syn::LitInt) {
            start = Some(input.parse()?);

            if input.peek(syn::Token![..=]) {
                dot_dot_eq = Some(input.parse()?);

                if !input.is_empty() {
                    end = Some(input.parse()?);
                }
            } else if input.peek(syn::Token![..]) {
                dot_dot = Some(input.parse()?);

                if !input.is_empty() {
                    end = Some(input.parse()?);
                }
            } else {
                return Err(lookahead.error());
            }
        } else {
            return Err(lookahead.error());
        }

        Ok(Self {
            start,
            dot_dot,
            dot_dot_eq,
            end,
        })
    }
}

impl ToTokens for NumberArgRange {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let start = self.start.as_ref();
        let dot_dot = self.dot_dot.as_ref();
        let dot_dot_eq = self.dot_dot_eq.as_ref();
        let end = self.end.as_ref();

        tokens.extend(quote! {
            #start #dot_dot #dot_dot_eq #end
        });
    }
}

impl std::fmt::Debug for NumberArgRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dot_dot = self.dot_dot.as_ref().map(|_| "..".to_string());
        let dot_dot_eq = self.dot_dot_eq.as_ref().map(|_| "..=".to_string());

        write!(
            f,
            "{}{}{}{}",
            self.start
                .as_ref()
                .map(|arg| arg.to_string())
                .unwrap_or_default(),
            dot_dot.unwrap_or_default(),
            dot_dot_eq.unwrap_or_default(),
            self.end
                .as_ref()
                .map(|arg| arg.to_string())
                .unwrap_or_default()
        )
    }
}

impl std::fmt::Display for NumberArgRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl NumberArgRange {
    pub fn new_exclusive(start: NumberArg, end: NumberArg) -> Self {
        Self {
            start: Some(start),
            dot_dot_eq: None,
            dot_dot: Some(syn::Token![..](Span::call_site())),
            end: Some(end),
        }
    }

    pub fn new_inclusive(start: NumberArg, end: NumberArg) -> Self {
        Self {
            start: Some(start),
            dot_dot_eq: Some(syn::Token![..=](Span::call_site())),
            dot_dot: None,
            end: Some(end),
        }
    }

    pub fn start_arg(&self, kind: NumberKind) -> NumberArg {
        self.start
            .as_ref()
            .cloned()
            .unwrap_or_else(|| NumberArg::new_min_constant(kind))
    }

    pub fn first_val(&self, kind: NumberKind) -> NumberValue {
        self.start_arg(kind).into_value(kind)
    }

    pub fn end_arg(&self, kind: NumberKind) -> NumberArg {
        self.end
            .as_ref()
            .cloned()
            .unwrap_or_else(|| NumberArg::new_max_constant(kind))
    }

    pub fn last_val(&self, kind: NumberKind) -> NumberValue {
        if let Some(end_arg) = &self.end {
            let val = end_arg.into_value(kind);

            if self.dot_dot_eq.is_some() {
                val
            } else {
                val.sub_usize(1)
            }
        } else {
            NumberArg::new_max_constant(kind).into_value(kind)
        }
    }

    pub fn is_full_range(&self) -> bool {
        self.start.is_none() && self.end.is_none()
    }

    pub fn to_value_range(&self, kind: NumberKind) -> syn::Result<NumberValueRange> {
        NumberValueRange::from_arg_range(self.clone(), kind)
    }

    pub fn iter(&self, kind: NumberKind) -> impl Iterator<Item = NumberArg> {
        self.iter_values(kind).map(|val| val.into_number_arg())
    }

    pub fn iter_values(&self, kind: NumberKind) -> impl Iterator<Item = NumberValue> {
        let first = self.first_val(kind);
        let last = self.last_val(kind);

        first.iter_to(last.add_usize(1))
    }
}

#[derive(Clone)]
pub struct StrictNumberArgRange(pub NumberArgRange);

impl Parse for StrictNumberArgRange {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let range: NumberArgRange = input.parse()?;

        if range.start.is_none() && range.end.is_none() {
            Err(input.error("Should not be a full range"))
        } else {
            Ok(Self(range))
        }
    }
}

impl ToTokens for StrictNumberArgRange {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

impl std::fmt::Debug for StrictNumberArgRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

impl std::fmt::Display for StrictNumberArgRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl std::ops::Deref for StrictNumberArgRange {
    type Target = NumberArgRange;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
