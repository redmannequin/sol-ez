use std::fmt::{Debug, Display};

use crate::{
    parsed_log::{
        ParsedCuLog, ParsedDataLog, ParsedFailedLog, ParsedInvokeLog, ParsedLog, ParsedOtherLog,
        ParsedProgramLog, ParsedReturnLog, ParsedSuccessLog,
    },
    raw_log::{
        RawCuLog, RawDataLog, RawFailedLog, RawInvokeLog, RawLog, RawOtherLog, RawProgramLog,
        RawReturnLog, RawSuccessLog,
    },
};

pub mod parsed;
pub mod raw;

/// A generic structured representation of a program execution log.
///
/// `StructuredLog` provides a hierarchical view of program execution, including:
/// - The program's identifier and execution depth (`program_id`, `depth`)
/// - The outcome of execution (`result`)
/// - Logs emitted directly by the program (`program_logs`, `data_logs`, `return_data`, `compute_log`)
/// - Nested logs from CPI (cross-program invocation) calls (`cpi_logs`)
/// - Raw, unstructured logs that were parsed to build this representation (`raw_logs`)
///
/// This struct is parameterized over the types of each log component, allowing it to be reused
/// in different contexts, such as raw log parsing, typed log rendering, or test scaffolding.
///
/// Type Parameters:
/// - `Id`: The type used to represent a program ID (e.g., `&str`, `Pubkey`)
/// - `ProgramResult`: The type representing the program’s result log
/// - `ProgramLog`: The type representing individual log lines emitted by the program
/// - `DataLog`: The type representing logs carrying custom data
/// - `ReturnData`: The type for return data emitted at the end of program execution
/// - `ComputeLog`: The type for compute unit logs
/// - `RawLog`: The type for raw log entries that this structured log was derived from
struct StructuredLog<Id, ProgramResult, ProgramLog, DataLog, ReturnData, ComputeLog, RawLog> {
    pub program_id: Id,
    pub depth: u8,
    pub result: ProgramResult,
    pub program_logs: Vec<ProgramLog>,
    pub data_logs: Vec<DataLog>,
    pub return_data: Option<ReturnData>,
    pub compute_log: Option<ComputeLog>,
    pub cpi_logs: Vec<Self>,
    pub raw_logs: Vec<RawLog>,
}

impl<Id, Program, Data, ReturnData, Compute, Err, RawLog>
    StructuredLog<Id, ProgramResult<Err>, Program, Data, ReturnData, Compute, RawLog>
