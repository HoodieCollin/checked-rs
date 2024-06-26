use syn::parse::Parse;

use crate::params::Params;

pub mod enum_item;
pub mod struct_item;

pub enum ClampedItem {
    Enum(enum_item::ClampedEnumItem),
    Struct(struct_item::ClampedStructItem),
}

impl Parse for ClampedItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if enum_item::ClampedEnumItem::has_enum_token(input.fork())? {
            Ok(Self::Enum(input.parse()?))
        } else {
            Ok(Self::Struct(input.parse()?))
        }
    }
}

impl ClampedItem {
    pub fn params(&self) -> syn::Result<Params> {
        Ok(match self {
            Self::Enum(item) => item.params()?,
            Self::Struct(item) => item.params(),
        })
    }
}
