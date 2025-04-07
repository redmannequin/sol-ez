use std::collections::BTreeMap;

use serde::Deserialize;

use crate::{error::SolGenError, idl};

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Config {
    pub program: Program,
    pub ix: BTreeMap<String, Ix>,
    pub ix_config: IxConfig,
    #[serde(default)]
    pub accounts: BTreeMap<String, Account>,
    #[serde(default)]
    pub message: BTreeMap<String, Message>,
}

impl Config {
    pub fn validate(&self) -> Result<(), SolGenError> {
        for (ix_name, ix) in self.ix.iter() {
            let mut idxs = vec![0; ix.accounts.len()];
            for (acc_name, acc) in ix.accounts.iter() {
                if idxs[acc.id] == 1 {
                    Err(anyhow::anyhow!(
                        "duplicate id in {} accounts({})",
                        ix_name,
                        acc_name
                    ))?;
                } else if acc.create && (acc.mutable | acc.signed) {
                    Err(anyhow::anyhow!(
                        "id({}) account({}) cant be create and mutable or signed",
                        ix_name,
                        acc_name
                    ))?;
                } else if let Some(ty) = &acc.r#type {
                    let _account_def = self.accounts.get(ty).ok_or(anyhow::anyhow!(
                        "id({}) account({}) type {} not defined",
                        ix_name,
                        acc_name,
                        ty
                    ))?;

                    // TODO check seed params from accounts and account seed
                }
                idxs[acc.id] = 1;
            }
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Program {
    pub name: String,
    pub version: (u8, u8, u8),
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Ix {
    pub id: usize,
    #[serde(default)]
    pub args: BTreeMap<String, Type>,
    pub accounts: BTreeMap<String, IxAccount>,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct IxConfig {
    pub discriminator_size: u8,
    pub discriminator_type: DiscriminatorType,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscriminatorType {
    Hash,
    Index,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct IxAccount {
    pub id: usize,
    #[serde(default)]
    pub create: bool,
    #[serde(default)]
    pub mutable: bool,
    #[serde(default)]
    pub signed: bool,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub seed: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Account {
    pub id: usize,
    pub payload: Message,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum Message {
    Struct(BTreeMap<String, Type>),
}

#[derive(Debug, PartialEq, Eq, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Type {
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
    Option(Box<Type>),
    FixedArray(Box<Type>, usize),
    DynamicArray(Box<Type>),
    Defined(String),
}

impl<'src> From<&'src Config> for idl::Idl<'src> {
    fn from(value: &'src Config) -> Self {
        idl::Idl {
            version: idl::Version {
                major: value.program.version.0,
                minor: value.program.version.1,
                patch: value.program.version.2,
            },
            name: value.program.name.as_str(),
            constants: vec![],
            accounts: value
                .accounts
                .iter()
                .map(|(name, account)| idl::Account {
                    name,
                    discriminator: None,
                    r#type: match &account.payload {
                        Message::Struct(fields) => idl::AccountDef {
                            kind: "struct",
                            fields: fields
                                .iter()
                                .map(|(name, ty)| idl::StructTypeDefField {
                                    name,
                                    r#type: ty.into(),
                                })
                                .collect(),
                        },
                    },
                })
                .collect(),
            instructions: value
                .ix
                .iter()
                .map(|(name, ix)| idl::Instruction {
                    name,
                    discriminator: None,
                    accounts: ix
                        .accounts
                        .iter()
                        .map(|(name, acc)| idl::InstructionAccount {
                            name,
                            is_mutable: acc.mutable,
                            is_signer: acc.signed,
                        })
                        .collect(),
                    args: ix
                        .args
                        .iter()
                        .map(|(name, ty)| idl::InstructionArg {
                            name,
                            r#type: ty.into(),
                        })
                        .collect(),
                })
                .collect(),
            types: value
                .message
                .iter()
                .map(|(name, ty)| idl::TypeDef {
                    name,
                    r#type: match ty {
                        Message::Struct(fields) => idl::TypeDefKind::Struct(idl::StructTypeDef {
                            fields: fields
                                .iter()
                                .map(|(name, ty)| idl::StructTypeDefField {
                                    name,
                                    r#type: ty.into(),
                                })
                                .collect(),
                        }),
                    },
                })
                .collect(),
            events: vec![],
            errors: vec![],
        }
    }
}

impl<'src> From<&'src Type> for idl::Type<'src> {
    fn from(value: &'src Type) -> Self {
        match value {
            Type::Bool => idl::Type::Bool,
            Type::U8 => idl::Type::U8,
            Type::U16 => idl::Type::U16,
            Type::U32 => idl::Type::U32,
            Type::U64 => idl::Type::U64,
            Type::U128 => idl::Type::U128,
            Type::I8 => idl::Type::I8,
            Type::I16 => idl::Type::I16,
            Type::I32 => idl::Type::I32,
            Type::I64 => idl::Type::I64,
            Type::I128 => idl::Type::I128,
            Type::Bytes => idl::Type::Bytes,
            Type::String => idl::Type::String,
            Type::PublicKey => idl::Type::PublicKey,
            Type::Option(ty) => idl::Type::Option(Box::new(ty.as_ref().into())),
            Type::FixedArray(ty, n) => idl::Type::FixedArray(Box::new(ty.as_ref().into()), *n),
            Type::DynamicArray(ty) => idl::Type::DynamicArray(Box::new(ty.as_ref().into())),
            Type::Defined(ty) => idl::Type::Defined(ty),
        }
    }
}