where
    Id: Eq + Debug + Display,
    Program: Log<RawLog = RawLog>,
    Data: Log<RawLog = RawLog>,
    Compute: Log<RawLog = RawLog>,
{
    pub fn from_logs<Invoke, Success, Failed, Return, Other>(
        logs: Vec<Log2<Invoke, Success, Failed, Program, Data, Return, Compute, Other>>,
    ) -> Vec<Self>
    where
        Invoke: Log<RawLog = RawLog> + InvokeLog<ProgramId = Id>,
        Success: Log<RawLog = RawLog> + SuccessLog<ProgramId = Id>,
        Failed: Log<RawLog = RawLog> + FailedLog<ProgramId = Id, Err = Err>,
        Return: Log<RawLog = RawLog> + ReturnLog<ProgramId = Id, Data = ReturnData>,
        Other: Log<RawLog = RawLog>,
    {
        let mut stack = Vec::new();
        let mut completed = Vec::new();

        for log in logs {
            match log {
                Log2::Invoke(log) => {
                    stack.push(FrameBuilder::new(
                        log.program_id(),
                        log.depth(),
                        log.raw_log(),
                    ));
                }
                Log2::Success(log) => {
                    let builder = stack
                        .pop()
                        .expect("Unmatched success log without a prior invoke");
                    assert_eq!(
                        builder.program_id,
                        log.program_id(),
                        "Mismatched success: expected {}, got {}",
                        builder.program_id,
                        log.program_id()
                    );
                    let structured = builder.finalize(ProgramResult::Success, log.raw_log());

                    if let Some(parent) = stack.last_mut() {
                        parent.cpi_logs.push(structured);
                    } else {
                        completed.push(structured);
                    }
                }
                Log2::Failed(log) => {
                    let builder = stack
                        .pop()
                        .expect("Unmatched failed log without a prior invoke");
                    assert_eq!(
                        builder.program_id,
                        log.program_id(),
                        "Mismatched failed: expected {}, got {}",
                        builder.program_id,
                        log.program_id()
                    );
                    let structured = builder.finalize(ProgramResult::Err(log.err()), log.raw_log());

                    if let Some(parent) = stack.last_mut() {
                        parent.cpi_logs.push(structured);
                    } else {
                        completed.push(structured);
                    }
                }
                Log2::Log(log) => {
                    if let Some(top) = stack.last_mut() {
                        top.push_program_log(log);
                    }
                }
                Log2::Data(log) => {
                    if let Some(top) = stack.last_mut() {
                        top.push_data_log(log);
                    }
                }
                Log2::Return(log) => {
                    if let Some(top) = stack.last_mut() {
                        top.set_return_data(log.program_id(), log.data(), log.raw_log());
                    }
                }
                Log2::Cu(log) => {
                    if let Some(top) = stack.last_mut() {
                        top.set_compute_log(log);
                    }
                }
                Log2::Other(log) => {
                    if let Some(top) = stack.last_mut() {
                        top.push_raw(log.raw_log());
                    }
                }
            }
        }

        assert!(
            stack.is_empty(),
            "Unbalanced log stack: {} frames left",
            stack.len()
        );
        completed
    }
}

enum ProgramResult<Err> {
    Success,
    Err(Err),
}

pub(crate) enum Log2<Invoke, Success, Failed, Program, Data, Return, Cu, Other> {
    Invoke(Invoke),
    Success(Success),
    Failed(Failed),
    Log(Program),
    Data(Data),
    Return(Return),
    Cu(Cu),
    Other(Other),
}

impl<'a> From<RawLog<'a>>
    for Log2<
        RawInvokeLog<'a>,
        RawSuccessLog<'a>,
        RawFailedLog<'a>,
        RawProgramLog<'a>,
        RawDataLog<'a>,
        RawReturnLog<'a>,
        RawCuLog<'a>,
        RawOtherLog<'a>,
    >
{
    fn from(value: RawLog<'a>) -> Self {
        match value {
            RawLog::Invoke(raw_invoke_log) => Log2::Invoke(raw_invoke_log),
            RawLog::Success(raw_success_log) => Log2::Success(raw_success_log),
            RawLog::Failed(raw_failed_log) => Log2::Failed(raw_failed_log),
            RawLog::Log(raw_program_log) => Log2::Log(raw_program_log),
            RawLog::Data(raw_data_log) => Log2::Data(raw_data_log),
            RawLog::Return(raw_return_log) => Log2::Return(raw_return_log),
            RawLog::Cu(raw_cu_log) => Log2::Cu(raw_cu_log),
            RawLog::Other(raw_other) => Log2::Other(raw_other),
        }
    }
}

impl From<ParsedLog>
    for Log2<
        ParsedInvokeLog,
        ParsedSuccessLog,
        ParsedFailedLog,
        ParsedProgramLog,
        ParsedDataLog,
        ParsedReturnLog,
        ParsedCuLog,
        ParsedOtherLog,
    >
{
    fn from(value: ParsedLog) -> Self {
        match value {
            ParsedLog::Invoke(invoke_log) => Log2::Invoke(invoke_log),
            ParsedLog::Success(success_log) => Log2::Success(success_log),
            ParsedLog::Failed(failed_log) => Log2::Failed(failed_log),
            ParsedLog::Log(program_log) => Log2::Log(program_log),
            ParsedLog::Data(data_log) => Log2::Data(data_log),
            ParsedLog::Return(return_log) => Log2::Return(return_log),
            ParsedLog::Cu(cu_log) => Log2::Cu(cu_log),
            ParsedLog::Other(other) => Log2::Other(other),
        }
    }
}

