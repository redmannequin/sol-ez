use pretty_assertions::assert_eq;
use sol_log_parser::{
    parsed_log::{
        ParsedCuLog, ParsedDataLog, ParsedFailedLog, ParsedInvokeLog, ParsedProgramLog,
        ParsedSuccessLog,
    },
    raw_log::{RawCuLog, RawDataLog, RawFailedLog, RawInvokeLog, RawProgramLog, RawSuccessLog},
    structured_log::parsed::ParsedProgramResult,
    LogParseError, ParsedLog, ParsedStructuredLog, RawLog,
};
use solana_pubkey::Pubkey;

#[test]
pub fn invoke_log() {
    let log = "Program 11111111111111111111111111111111 invoke [1]";

    let raw_log = RawLog::parse(log);
    assert_eq!(
        raw_log,
        RawLog::Invoke(RawInvokeLog {
            raw: "Program 11111111111111111111111111111111 invoke [1]",
            program_id: "11111111111111111111111111111111",
            depth: 1
        })
    );

    let parsed_log = ParsedLog::from_raw(&raw_log).expect("failed to parse log");
    assert_eq!(
        parsed_log,
        ParsedLog::Invoke(ParsedInvokeLog {
            raw: "Program 11111111111111111111111111111111 invoke [1]".into(),
            program_id: Pubkey::from_str_const("11111111111111111111111111111111"),
            depth: 1
        })
    )
}

#[test]
pub fn success_log() {
    let log = "Program 11111111111111111111111111111111 success";

    let raw_log = RawLog::parse(log);
    assert_eq!(
        raw_log,
        RawLog::Success(RawSuccessLog {
            raw: "Program 11111111111111111111111111111111 success",
            program_id: "11111111111111111111111111111111",
        })
    );

    let parsed_log = ParsedLog::from_raw(&raw_log).expect("failed to parse log");
    assert_eq!(
        parsed_log,
        ParsedLog::Success(ParsedSuccessLog {
            raw: "Program 11111111111111111111111111111111 success".into(),
            program_id: Pubkey::from_str_const("11111111111111111111111111111111"),
        })
    )
}

#[test]
pub fn failed_log() {
    let log = "Program 11111111111111111111111111111111 failed: insufficient funds";

    let raw_log = RawLog::parse(log);
    assert_eq!(
        raw_log,
        RawLog::Failed(RawFailedLog {
            raw: "Program 11111111111111111111111111111111 failed: insufficient funds",
            program_id: "11111111111111111111111111111111",
            err: "insufficient funds"
        })
    );

    let parsed_log = ParsedLog::from_raw(&raw_log).expect("failed to parse log");
    assert_eq!(
        parsed_log,
        ParsedLog::Failed(ParsedFailedLog {
            raw: "Program 11111111111111111111111111111111 failed: insufficient funds".into(),
            program_id: Pubkey::from_str_const("111111111111111111111111111111111"),
            err: String::from("insufficient funds")
        })
    )
}

#[test]
pub fn program_log() {
    let log = "Program log: Hello from inside the program";

    let raw_log = RawLog::parse(log);
    assert_eq!(
        raw_log,
        RawLog::Log(RawProgramLog {
            raw: "Program log: Hello from inside the program",
            msg: "Hello from inside the program",
        })
    );

    let parsed_log = ParsedLog::from_raw(&raw_log).expect("failed to parse log");
    assert_eq!(
        parsed_log,
        ParsedLog::Log(ParsedProgramLog {
            raw: "Program log: Hello from inside the program".into(),
            msg: String::from("Hello from inside the program")
        })
    )
}

#[test]
pub fn data_log() {
    let log = "Program data: aGVsbG8gc29sYW5h";

    let raw_log = RawLog::parse(log);
    assert_eq!(
        raw_log,
        RawLog::Data(RawDataLog {
            raw: "Program data: aGVsbG8gc29sYW5h",
            data: "aGVsbG8gc29sYW5h",
        })
    );

    let parsed_log = ParsedLog::from_raw(&raw_log).expect("failed to parse log");
    assert_eq!(
        parsed_log,
        ParsedLog::Data(ParsedDataLog {
            raw: "Program data: aGVsbG8gc29sYW5h".into(),
            data: b"hello solana".to_vec()
        })
    )
}

#[test]
pub fn cu_log() {
    let log = "Program 11111111111111111111111111111111 consumed 1820 of 200000 compute units";

    let raw_log = RawLog::parse(log);
    assert_eq!(
        raw_log,
        RawLog::Cu(RawCuLog {
            raw: "Program 11111111111111111111111111111111 consumed 1820 of 200000 compute units",
            program_id: "11111111111111111111111111111111",
            consumed: 1820,
            budget: 200000
        })
    );

    let parsed_log = ParsedLog::from_raw(&raw_log).expect("failed to parse log");
    assert_eq!(
        parsed_log,
        ParsedLog::Cu(ParsedCuLog {
            raw: "Program 11111111111111111111111111111111 consumed 1820 of 200000 compute units"
                .into(),
            program_id: Pubkey::from_str_const("111111111111111111111111111111111"),
            consumed: 1820,
            budget: 200000
        })
    )
}

