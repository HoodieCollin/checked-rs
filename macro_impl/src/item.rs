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
            Self::Struct(item) => item.params()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use syn::parse_quote;

    use crate::{item::enum_item::ClampedEnumItem, params::Params};

    fn generate_enum_params(item: ClampedEnumItem) -> Result<Params> {
        let params = item.params()?;

        println!("$$$$ {:#?}", params);

        Ok(params)
    }

    #[test]
    fn test_enum_simple() -> Result<()> {
        generate_enum_params(parse_quote! {
            #[usize]
            enum DoubleSentinel {
                Zero(0),
                Valid(..),
                Invalid(usize::MAX),
            }
        })?;

        Ok(())
    }

    #[test]
    fn test_enum_non_comprehensive() -> Result<()> {
        generate_enum_params(parse_quote! {
            #[usize]
            enum TenTwentyThirty {
                Ten(10),
                Twenty(20),
                Thirty(30),
            }
        })?;

        Ok(())
    }

    #[test]
    fn test_enum_multiple_exacts() -> Result<()> {
        generate_enum_params(parse_quote! {
            #[usize]
            enum SpecificValues {
                OneTwoOrSeven(1, 2, 7),
                AnythingElse(..),
            }
        })?;

        Ok(())
    }

    #[test]
    fn test_enum_multiple_ranges() -> Result<()> {
        generate_enum_params(parse_quote! {
            #[usize]
            enum HundredToThousand {
                Valid(..),
                Invalid(..100, 1000..)
            }
        })?;

        Ok(())
    }

    #[test]
    fn test_enum_nested() -> Result<()> {
        generate_enum_params(parse_quote! {
            #[usize]
            enum ResponseCode {
                Success[200..300] {
                    Okay(200),
                    Created(201),
                    Accepted(202),
                    Unknown(..),
                },
                Redirect[300..400] {
                    MultipleChoices(300),
                    MovedPermanently(301),
                    Found(302),
                    Unknown(..),
                },
                Error {
                    Client[400..500] {
                        BadRequest(400),
                        Unauthorized(401),
                        PaymentRequired(402),
                        Forbidden(403),
                        NotFound(404),
                        Unknown(..)
                    },
                    Server[500..600] {
                        Internal(500),
                        NotImplemented(501),
                        BadGateway(502),
                        ServiceUnavailable(503),
                        GatewayTimeout(504),
                        Unknown(..)
                    }
                }
            }
        })?;

        Ok(())
    }
}
