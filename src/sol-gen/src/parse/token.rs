#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub span: Span,
    pub r#type: TokenType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    // keywords
    Account,     // account
    Accounts,    // accounts
    Contract,    // contract
    Init,        // init
    Instruction, // inctruction
    Message,     // message
    Mutable,     // mutable
    // types
    Bool,   // bool
    Option, // option
    Signer, // Signer
    Str,    // str
    U64,    // u64
    U8,     // u8
    // slice of chars
    Identifer, // any indentifer
    Intager,   // any intager
    //
    Comma,     // ,
    Assign,    // =
    Colon,     // :
    SimiColon, // ;
    //
    LParam,   // (
    RParam,   // )
    LBrace,   // {
    RBrace,   // }
    LBracket, // [
    RBracket, // ]
    //
    InvalidChar(char),
}
