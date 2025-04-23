use std::str::FromStr;

use base64::{prelude::BASE64_STANDARD, Engine};
use solana_pubkey::Pubkey;

use crate::{
    raw_log::{
        RawCuLog, RawDataLog, RawFailedLog, RawInvokeLog, RawLog, RawOtherLog, RawProgramLog,
        RawReturnLog, RawSuccessLog,
    },
    Result,
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
    Other(ParsedOtherLog),
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
            RawLog::Other(log) => ParsedOtherLog::from_raw(log).map(ParsedLog::from),
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

impl From<ParsedOtherLog> for ParsedLog {
    fn from(value: ParsedOtherLog) -> Self {
        ParsedLog::Other(value)
    }
}
// A Program Invoke Log
///
/// `Program <id> invoke [n]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedInvokeLog {
    pub raw: String,
    pub program_id: Pubkey,
    pub depth: u8,
}

impl ParsedInvokeLog {
    pub fn from_raw(log: &RawInvokeLog) -> Result<Self> {
        Ok(ParsedInvokeLog {
            raw: log.raw.to_string(),
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
    pub raw: String,
    pub program_id: Pubkey,
}

impl ParsedSuccessLog {
    pub fn from_raw(log: &RawSuccessLog) -> Result<Self> {
        Ok(ParsedSuccessLog {
            raw: log.raw.to_string(),
            program_id: Pubkey::from_str(log.program_id)?,
        })
    }
}

// A Program Failed Log
///
/// `Program <id> failed: <err>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFailedLog {
    pub raw: String,
    pub program_id: Pubkey,
    pub err: String,
}

impl ParsedFailedLog {
    pub fn from_raw(log: &RawFailedLog) -> Result<Self> {
        Ok(ParsedFailedLog {
            raw: log.raw.to_string(),
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
    pub raw: String,
    pub msg: String,
}

impl ParsedProgramLog {
    pub fn from_raw(log: &RawProgramLog) -> Result<Self> {
        Ok(ParsedProgramLog {
            raw: log.raw.to_string(),
            msg: log.msg.to_string(),
        })
    }
}

// A Program Data Log
///
/// `Program data: <base64>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedDataLog {
    pub raw: String,
    pub data: Vec<u8>,
}

impl ParsedDataLog {
    pub fn from_raw(log: &RawDataLog) -> Result<Self> {
        Ok(ParsedDataLog {
            raw: log.raw.to_string(),
            data: BASE64_STANDARD.decode(log.data)?,
        })
    }
}

// A Program Return Log
///
/// `Program return: <id> <base64>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedReturnLog {
    pub raw: String,
    pub program_id: Pubkey,
    pub data: Vec<u8>,
}

impl ParsedReturnLog {
    pub fn from_raw(log: &RawReturnLog) -> Result<Self> {
        Ok(ParsedReturnLog {
            raw: log.raw.to_string(),
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
    pub raw: String,
    pub program_id: Pubkey,
    pub consumed: u64,
    pub budget: u64,
}

impl ParsedCuLog {
    pub fn from_raw(log: &RawCuLog) -> Result<Self> {
        Ok(ParsedCuLog {
            raw: log.raw.to_string(),
            program_id: Pubkey::from_str(log.program_id)?,
            consumed: log.consumed,
            budget: log.budget,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedOtherLog {
    pub raw: String,
}

impl ParsedOtherLog {
    pub fn from_raw(log: &RawOtherLog) -> Result<Self> {
        Ok(ParsedOtherLog {
            raw: log.raw.to_string(),
        })
    }
}

/* *************************************************************************** *
 * HELPER CODE
 * *************************************************************************** */

mod helper_code {
    use solana_pubkey::Pubkey;

    use crate::structured_log::{
        ComputeUnitsLog, FailedLog, InvokeLog, Log, ReturnLog, SuccessLog,
    };

    use super::{
        ParsedCuLog, ParsedDataLog, ParsedFailedLog, ParsedInvokeLog, ParsedOtherLog,
        ParsedProgramLog, ParsedReturnLog, ParsedSuccessLog,
    };

    impl Log for ParsedInvokeLog {
        type RawLog = String;

        fn raw_log(&self) -> Self::RawLog {
            self.raw.clone()
        }
    }

    impl Log for ParsedSuccessLog {
        type RawLog = String;

        fn raw_log(&self) -> Self::RawLog {
            self.raw.clone()
        }
    }

    impl Log for ParsedFailedLog {
        type RawLog = String;

        fn raw_log(&self) -> Self::RawLog {
            self.raw.clone()
        }
    }

    impl Log for ParsedProgramLog {
        type RawLog = String;

        fn raw_log(&self) -> Self::RawLog {
            self.raw.clone()
        }
    }

    impl Log for ParsedDataLog {
        type RawLog = String;

        fn raw_log(&self) -> Self::RawLog {
            self.raw.clone()
        }
    }

    impl Log for ParsedReturnLog {
        type RawLog = String;

        fn raw_log(&self) -> Self::RawLog {
            self.raw.clone()
        }
    }

    impl Log for ParsedCuLog {
        type RawLog = String;

        fn raw_log(&self) -> Self::RawLog {
            self.raw.clone()
        }
    }

    impl Log for ParsedOtherLog {
        type RawLog = String;

        fn raw_log(&self) -> Self::RawLog {
            self.raw.clone()
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

    impl ComputeUnitsLog for ParsedCuLog {
        type ProgramId = Pubkey;

        fn program_id(&self) -> Self::ProgramId {
            self.program_id
        }
    }
}
