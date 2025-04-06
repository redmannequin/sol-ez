use std::str::FromStr;

use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Idl<'src> {
    pub version: Version,
    pub name: &'src str,
    #[serde(default)]
    pub constants: Vec<Constant<'src>>,
    #[serde(default)]
    pub accounts: Vec<Account<'src>>,
    #[serde(default)]
    pub instructions: Vec<Instruction<'src>>,
    #[serde(default)]
    pub types: Vec<TypeDef<'src>>,
    #[serde(default)]
    pub events: Vec<Event<'src>>,
    #[serde(default)]
    pub errors: Vec<Error<'src>>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Type<'src> {
    Bool,
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    Bytes,
    String,
    PublicKey,
    Option(Box<Type<'src>>),
    #[serde(rename = "array")]
    FixedArray(Box<Type<'src>>, usize),
    DynamicArray(Box<Type<'src>>),
    Defined(&'src str),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Version, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Version::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl FromStr for Version {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid version format: {}", s));
        }

        let major = parts[0]
            .parse()
            .map_err(|e| format!("Invalid major: {}", e))?;
        let minor = parts[1]
            .parse()
            .map_err(|e| format!("Invalid minor: {}", e))?;
        let patch = parts[2]
            .parse()
            .map_err(|e| format!("Invalid patch: {}", e))?;

        Ok(Version {
            major,
            minor,
            patch,
        })
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Constant<'src> {
    pub name: &'src str,
    pub r#type: Type<'src>,
    pub value: &'src str,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Account<'src> {
    pub name: &'src str,
    pub discriminator: Option<Discriminator>,
    pub r#type: AccountDef<'src>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct AccountDef<'src> {
    pub kind: &'src str,
    pub fields: Vec<StructTypeDefField<'src>>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Instruction<'src> {
    pub name: &'src str,
    pub discriminator: Option<Discriminator>,
    pub accounts: Vec<InstructionAccount<'src>>,
    pub args: Vec<InstructionArg<'src>>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct Discriminator(pub Vec<u8>);

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct InstructionAccount<'src> {
    pub name: &'src str,
    #[serde(rename = "isMut")]
    pub is_mutable: bool,
    #[serde(rename = "isSigner")]
    pub is_signer: bool,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct InstructionArg<'src> {
    pub name: &'src str,
    pub r#type: Type<'src>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct TypeDef<'src> {
    pub name: &'src str,
    pub r#type: TypeDefKind<'src>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum TypeDefKind<'src> {
    #[serde(borrow)]
    Struct(StructTypeDef<'src>),
    #[serde(borrow)]
    Enum(EnumTypeDef<'src>),
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct StructTypeDef<'src> {
    #[serde(borrow)]
    pub fields: Vec<StructTypeDefField<'src>>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct StructTypeDefField<'src> {
    pub name: &'src str,
    pub r#type: Type<'src>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct EnumTypeDef<'src> {
    #[serde(borrow)]
    pub variants: Vec<EnumTypeDefVariant<'src>>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct EnumTypeDefVariant<'src> {
    pub name: &'src str,
    pub fields: Option<Vec<EnumTypeDefVariantField<'src>>>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct EnumTypeDefVariantField<'src> {
    pub name: &'src str,
    pub r#type: Type<'src>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Event<'src> {
    pub name: &'src str,
    pub discriminator: Option<Discriminator>,
    pub fields: Vec<EventField<'src>>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct EventField<'src> {
    pub name: &'src str,
    pub r#type: Type<'src>,
    pub index: bool,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Error<'src> {
    pub code: u32,
    pub name: &'src str,
    pub msg: &'src str,
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use crate::idl::{
        Account, AccountDef, Constant, Discriminator, EnumTypeDef, EnumTypeDefVariant,
        EnumTypeDefVariantField, Error, Event, EventField, Idl, Instruction, InstructionAccount,
        InstructionArg, StructTypeDef, StructTypeDefField, Type, TypeDef, TypeDefKind, Version,
    };

    fn deserialize<'a, T: Deserialize<'a>>(src: &'a str) -> T {
        let value = &mut serde_json::Deserializer::from_str(src);
        serde_path_to_error::deserialize(value).expect("failed to parse")
    }

    #[test]
    fn test_idl() {
        let src = r#"
            {
                "version": "0.1.0",
                "name": "test",
                "constants": [{
                    "name": "MAX_LEN",
                    "type": { "defined": "usize" },
                    "value": "30"
                }],
                "accounts": [{
                    "name": "Access",
                    "discriminator": [117, 154, 108, 210, 202, 83, 96, 222],
                    "type": {
                        "kind": "struct",
                        "fields": [{
                            "name": "wallet",
                            "docs": ["The pubkey of the wallet being granted access (32)."],
                            "type": "publicKey"
                        }]
                    }   
                }],
                "instructions": [{
                    "name": "create",
                    "discriminator": [232, 16, 8, 240, 20, 109, 214, 66],
                    "accounts": [{
                        "name": "master",
                        "isMut": true,
                        "isSigner": false
                    }],
                    "args": [{
                        "name": "name",
                        "type": "string"
                    }]
                }],
                "types": [{
                    "name": "Creators",
                    "type": {
                        "kind": "struct",
                        "fields": [{
                            "name": "address",
                            "type": "publicKey"
                        },{
                            "name": "share",
                            "type": "u8"
                        }]
                    }
                }],
                "events": [{
                    "name": "Updated",
                    "discriminator": [94, 250, 56, 174, 183, 63, 43, 169],
                    "fields": [{
                        "name": "xnft",
                        "type": "publicKey",
                        "index": false
                    }]
                }],
                "errors": [{
                    "code": 6000,
                    "name": "CannotReviewOwned",
                    "msg": "You cannot create a review"
                }]
            }
        "#;
        let value = deserialize(src);
        assert_eq!(
            Idl {
                version: Version {
                    major: 0,
                    minor: 1,
                    patch: 0
                },
                name: "test",
                constants: vec![Constant {
                    name: "MAX_LEN",
                    r#type: Type::Defined("usize"),
                    value: "30"
                }],
                accounts: vec![Account {
                    name: "Access",
                    discriminator: Some(Discriminator(vec![117, 154, 108, 210, 202, 83, 96, 222])),
                    r#type: AccountDef {
                        kind: "struct",
                        fields: vec![StructTypeDefField {
                            name: "wallet",
                            r#type: Type::PublicKey
                        }]
                    }
                }],
                instructions: vec![Instruction {
                    name: "create",
                    discriminator: Some(Discriminator(vec![232, 16, 8, 240, 20, 109, 214, 66])),
                    accounts: vec![InstructionAccount {
                        name: "master",
                        is_mutable: true,
                        is_signer: false
                    }],
                    args: vec![InstructionArg {
                        name: "name",
                        r#type: Type::String
                    }]
                }],
                types: vec![TypeDef {
                    name: "Creators",
                    r#type: TypeDefKind::Struct(StructTypeDef {
                        fields: vec![
                            StructTypeDefField {
                                name: "address",
                                r#type: Type::PublicKey
                            },
                            StructTypeDefField {
                                name: "share",
                                r#type: Type::U8
                            }
                        ]
                    })
                }],
                events: vec![Event {
                    name: "Updated",
                    discriminator: Some(Discriminator(vec![94, 250, 56, 174, 183, 63, 43, 169])),
                    fields: vec![EventField {
                        name: "xnft",
                        r#type: Type::PublicKey,
                        index: false
                    }]
                }],
                errors: vec![Error {
                    code: 6000,
                    name: "CannotReviewOwned",
                    msg: "You cnnot create a review",
                }]
            },
            value
        );
    }

    #[test]
    fn test_struct_type_def() {
        let src = r#"
            {
                "name": "CuratorStatus",
                "type": {
                    "kind": "struct",
                    "fields": [{
                        "name": "pubkey",
                        "docs": ["The pubkey of the `Curator` program account (32)."],
                        "type": "publicKey"
                    },{
                        "name": "verified",
                        "docs": [
                            "Whether the curator's authority has verified the assignment (1)."
                        ],
                        "type": "bool"
                    }]
                }
            }
        "#;
        let value = deserialize(src);
        assert_eq!(
            TypeDef {
                name: "CuratorStatus",
                r#type: TypeDefKind::Struct(StructTypeDef {
                    fields: vec![
                        StructTypeDefField {
                            name: "pubkey",
                            r#type: Type::PublicKey
                        },
                        StructTypeDefField {
                            name: "verified",
                            r#type: Type::Bool
                        }
                    ]
                })
            },
            value
        )
    }

    #[test]
    fn test_enum_type_def() {
        let src = r#"
            {
                "name": "SupplyType",
                "type": {
                    "kind": "enum",
                    "variants": [{
                        "name": "Infinite"
                    },{
                        "name": "Limited",
                        "fields": [{
                            "name": "value",
                            "type": "u64"
                        }]
                    }]
                }
            }
        "#;
        let value = deserialize(src);
        assert_eq!(
            TypeDef {
                name: "SupplyType",
                r#type: TypeDefKind::Enum(EnumTypeDef {
                    variants: vec![
                        EnumTypeDefVariant {
                            name: "Infinite",
                            fields: None
                        },
                        EnumTypeDefVariant {
                            name: "Limited",
                            fields: Some(vec![EnumTypeDefVariantField {
                                name: "value",
                                r#type: Type::U64
                            }])
                        }
                    ]
                })
            },
            value
        )
    }

    #[test]
    fn test_enum_type_def_variant() {
        let src = r#"
            {
                "name": "None"
            }
        "#;
        let value = deserialize::<EnumTypeDefVariant>(src);
        assert_eq!(
            EnumTypeDefVariant {
                name: "None",
                fields: None
            },
            value
        );

        let src = r#"
            {
                "name": "Some",
                "fields": [{
                    "name": "value",
                    "type": "u64"
                }]
            }
        "#;
        let value = deserialize::<EnumTypeDefVariant>(src);
        assert_eq!(
            EnumTypeDefVariant {
                name: "Some",
                fields: Some(vec![EnumTypeDefVariantField {
                    name: "value",
                    r#type: Type::U64
                }])
            },
            value
        );
    }

    #[test]
    fn test_discriminator() {
        let src = "[94, 250, 56, 174, 183, 63, 43, 169]";
        let value = deserialize::<Discriminator>(src);
        assert_eq!(
            Discriminator(vec![94, 250, 56, 174, 183, 63, 43, 169]),
            value
        );
    }

    #[test]
    fn test_event() {
        let src = r#"
            {
                "name": "XnftUpdated",
                "discriminator": [94, 250, 56, 174, 183, 63, 43, 169],
                "fields": []
            }
        "#;
        let value = deserialize::<Event>(src);
        assert_eq!(
            Event {
                name: "XnftUpdated",
                discriminator: Some(Discriminator(vec![94, 250, 56, 174, 183, 63, 43, 169])),
                fields: vec![]
            },
            value
        );
    }

    #[test]
    fn test_event_field() {
        let src = r#"
            {
                "name": "xnft",
                "type": "publicKey",
                "index": false
            }
        "#;
        let value = deserialize::<EventField>(src);
        assert_eq!(
            EventField {
                name: "xnft",
                r#type: Type::PublicKey,
                index: false
            },
            value
        );
    }

    #[test]
    fn test_error() {
        let src = r#"
            {
                "code": 6011,
                "name": "SupplyReduction",
                "msg": "Updated supply"
            }
        "#;
        let value = deserialize::<Error>(src);
        assert_eq!(
            Error {
                code: 6011,
                name: "SupplyReduction",
                msg: "Updated supply",
            },
            value
        );
    }
}
