use std::str::FromStr;

use base64::{Engine, prelude::BASE64_STANDARD};
use solana_pubkey::Pubkey;

use crate::{
    Result,
    raw_log::{
        RawCuLog, RawDataLog, RawFailedLog, RawInvokeLog, RawLog, RawProgramLog, RawReturnLog,
        RawSuccessLog,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedLog {
    Invoke(ParsedInvokeLog),
    Success(ParsedSuccessLog),
    Failed(ParsedFailedLog),
    Log(ParsedProgramLog),
    Data(ParsedDataLog),
    Return(ParsedReturnLog),
    Cu(ParsedCuLog),
    Other(String),
}

impl ParsedLog {
    pub fn from_raw(raw: &RawLog) -> Result<Self> {
        match raw {
            RawLog::Invoke(log) => ParsedInvokeLog::from_raw(log).map(ParsedLog::from),
            RawLog::Success(log) => ParsedSuccessLog::from_raw(log).map(ParsedLog::from),
            RawLog::Failed(log) => ParsedFailedLog::from_raw(log).map(ParsedLog::from),
            RawLog::Log(log) => ParsedProgramLog::from_raw(log).map(ParsedLog::from),
            RawLog::Data(log) => ParsedDataLog::from_raw(log).map(ParsedLog::from),
            RawLog::Return(log) => ParsedReturnLog::from_raw(log).map(ParsedLog::from),
            RawLog::Cu(log) => ParsedCuLog::from_raw(log).map(ParsedLog::from),
            RawLog::Other(log) => Ok(ParsedLog::Other(log.to_string())),
        }
    }
}

impl From<ParsedInvokeLog> for ParsedLog {
    fn from(value: ParsedInvokeLog) -> Self {
        ParsedLog::Invoke(value)
    }
}

impl From<ParsedSuccessLog> for ParsedLog {
    fn from(value: ParsedSuccessLog) -> Self {
        ParsedLog::Success(value)
    }
}
impl From<ParsedFailedLog> for ParsedLog {
    fn from(value: ParsedFailedLog) -> Self {
        ParsedLog::Failed(value)
    }
}
impl From<ParsedProgramLog> for ParsedLog {
    fn from(value: ParsedProgramLog) -> Self {
        ParsedLog::Log(value)
    }
}
impl From<ParsedDataLog> for ParsedLog {
    fn from(value: ParsedDataLog) -> Self {
        ParsedLog::Data(value)
    }
}
impl From<ParsedReturnLog> for ParsedLog {
    fn from(value: ParsedReturnLog) -> Self {
        ParsedLog::Return(value)
    }
}
impl From<ParsedCuLog> for ParsedLog {
    fn from(value: ParsedCuLog) -> Self {
        ParsedLog::Cu(value)
    }
}

// A Program Invoke Log
///
/// `Program <id> invoke [n]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedInvokeLog {
    pub program_id: Pubkey,
    pub depth: u8,
}

impl ParsedInvokeLog {
    pub fn from_raw(log: &RawInvokeLog) -> Result<Self> {
        Ok(ParsedInvokeLog {
            program_id: Pubkey::from_str(log.program_id)?,
            depth: log.depth,
        })
    }
}

// A Program Success Log
///
/// `Program <id> success`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSuccessLog {
    pub program_id: Pubkey,
}

impl ParsedSuccessLog {
    pub fn from_raw(log: &RawSuccessLog) -> Result<Self> {
        Ok(ParsedSuccessLog {
            program_id: Pubkey::from_str(log.program_id)?,
        })
    }
}

// A Program Failed Log
///
/// `Program <id> failed: <err>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFailedLog {
    pub program_id: Pubkey,
    pub err: String,
}

impl ParsedFailedLog {
    pub fn from_raw(log: &RawFailedLog) -> Result<Self> {
        Ok(ParsedFailedLog {
            program_id: Pubkey::from_str(log.program_id)?,
            err: log.err.to_string(),
        })
    }
}

// A Program Log Log
///
/// `Program log: <msg>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedProgramLog {
    pub msg: String,
}

impl ParsedProgramLog {
    pub fn from_raw(log: &RawProgramLog) -> Result<Self> {
        Ok(ParsedProgramLog {
            msg: log.msg.to_string(),
        })
    }
}

// A Program Data Log
///
/// `Program data: <base64>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedDataLog {
    pub data: Vec<u8>,
}

