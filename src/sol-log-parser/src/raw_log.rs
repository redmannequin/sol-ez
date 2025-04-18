/// A Raw Log
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawLog<'a> {
    Invoke(RawInvokeLog<'a>),
    Success(RawSuccessLog<'a>),
    Failed(RawFailedLog<'a>),
    Log(RawProgramLog<'a>),
    Data(RawDataLog<'a>),
    Return(RawReturnLog<'a>),
    Cu(RawCuLog<'a>),
    Other(RawOtherLog<'a>),
}

impl<'a> RawLog<'a> {
    pub fn parse(log: &'a str) -> Self {
        let trimmed = log.trim();

        if let Some(rest) = trimmed.strip_prefix("Program log: ") {
            return RawLog::Log(RawProgramLog {
                raw: log,
                msg: rest,
            });
        }

        if let Some(rest) = trimmed.strip_prefix("Program data: ") {
            return RawLog::Data(RawDataLog {
                raw: log,
                data: rest,
            });
        }

        if let Some(rest) = trimmed.strip_prefix("Program return: ") {
            let Some((program_id, data)) = rest.split_once(' ') else {
                return RawLog::Other(RawOtherLog { raw: log });
            };

            return RawLog::Return(RawReturnLog {
                raw: log,
                program_id,
                data,
            });
        }

        if let Some(rest) = trimmed.strip_prefix("Program ") {
            let Some((program_id, suffix)) = rest.split_once(' ') else {
                return RawLog::Other(RawOtherLog { raw: log });
            };

            if let Some(depth) = suffix
                .strip_prefix("invoke [")
                .and_then(|s| s.strip_suffix(']'))
            {
                return depth
                    .parse()
                    .ok()
                    .map(|depth| {
                        RawLog::Invoke(RawInvokeLog {
                            raw: log,
                            program_id,
                            depth,
                        })
                    })
                    .unwrap_or_else(|| RawLog::Other(RawOtherLog { raw: log }));
            }

            if suffix == "success" {
                return RawLog::Success(RawSuccessLog {
                    raw: log,
                    program_id,
                });
            }

            if let Some(err) = suffix.strip_prefix("failed: ") {
                return RawLog::Failed(RawFailedLog {
                    raw: log,
                    program_id,
                    err,
                });
            }

            if let Some((consumed, of_budget)) = suffix
                .strip_prefix("consumed ")
                .and_then(|s| s.split_once(" of "))
            {
                let Some(budget) = of_budget.strip_suffix(" compute units") else {
                    return RawLog::Other(RawOtherLog { raw: log });
                };

                return RawLog::Cu(RawCuLog {
                    raw: log,
                    program_id,
                    consumed,
                    budget,
                });
            }
        }

        RawLog::Other(RawOtherLog { raw: log })
    }
}

/// A Raw Invoke Log
///
/// `Program <id> invoke [n]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawInvokeLog<'a> {
    pub raw: &'a str,
    pub program_id: &'a str,
    pub depth: u8,
}

/// A Raw Success Log
///
/// `Program <id> success`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawSuccessLog<'a> {
    pub raw: &'a str,
    pub program_id: &'a str,
}

/// A Raw Failed Log
///
/// `Program <id> failed: <err>``
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawFailedLog<'a> {
    pub raw: &'a str,
    pub program_id: &'a str,
    pub err: &'a str,
}

/// A Raw Program Log
///
/// `Program log: <msg>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawProgramLog<'a> {
    pub raw: &'a str,
    pub msg: &'a str,
}

/// A Raw Data Log
///
/// `Program data: <base64>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawDataLog<'a> {
    pub raw: &'a str,
    pub data: &'a str,
}

/// A Raw Return Log
///
/// `Program return: <id> <base64>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawReturnLog<'a> {
    pub raw: &'a str,
    pub program_id: &'a str,
    pub data: &'a str,
}

/// A Raw Cu Log
///
/// `Program <id> consumed <x> of <y> compute units`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawCuLog<'a> {
    pub raw: &'a str,
    pub program_id: &'a str,
    pub consumed: &'a str,
    pub budget: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawOtherLog<'a> {
    pub raw: &'a str,
}

/* *************************************************************************** *
 *     HELPER CODE
 * *************************************************************************** */

mod helper_code {
    use crate::structured_log::{FailedLog, InvokeLog, Log, ReturnLog, SuccessLog};

    use super::{
        RawCuLog, RawDataLog, RawFailedLog, RawInvokeLog, RawOtherLog, RawProgramLog, RawReturnLog,
        RawSuccessLog,
    };

    impl<'a> Log for RawInvokeLog<'a> {
        type RawLog = &'a str;

        fn raw_log(&self) -> Self::RawLog {
            self.raw
        }
    }

    impl<'a> Log for RawSuccessLog<'a> {
        type RawLog = &'a str;

        fn raw_log(&self) -> Self::RawLog {
            self.raw
        }
    }

    impl<'a> Log for RawFailedLog<'a> {
        type RawLog = &'a str;

        fn raw_log(&self) -> Self::RawLog {
            self.raw
        }
    }

    impl<'a> Log for RawProgramLog<'a> {
        type RawLog = &'a str;

        fn raw_log(&self) -> Self::RawLog {
            self.raw
        }
    }

    impl<'a> Log for RawDataLog<'a> {
        type RawLog = &'a str;

        fn raw_log(&self) -> Self::RawLog {
            self.raw
        }
    }

    impl<'a> Log for RawReturnLog<'a> {
        type RawLog = &'a str;

        fn raw_log(&self) -> Self::RawLog {
            self.raw
        }
    }

    impl<'a> Log for RawCuLog<'a> {
        type RawLog = &'a str;

        fn raw_log(&self) -> Self::RawLog {
            self.raw
        }
    }

    impl<'a> Log for RawOtherLog<'a> {
        type RawLog = &'a str;

        fn raw_log(&self) -> Self::RawLog {
            self.raw
        }
    }

    impl<'a> InvokeLog for RawInvokeLog<'a> {
        type ProgramId = &'a str;

        fn program_id(&self) -> Self::ProgramId {
            self.program_id
        }

        fn depth(&self) -> u8 {
            self.depth
        }
    }

    impl<'a> SuccessLog for RawSuccessLog<'a> {
        type ProgramId = &'a str;

        fn program_id(&self) -> Self::ProgramId {
            self.program_id
        }
    }

    impl<'a> FailedLog for RawFailedLog<'a> {
        type ProgramId = &'a str;
        type Err = &'a str;

        fn program_id(&self) -> Self::ProgramId {
            self.program_id
        }

        fn err(&self) -> Self::Err {
            self.err
        }
    }

    impl<'a> ReturnLog for RawReturnLog<'a> {
        type ProgramId = &'a str;
        type Data = &'a str;

        fn program_id(&self) -> Self::ProgramId {
            self.program_id
        }

        fn data(&self) -> Self::Data {
            self.data
        }
    }
}
