#![forbid(unsafe_code)]

//! The CSV content contract — a technology of `xmip-core-contract`.
//!
//! ADR-0010: a contract is the content axis, not a transport. This implements the
//! capability's [`Contract`] trait for CSV: every non-empty row must have the
//! same number of fields as the first, honouring `"..."` quoting with `""` as an
//! escaped quote. A quoted field with an embedded newline spans lines and is
//! beyond this first cut; the validator says so rather than passing it silently.

use contract::{
    Contract, ContractDescriptor, ContractError, ContractId, ValidationIssue, ValidationResult,
};
use stream::Stream;

/// The CSV contract.
pub struct Csv {
    descriptor: ContractDescriptor,
}

impl Csv {
    #[must_use]
    pub fn new() -> Self {
        Self {
            descriptor: ContractDescriptor {
                id: ContractId("csv".to_string()),
                version: "1".to_string(),
                representation: "text/csv".to_string(),
            },
        }
    }
}

impl Default for Csv {
    fn default() -> Self {
        Self::new()
    }
}

impl Contract for Csv {
    fn descriptor(&self) -> &ContractDescriptor {
        &self.descriptor
    }

    fn identify(&self, stream: &Stream) -> Result<bool, ContractError> {
        let text = text(stream)?;
        Ok(text
            .lines()
            .next()
            .is_some_and(|header| header.contains(',')))
    }

    fn validate(&self, stream: &Stream) -> Result<ValidationResult, ContractError> {
        let text = text(stream)?;
        let mut issues = Vec::new();
        let mut expected: Option<usize> = None;

        for (offset, line) in text.lines().enumerate() {
            if line.is_empty() {
                continue;
            }
            match fields(line) {
                Ok(count) => match expected {
                    None => expected = Some(count),
                    Some(want) if count != want => issues.push(ValidationIssue::at(
                        "field-count",
                        &format!("row has {count} fields, the header has {want}"),
                        &format!("line {}", offset + 1),
                    )),
                    Some(_) => {}
                },
                Err(reason) => issues.push(ValidationIssue::at(
                    "malformed",
                    &reason,
                    &format!("line {}", offset + 1),
                )),
            }
        }

        Ok(ValidationResult::of(issues))
    }
}

fn text(stream: &Stream) -> Result<&str, ContractError> {
    std::str::from_utf8(stream.bytes()).map_err(|error| ContractError {
        message: format!("not UTF-8 text: {error}"),
    })
}

/// Count the fields in one row, honouring `"..."` quoting with `""` escapes.
/// Errors on a quote that never closes.
fn fields(line: &str) -> Result<usize, String> {
    let mut count = 1;
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(character) = chars.next() {
        match character {
            '"' if in_quotes && chars.peek() == Some(&'"') => {
                chars.next();
            }
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => count += 1,
            _ => {}
        }
    }
    if in_quotes {
        Err("a quoted field is not closed (an embedded newline is not supported yet)".to_string())
    } else {
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contract::fixture::stream_as;

    fn stream(text: &str) -> Stream {
        stream_as(text, Some("text/csv"))
    }

    #[test]
    fn consistent_rows_are_valid() {
        let held = Csv::new()
            .validate(&stream("a,b,c\n1,2,3\n4,5,6"))
            .expect("validates");
        assert!(held.valid, "issues: {:?}", held.issues);
    }

    #[test]
    fn a_short_row_is_an_issue() {
        let held = Csv::new()
            .validate(&stream("a,b,c\n1,2"))
            .expect("validates");
        assert!(!held.valid);
        assert_eq!(held.issues[0].code, "field-count");
    }

    #[test]
    fn quoted_commas_do_not_count_as_fields() {
        let held = Csv::new()
            .validate(&stream("a,b\n\"x,y\",z"))
            .expect("validates");
        assert!(held.valid, "issues: {:?}", held.issues);
    }

    #[test]
    fn an_unclosed_quote_is_malformed() {
        let held = Csv::new()
            .validate(&stream("a,b\n\"oops,z"))
            .expect("validates");
        assert_eq!(held.issues[0].code, "malformed");
    }

    #[test]
    fn a_comma_header_identifies_as_csv() {
        assert!(Csv::new().identify(&stream("a,b,c")).expect("identifies"));
    }
}