impl ParsedDataLog {
    pub fn from_raw(log: &RawDataLog) -> Result<Self> {
        Ok(ParsedDataLog {
            data: BASE64_STANDARD.decode(log.data)?,
        })
    }
}

// A Program Return Log
///
/// `Program return: <id> <base64>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedReturnLog {
    pub program_id: Pubkey,
    pub data: Vec<u8>,
}

impl ParsedReturnLog {
    pub fn from_raw(log: &RawReturnLog) -> Result<Self> {
        Ok(ParsedReturnLog {
            program_id: Pubkey::from_str(log.program_id)?,
            data: BASE64_STANDARD.decode(log.data)?,
        })
    }
}

// A Program Compute Unit Log
///
/// `Program <id> consumed <x> of <y> compute units`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCuLog {
    pub program_id: Pubkey,
    pub consumed: u64,
    pub budget: u64,
}

impl ParsedCuLog {
    pub fn from_raw(log: &RawCuLog) -> Result<Self> {
        Ok(ParsedCuLog {
            program_id: Pubkey::from_str(log.program_id)?,
            consumed: log.consumed.parse()?,
            budget: log.budget.parse()?,
        })
    }
}

/* *************************************************************************** *
 * HELPER CODE
 * *************************************************************************** */

mod helper_code {
    use solana_pubkey::Pubkey;

    use crate::structured_log::{FailedLog, InvokeLog, Log, Log2, ReturnLog, SuccessLog};

    use super::{
        ParsedCuLog, ParsedDataLog, ParsedFailedLog, ParsedInvokeLog, ParsedLog, ParsedProgramLog,
        ParsedReturnLog, ParsedSuccessLog,
    };

    type Log2Helper = Log2<
        ParsedInvokeLog,
        ParsedSuccessLog,
        ParsedFailedLog,
        ParsedProgramLog,
        ParsedDataLog,
        ParsedReturnLog,
        ParsedCuLog,
        String,
    >;

    impl Log for ParsedInvokeLog {
        type RawLog = Log2Helper;

        fn raw_log(&self) -> Self::RawLog {
            ParsedLog::Invoke(self.clone()).into()
        }
    }

    impl Log for ParsedSuccessLog {
        type RawLog = Log2Helper;

        fn raw_log(&self) -> Self::RawLog {
            ParsedLog::Success(self.clone()).into()
        }
    }

    impl Log for ParsedFailedLog {
        type RawLog = Log2Helper;

        fn raw_log(&self) -> Self::RawLog {
            ParsedLog::Failed(self.clone()).into()
        }
    }

    impl Log for ParsedProgramLog {
        type RawLog = Log2Helper;

        fn raw_log(&self) -> Self::RawLog {
            ParsedLog::Log(self.clone()).into()
        }
    }

    impl Log for ParsedDataLog {
        type RawLog = Log2Helper;

        fn raw_log(&self) -> Self::RawLog {
            ParsedLog::Data(self.clone()).into()
        }
    }

    impl Log for ParsedReturnLog {
        type RawLog = Log2Helper;

        fn raw_log(&self) -> Self::RawLog {
            ParsedLog::Return(self.clone()).into()
        }
    }

    impl Log for ParsedCuLog {
        type RawLog = Log2Helper;

        fn raw_log(&self) -> Self::RawLog {
            ParsedLog::Cu(self.clone()).into()
        }
    }

    impl InvokeLog for ParsedInvokeLog {
        type ProgramId = Pubkey;

        fn program_id(&self) -> Self::ProgramId {
            self.program_id
        }

        fn depth(&self) -> u8 {
            self.depth
        }
    }

    impl SuccessLog for ParsedSuccessLog {
        type ProgramId = Pubkey;

        fn program_id(&self) -> Self::ProgramId {
            self.program_id
        }
    }

    impl FailedLog for ParsedFailedLog {
        type ProgramId = Pubkey;
        type Err = String;

        fn program_id(&self) -> Self::ProgramId {
            self.program_id
        }

        fn err(&self) -> Self::Err {
            self.err.clone()
        }
    }

    impl ReturnLog for ParsedReturnLog {
        type ProgramId = Pubkey;
        type Data = Vec<u8>;

        fn program_id(&self) -> Self::ProgramId {
            self.program_id
        }

        fn data(&self) -> Self::Data {
            self.data.clone()
        }
    }
}
