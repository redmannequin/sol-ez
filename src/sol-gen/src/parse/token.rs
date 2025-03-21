#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Token {
    pub span: Span,
    pub r#type: TokenType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    // keywords
    Contract,    // contract
    Instruction, // inctruction
    Accounts,    // accounts
    Account,     // account
    Mut,         // mut
    Init,        // init
    // types
    Option, // option
    Str,    // str
    U8,     // u8
    U64,    // u64
    Bool,   // bool
    // slice of chars
    Identifer, // any indentifer
    Intager,   // any intager
    //
    Comma,     // ,
    Assign,    // =
    Colon,     // :
    SimiColon, // ;
    //
    LParam,  // (
    RParam,  // )
    LBrace,  // {
    RBrace,  // }
    LBraket, // [
    RBracke, // ]
}
