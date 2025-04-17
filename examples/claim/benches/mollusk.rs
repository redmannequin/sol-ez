// Portions of this file are derived from code licensed under the Apache License, Version 2.0.
// See: https://www.apache.org/licenses/LICENSE-2.0
//
// Original source: https://github.com/anza-xyz/mollusk
// Modifications have been made.

use std::path::{Path, PathBuf};

use chrono::Utc;
use num_format::{Locale, ToFormattedString};

pub struct MyBenchResult<'a> {
    pub name: &'a str,
    pub cus_consumed: u64,
    pub root_cus_consumed: Option<u64>,
    pub cpi_cus_consumed: Option<u64>,
}

pub fn write_results(results: Vec<MyBenchResult>) {
    let table_header = Utc::now().to_string();
    let mut out_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    out_dir.push("benches");

    let path = out_dir.join("compute_units.md");

    // Load the existing bench content and parse the most recent table.
    let mut no_changes = true;
    let existing_content = if path.exists() {
        Some(std::fs::read_to_string(&path).unwrap())
    } else {
        None
    };
    let previous = existing_content
        .as_ref()
        .map(|content| parse_last_md_table(content));

    // Prepare to write a new table.
    let mut md_table = md_header(&table_header);

    // Evaluate the results against the previous table, if any.
    // If there are changes, write a new table.
    // If there are no changes, break out and abort gracefully.
    for result in results {
        let delta = match previous.as_ref().and_then(|prev_results| {
            prev_results
                .iter()
                .find(|prev_result| prev_result.name == result.name)
        }) {
            Some(prev) => {
                let delta = result.cus_consumed as i64 - prev.cus_consumed as i64;
                if delta == 0 {
                    "--".to_string()
                } else {
                    no_changes = false;
                    if delta > 0 {
                        format!("+{}", delta.to_formatted_string(&Locale::en))
                    } else {
                        delta.to_formatted_string(&Locale::en)
                    }
                }
            }
            None => {
                no_changes = false;
                "- new -".to_string()
            }
        };
        md_table.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            result.name,
            result.cus_consumed,
            delta,
            result
                .root_cus_consumed
                .map(|cu| cu.to_formatted_string(&Locale::en))
                .unwrap_or("--".to_string()),
            result
                .cpi_cus_consumed
                .map(|cu| cu.to_formatted_string(&Locale::en))
                .unwrap_or("--".to_string()),
        ));
    }

    // Only create a new table if there were changes.
    if !no_changes {
        md_table.push('\n');
        prepend_to_md_file(&path, &md_table);
    }
}

fn md_header(table_header: &str) -> String {
    format!(
        r#"#### {}

| Name | CUs  | Delta | Root Cus | CPIs Cus |
|------|------|-------|----------|----------|
"#,
        table_header
    )
}

fn parse_last_md_table(content: &str) -> Vec<MyBenchResult> {
    let mut results = vec![];

    for line in content.lines().skip(6) {
        if line.starts_with("####") || line.is_empty() {
            break;
        }

        let mut parts = line.split('|').skip(1).map(str::trim);
        let name = parts.next().unwrap();
        let cus_consumed = parts.next().unwrap().parse().unwrap();

        results.push(MyBenchResult {
            name,
            cus_consumed,
            root_cus_consumed: None,
            cpi_cus_consumed: None,
        });
    }

    results
}

fn prepend_to_md_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }

    let contents = if path.exists() {
        std::fs::read_to_string(path).unwrap()
    } else {
        String::new()
    };

    let mut new_contents = content.to_string();
    new_contents.push_str(&contents);

    std::fs::write(path, new_contents).unwrap();
}
