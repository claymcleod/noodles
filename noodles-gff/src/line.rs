//! GFF lines.

use std::{error, fmt, str::FromStr};

use super::{directive, record, Directive, Record};

/// A GFF line.
#[derive(Clone, Debug, PartialEq)]
pub enum Line {
    /// A directive (`##`).
    Directive(Directive),
    /// A comment (`#`),
    Comment(String),
    /// A record.
    Record(Record),
}

/// An error returns when a raw GFF line fails to parse.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    /// The directive is invalid.
    InvalidDirective(directive::ParseError),
    /// The record is invalid.
    InvalidRecord(record::ParseError),
}

impl error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDirective(e) => write!(f, "{}", e),
            Self::InvalidRecord(e) => write!(f, "{}", e),
        }
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Line::Directive(directive) => write!(f, "{}", directive),
            Line::Comment(comment) => write!(f, "#{}", comment),
            Line::Record(record) => write!(f, "{}", record),
        }
    }
}

impl FromStr for Line {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with(directive::PREFIX) {
            s.parse()
                .map(Self::Directive)
                .map_err(ParseError::InvalidDirective)
        } else if let Some(t) = s.strip_prefix('#') {
            Ok(Self::Comment(t.into()))
        } else {
            s.parse()
                .map(Self::Record)
                .map_err(ParseError::InvalidRecord)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(
            "##gff-version 3".parse(),
            Ok(Line::Directive(Directive::GffVersion(Default::default())))
        );

        assert_eq!(
            "#format: gff3".parse(),
            Ok(Line::Comment(String::from("format: gff3")))
        );

        assert!(matches!(
            "sq0\tNOODLES\tgene\t8\t13\t.\t+\t.\tgene_id=ndls0;gene_name=gene0".parse(),
            Ok(Line::Record(_))
        ));
    }
}
