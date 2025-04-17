use crate::raw_log::{RawCuLog, RawDataLog, RawLog, RawProgramLog};

use super::Log2;

/// A Raw Program Result
pub enum RawProgramResult<'a> {
    Success,
    Err(&'a str),
}

/// A Raw Structured Log
pub struct RawStructuredLog<'a> {
    pub program_id: &'a str,
    pub depth: u8,
    pub result: RawProgramResult<'a>,
    pub program_logs: Vec<RawProgramLog<'a>>,
    pub data_logs: Vec<RawDataLog<'a>>,
    pub return_data: Option<&'a str>,
    pub compute_log: Option<RawCuLog<'a>>,
    pub cpi_logs: Vec<RawStructuredLog<'a>>,
    pub raw_logs: Vec<RawLog<'a>>,
}

impl<'a> RawStructuredLog<'a> {
    pub fn from_raw_logs(logs: Vec<RawLog<'a>>) -> Vec<Self> {
        let log2: Vec<_> = logs.into_iter().map(Log2::from).collect();
        let structured_log = helper_code::RawStructuredLogHelper::from_logs(log2);
        structured_log.into_iter().map(Self::from).collect()
    }
}

/* *************************************************************************** *
 *  HELPER CODE
 * *************************************************************************** */

mod helper_code {
    use crate::{
        raw_log::{
            RawCuLog, RawDataLog, RawFailedLog, RawInvokeLog, RawLog, RawProgramLog, RawReturnLog,
            RawSuccessLog,
        },
        structured_log::{Log2, ProgramResult, StructuredLog},
    };

    use super::{RawProgramResult, RawStructuredLog};

    impl<'a> From<RawStructuredLogHelper<'a>> for RawStructuredLog<'a> {
        fn from(value: RawStructuredLogHelper<'a>) -> Self {
            Self {
                program_id: value.program_id,
                depth: value.depth,
                result: match value.result {
                    ProgramResult::Success => RawProgramResult::Success,
                    ProgramResult::Err(err) => RawProgramResult::Err(err),
                },
                program_logs: value.program_logs,
                data_logs: value.data_logs,
                return_data: value.return_data,
                compute_log: value.compute_log,
                cpi_logs: value.cpi_logs.into_iter().map(Self::from).collect(),
                raw_logs: value
                    .raw_logs
                    .into_iter()
                    .map(|log| match log {
                        Log2::Invoke(log) => RawLog::Invoke(log),
                        Log2::Success(log) => RawLog::Success(log),
                        Log2::Failed(log) => RawLog::Failed(log),
                        Log2::Log(log) => RawLog::Log(log),
                        Log2::Data(log) => RawLog::Data(log),
                        Log2::Return(log) => RawLog::Return(log),
                        Log2::Cu(log) => RawLog::Cu(log),
                        Log2::Other(log) => RawLog::Other(log),
                    })
                    .collect(),
            }
        }
    }

    pub type RawStructuredLogHelper<'a> = StructuredLog<
        &'a str,
        ProgramResult<&'a str>,
        RawProgramLog<'a>,
        RawDataLog<'a>,
        &'a str,
        RawCuLog<'a>,
        Log2<
            RawInvokeLog<'a>,
            RawSuccessLog<'a>,
            RawFailedLog<'a>,
            RawProgramLog<'a>,
            RawDataLog<'a>,
            RawReturnLog<'a>,
            RawCuLog<'a>,
            &'a str,
        >,
    >;
}