#[test]
fn structured_log() {
    let logs = [
        "Program D4SghRBTyA7HQSEH89uT9LgCs1TTtrPptwuqm1sLSsns invoke [1]",
        "Program 11111111111111111111111111111111 invoke [2]",
        "Program log: Instruction: CreateAccount",
        "Program 11111111111111111111111111111111 consumed 4731 of 1396590 compute units",
        "Program 11111111111111111111111111111111 success",
        "Program D4SghRBTyA7HQSEH89uT9LgCs1TTtrPptwuqm1sLSsns consumed 8388 of 1400000 compute units",
        "Program D4SghRBTyA7HQSEH89uT9LgCs1TTtrPptwuqm1sLSsns success",
        "Program D4SghRBTyA7HQSEH89uT9LgCs1TTtrPptwuqm1sLSsns invoke [1]",
        "Program D4SghRBTyA7HQSEH89uT9LgCs1TTtrPptwuqm1sLSsns consumed 528 of 1400000 compute units",
        "Program D4SghRBTyA7HQSEH89uT9LgCs1TTtrPptwuqm1sLSsns success",
        "Program D4SghRBTyA7HQSEH89uT9LgCs1TTtrPptwuqm1sLSsns invoke [1]",
        "Program log: testing",
        "Program D4SghRBTyA7HQSEH89uT9LgCs1TTtrPptwuqm1sLSsns consumed 949 of 1400000 compute units",
        "Program D4SghRBTyA7HQSEH89uT9LgCs1TTtrPptwuqm1sLSsns success",
    ];

    let parsed_logs = logs
        .into_iter()
        .map(RawLog::parse)
        .map(|raw| ParsedLog::from_raw(&raw))
        .collect::<Result<Vec<_>, LogParseError>>()
        .expect("Failed to parsed logs");

    let parsed_structured_logs = ParsedStructuredLog::from_parsed_logs(parsed_logs);
    assert_eq!(parsed_structured_logs.len(), 3);

    assert_eq!(
        parsed_structured_logs[0],
        ParsedStructuredLog {
            program_id: Pubkey::from_str_const("D4SghRBTyA7HQSEH89uT9LgCs1TTtrPptwuqm1sLSsns"),
            depth: 1,
            result: ParsedProgramResult::Success,
            program_logs: vec![],
            data_logs: vec![],
            return_data: None,
            compute_log: Some(ParsedCuLog {
                raw: "Program D4SghRBTyA7HQSEH89uT9LgCs1TTtrPptwuqm1sLSsns consumed 8388 of 1400000 compute units".into(),
                program_id: Pubkey::from_str_const("D4SghRBTyA7HQSEH89uT9LgCs1TTtrPptwuqm1sLSsns"),
                consumed: 8388,
                budget: 1400000
            }),
            cpi_logs: vec![ParsedStructuredLog {
                program_id: Pubkey::from_str_const("11111111111111111111111111111111"),
                depth: 2,
                result: ParsedProgramResult::Success,
                program_logs: vec![ParsedProgramLog {
                    raw: "Program log: Instruction: CreateAccount".into(),
                    msg: String::from("Instruction: CreateAccount")
                }],
                data_logs: vec![],
                return_data: None,
                compute_log: Some(ParsedCuLog {
                    raw: "Program 11111111111111111111111111111111 consumed 4731 of 1396590 compute units".into(),
                    program_id: Pubkey::from_str_const("11111111111111111111111111111111"),
                    consumed: 4731,
                    budget: 1396590
                }),
                cpi_logs: vec![],
                raw_logs: vec![
                    "Program 11111111111111111111111111111111 invoke [2]".into(),
                    "Program log: Instruction: CreateAccount".into(),
                    "Program 11111111111111111111111111111111 consumed 4731 of 1396590 compute units".into(),
                    "Program 11111111111111111111111111111111 success".into(),
                ]
            }],
            raw_logs: vec![
                "Program D4SghRBTyA7HQSEH89uT9LgCs1TTtrPptwuqm1sLSsns invoke [1]".into(),
                "Program D4SghRBTyA7HQSEH89uT9LgCs1TTtrPptwuqm1sLSsns consumed 8388 of 1400000 compute units".into(),
                "Program D4SghRBTyA7HQSEH89uT9LgCs1TTtrPptwuqm1sLSsns success".into(),
            ]
        }
    )
}