pub(crate) trait Log {
    type RawLog;

    fn raw_log(&self) -> Self::RawLog;
}

pub(crate) trait InvokeLog {
    type ProgramId;

    fn program_id(&self) -> Self::ProgramId;
    fn depth(&self) -> u8;
}

pub(crate) trait SuccessLog {
    type ProgramId;

    fn program_id(&self) -> Self::ProgramId;
}

pub(crate) trait FailedLog {
    type ProgramId;
    type Err;

    fn program_id(&self) -> Self::ProgramId;
    fn err(&self) -> Self::Err;
}

pub(crate) trait ReturnLog {
    type ProgramId;
    type Data;

    fn program_id(&self) -> Self::ProgramId;
    fn data(&self) -> Self::Data;
}

struct FrameBuilder<Id, ProgramResult, ProgramLog, DataLog, ReturnData, ComputeLog, RawLog> {
    program_id: Id,
    depth: u8,
    program_logs: Vec<ProgramLog>,
    data_logs: Vec<DataLog>,
    return_data: Option<ReturnData>,
    compute_log: Option<ComputeLog>,
    raw_logs: Vec<RawLog>,
    cpi_logs:
        Vec<StructuredLog<Id, ProgramResult, ProgramLog, DataLog, ReturnData, ComputeLog, RawLog>>,
}

impl<Id, ProgramResult, ProgramLog, DataLog, ReturnData, ComputeLog, RawLog>
    FrameBuilder<Id, ProgramResult, ProgramLog, DataLog, ReturnData, ComputeLog, RawLog>
where
    Id: Eq + Debug + Display,
    ProgramLog: Log<RawLog = RawLog>,
    DataLog: Log<RawLog = RawLog>,
    ComputeLog: Log<RawLog = RawLog>,
{
    fn new(program_id: Id, depth: u8, raw: RawLog) -> Self {
        Self {
            program_id: program_id,
            depth: depth,
            program_logs: vec![],
            data_logs: vec![],
            return_data: None,
            compute_log: None,
            raw_logs: vec![raw],
            cpi_logs: vec![],
        }
    }

    fn push_program_log(&mut self, log: ProgramLog) {
        self.raw_logs.push(log.raw_log());
        self.program_logs.push(log);
    }

    fn push_data_log(&mut self, log: DataLog) {
        self.raw_logs.push(log.raw_log());
        self.data_logs.push(log);
    }

    fn push_raw(&mut self, raw: RawLog) {
        self.raw_logs.push(raw);
    }

    fn set_return_data(&mut self, program_id: Id, data: ReturnData, raw: RawLog) {
        assert_eq!(
            self.program_id, program_id,
            "Return program ID mismatch: expected {}, got {}",
            self.program_id, program_id
        );
        self.raw_logs.push(raw);
        self.return_data = Some(data);
    }

    fn set_compute_log(&mut self, log: ComputeLog) {
        self.raw_logs.push(log.raw_log());
        self.compute_log = Some(log);
    }

    fn finalize(
        mut self,
        result: ProgramResult,
        final_raw: RawLog,
    ) -> StructuredLog<Id, ProgramResult, ProgramLog, DataLog, ReturnData, ComputeLog, RawLog> {
        self.raw_logs.push(final_raw);
        self.raw_logs.shrink_to_fit();
        self.cpi_logs.shrink_to_fit();
        self.data_logs.shrink_to_fit();
        self.program_logs.shrink_to_fit();

        StructuredLog {
            program_id: self.program_id,
            depth: self.depth,
            result,
            program_logs: self.program_logs,
            data_logs: self.data_logs,
            return_data: self.return_data,
            compute_log: self.compute_log,
            cpi_logs: self.cpi_logs,
            raw_logs: self.raw_logs,
        }
    }
}
