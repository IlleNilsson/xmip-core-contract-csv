#![forbid(unsafe_code)]

//! The CSV content contract — a technology of `xmip-core-contract`.
//!
//! ADR-0010: a contract is the content axis, not a transport. This implements the
//! capability's [`Contract`] trait for CSV: every non-empty record must have the
//! same number of fields as the first. The records are the Foundation's walk,
//! `message::record`, which the CSV shape sections by too: RFC 4180 quoting,
//! `""` for a quote, and a quoted field may carry a line break.
//!
//! This walked lines on its own until 2026-09-23, and refused a quoted line
//! break the shape accepted, and took a quote anywhere in a field as opening
//! one (open-problems.md, problem 25, row d).

use contract::{
    Contract, ContractDescriptor, ContractError, ContractId, ValidationIssue, ValidationResult,
};
use message::record::{self, Delimited};
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
        text(stream)?;
        let bytes = stream.bytes();
        Ok(record::lines(bytes)
            .first()
            .is_some_and(|header| bytes[header.clone()].contains(&b',')))
    }

    fn validate(&self, stream: &Stream) -> Result<ValidationResult, ContractError> {
        text(stream)?;
        let bytes = stream.bytes();
        let mut issues = Vec::new();
        let mut expected: Option<usize> = None;

        for walked in Delimited::default().records(bytes) {
            let found = match walked {
                Ok(found) => found,
                // The walk cannot say where the next record starts once one
                // cannot be cut, so the first malformed record is the last
                // one read.
                Err((reason, at)) => {
                    issues.push(ValidationIssue::at(
                        "malformed",
                        reason,
                        &line_of(bytes, at),
                    ));
                    break;
                }
            };
            if found.range.is_empty() {
                continue;
            }
            let count = found.fields.len();
            match expected {
                None => expected = Some(count),
                Some(want) if count != want => issues.push(ValidationIssue::at(
                    "field-count",
                    &format!("row has {count} fields, the header has {want}"),
                    &line_of(bytes, found.range.start),
                )),
                Some(_) => {}
            }
        }

        Ok(ValidationResult::of(issues))
    }
}

/// CSV is text; bytes that are not are an error, not an issue.
fn text(stream: &Stream) -> Result<(), ContractError> {
    record::text(stream.bytes()).map_err(|(reason, at)| ContractError {
        message: format!("not text: {reason} at byte {at}"),
    })
}

/// Where byte `at` is, as the line it starts on: a record that carries a
/// line break is found by the line it opens on.
fn line_of(bytes: &[u8], at: usize) -> String {
    // The pieces a split at every line feed makes are the lines so far.
    let line = bytes[..at.min(bytes.len())]
        .split(|byte| *byte == b'\n')
        .count();
    format!("line {line}")
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

    #[test]
    fn a_quoted_line_break_is_one_field_and_rows_are_placed_by_their_first_line() {
        // Row d of problem 25: this contract refused a quoted line break the
        // CSV shape accepted. One walk now, and the record after a two-line
        // record is found on the line it really starts on.
        let held = Csv::new()
            .validate(&stream("a,b\n\"two\nlines\",z\n1"))
            .expect("validates");
        assert_eq!(held.issues.len(), 1, "issues: {:?}", held.issues);
        assert_eq!(held.issues[0].code, "field-count");
        assert_eq!(held.issues[0].path.as_deref(), Some("line 4"));
    }

    #[test]
    fn a_quote_inside_a_field_is_content_and_after_a_closing_one_is_not() {
        // RFC 4180: a quote opens a field only where the field starts. This
        // contract took one anywhere as opening, and called `x"y` unclosed.
        let held = Csv::new()
            .validate(&stream("a,b\nx\"y,z"))
            .expect("validates");
        assert!(held.valid, "issues: {:?}", held.issues);

        let held = Csv::new()
            .validate(&stream("a,b\n\"x\"y,z"))
            .expect("validates");
        assert_eq!(held.issues[0].code, "malformed");
    }
}
