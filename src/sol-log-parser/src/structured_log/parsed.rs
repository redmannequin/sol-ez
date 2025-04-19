use solana_pubkey::Pubkey;

use crate::parsed_log::{ParsedDataLog, ParsedLog, ParsedProgramLog};

use super::{ComputeUnits, Log2};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedStructuredLog {
    pub program_id: Pubkey,
    pub depth: u8,
    pub result: ParsedProgramResult,
    pub program_logs: Vec<ParsedProgramLog>,
    pub data_logs: Vec<ParsedDataLog>,
    pub return_data: Option<Vec<u8>>,
    pub compute_log: Option<ComputeUnits>,
    pub cpi_logs: Vec<Self>,
    pub raw_logs: Vec<String>,
}

impl ParsedStructuredLog {
    pub fn from_parsed_logs(logs: Vec<ParsedLog>) -> Vec<Self> {
        let log2: Vec<_> = logs.into_iter().map(Log2::from).collect();
        let structured_log = helper_code::ParsedStructuredLogHelper::from_logs(log2);
        structured_log.into_iter().map(Self::from).collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedProgramResult {
    Success,
    Err(String),
}

/* *************************************************************************** *
 *  HELPER CODE
 * *************************************************************************** */

mod helper_code {
    use solana_pubkey::Pubkey;

    use crate::{
        parsed_log::{ParsedDataLog, ParsedProgramLog},
        structured_log::{ProgramResult, StructuredLog},
    };

    use super::{ParsedProgramResult, ParsedStructuredLog};

    impl<'a> From<ParsedStructuredLogHelper> for ParsedStructuredLog {
        fn from(value: ParsedStructuredLogHelper) -> Self {
            Self {
                program_id: value.program_id,
                depth: value.depth,
                result: match value.result {
                    ProgramResult::Success => ParsedProgramResult::Success,
                    ProgramResult::Err(err) => ParsedProgramResult::Err(err),
                },
                program_logs: value.program_logs,
                data_logs: value.data_logs,
                return_data: value.return_data,
                compute_log: value.compute_log,
                cpi_logs: value.cpi_logs.into_iter().map(Self::from).collect(),
                raw_logs: value.raw_logs,
            }
        }
    }

    pub type ParsedStructuredLogHelper = StructuredLog<
        Pubkey,
        ProgramResult<String>,
        ParsedProgramLog,
        ParsedDataLog,
        Vec<u8>,
        String,
    >;
}
