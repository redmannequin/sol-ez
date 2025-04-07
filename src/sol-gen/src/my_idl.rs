use crate::config;

pub struct MyIdl {
    pub version: Version,
    pub name: String,
    pub accounts: Vec<Account>,
    pub instructions: Vec<Instruction>,
    pub instruction_discriminator_size: usize,
}

pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

pub struct Account {
    pub id: u8,
    pub name: String,
    pub fields: Vec<Field>,
}

pub struct Field {
    pub name: String,
    pub ty: Type,
}

pub struct Instruction {
    pub id: u8,
    pub name: String,
    pub accounts: Vec<InstructionAccount>,
    pub args: Vec<Field>,
}

pub struct InstructionAccount {
    pub id: u8,
    pub name: String,
    pub state: IxAccountState,
    pub is_signed: bool,
    pub seed: Option<InstructionAccountSeed>,
    pub payload: Option<String>,
}

pub struct InstructionAccountSeed {
    pub is_bump: bool,
    pub inputs: Vec<String>,
    pub func: String,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum IxAccountState {
    Create,
    ReadOnly,
    Mutable,
}

impl IxAccountState {
    pub fn is_create(self) -> bool {
        self == Self::Create
    }

    pub fn is_mutable(self) -> bool {
        self == Self::Mutable
    }
}

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

impl From<config::Config> for MyIdl {
    fn from(value: config::Config) -> Self {
        MyIdl {
            version: Version {
                major: value.program.version.0,
                minor: value.program.version.1,
                patch: value.program.version.2,
            },
            name: value.program.name,
            instruction_discriminator_size: value.ix_config.discriminator_size as usize,
            accounts: value
                .accounts
                .into_iter()
                .map(|(name, acc)| Account {
                    id: acc.id as u8,
                    name,
                    fields: match acc.payload {
                        config::Message::Struct(fields) => fields
                            .into_iter()
                            .map(|(name, ty)| Field {
                                name: name.to_string(),
                                ty: ty.into(),
                            })
                            .collect(),
                    },
                })
                .collect(),
            instructions: {
                let mut sorted = value.ix.into_iter().map(|x| x).collect::<Vec<_>>();
                sorted.sort_by(|a, b| a.1.id.cmp(&b.1.id));
                sorted
            }
            .into_iter()
            .map(|(name, ix)| Instruction {
                id: ix.id as u8,
                name,
                accounts: {
                    let mut sorted = ix.accounts.into_iter().map(|x| x).collect::<Vec<_>>();
                    sorted.sort_by(|a, b| a.1.id.cmp(&b.1.id));
                    sorted
                }
                .into_iter()
                .map(|(name, acc)| InstructionAccount {
                    id: acc.id as u8,
                    name,
                    state: match (acc.create, acc.mutable) {
                        (true, _) => IxAccountState::Create,
                        (false, true) => IxAccountState::Mutable,
                        (false, false) => IxAccountState::ReadOnly,
                    },
                    is_signed: acc.signed,
                    seed: None,
                    payload: acc.r#type,
                })
                .collect(),
                args: ix
                    .args
                    .iter()
                    .map(|field| Field {
                        name: field.0.to_string(),
                        ty: Type::from(field.1.clone()),
                    })
                    .collect(),
            })
            .collect(),
        }
    }
}

impl From<config::Type> for Type {
    fn from(value: config::Type) -> Self {
        match value {
            config::Type::Bool => Type::Bool,
            config::Type::U8 => Type::U8,
            config::Type::U16 => Type::U16,
            config::Type::U32 => Type::U32,
            config::Type::U64 => Type::U64,
            config::Type::U128 => Type::U128,
            config::Type::I8 => Type::I8,
            config::Type::I16 => Type::I16,
            config::Type::I32 => Type::I32,
            config::Type::I64 => Type::I64,
            config::Type::I128 => Type::I128,
            config::Type::Bytes => Type::Bytes,
            config::Type::String => Type::String,
            config::Type::PublicKey => Type::PublicKey,
            config::Type::Option(ty) => Type::Option(Box::new(Type::from(*ty))),
            config::Type::FixedArray(ty, n) => Type::FixedArray(Box::new(Type::from(*ty)), n),
            config::Type::DynamicArray(ty) => Type::DynamicArray(Box::new(Type::from(*ty))),
            config::Type::Defined(ty) => Type::Defined(ty),
        }
    }
}
