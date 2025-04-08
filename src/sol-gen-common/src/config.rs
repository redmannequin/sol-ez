use std::{collections::BTreeMap, str::FromStr};

use serde::Deserialize;

use crate::error::SolGenError;

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
    pub seed: Option<AccountSeed>,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct AccountSeed {
    pub bump: bool,
    pub func: AccountSeedFunc,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AccountSeedFunc {
    pub inputs: Vec<String>,
    pub func: Vec<SeedType>,
}

impl<'de> Deserialize<'de> for AccountSeedFunc {
    fn deserialize<D>(deserializer: D) -> Result<AccountSeedFunc, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        AccountSeedFunc::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl FromStr for AccountSeedFunc {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split_once("=>")
            .ok_or_else(|| format!("Invalid seed func format: {}", s))
            .and_then(|(inputs, func)| {
                let inputs: Vec<String> = inputs
                    .split_once('[')
                    .and_then(|(_, inputs)| inputs.split_once(']'))
                    .map(|(inputs, _)| inputs.split(",").map(|s| String::from(s.trim())).collect())
                    .ok_or_else(|| format!("Invalid seed input format: {}", s))?;

                let func = func
                    .split('+')
                    .map(|arg| {
                        let s = arg.trim();
                        s.strip_prefix("'")
                            .and_then(|s| s.strip_suffix("'"))
                            .map(|s| Ok(SeedType::Defined(s.to_string())))
                            .unwrap_or_else(|| {
                                Ok(SeedType::Input(
                                    inputs
                                        .iter()
                                        .map(|s| s.as_str())
                                        .enumerate()
                                        .find(|(_, i)| *i == s)
                                        .map(|(i, _)| i)
                                        .ok_or_else(|| {
                                            format!("Invalid seed func arg not defined: {}", s)
                                        })?,
                                ))
                            })
                    })
                    .collect::<Result<_, String>>()?;

                Ok(AccountSeedFunc { inputs, func })
            })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SeedType {
    Defined(String),
    Input(usize),
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
