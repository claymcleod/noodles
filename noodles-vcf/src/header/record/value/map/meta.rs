mod builder;
mod tag;

use std::fmt::{self, Display};

use indexmap::IndexMap;

use super::{Fields, Inner, Map, TryFromFieldsError};
use crate::header::Number;

type StandardTag = tag::Standard;
type Tag = super::tag::Tag<StandardTag>;

/// An inner VCF header meta map value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Meta {
    values: Vec<String>,
}

impl Inner for Meta {
    type Id = String;
    type StandardTag = StandardTag;
    type Builder = builder::Builder;
}

impl Map<Meta> {
    /// Creates a VCF header meta map value.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::header::record::value::{map::Meta, Map};
    ///
    /// let map = Map::<Meta>::new(
    ///     "Assay",
    ///     vec![String::from("WholeGenome"), String::from("Exome")],
    /// );
    /// ```
    pub fn new<I>(id: I, values: Vec<String>) -> Self
    where
        I: Into<String>,
    {
        Self {
            id: id.into(),
            inner: Meta { values },
            other_fields: IndexMap::new(),
        }
    }

    /// Returns the meta values.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_vcf::header::record::value::{map::Meta, Map};
    /// let values = vec![String::from("WholeGenome"), String::from("Exome")];
    /// let map = Map::<Meta>::new("Assay", values.clone());
    /// assert_eq!(map.values(), &values);
    /// ```
    pub fn values(&self) -> &[String] {
        &self.inner.values
    }
}

impl Display for Map<Meta> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        super::fmt_display_prefix(f, self.id())?;

        ",Type=String".fmt(f)?;
        write!(f, ",Number={}", Number::Unknown)?;

        ",Values=".fmt(f)?;
        '['.fmt(f)?;

        for (i, value) in self.values().iter().enumerate() {
            if i > 0 {
                ", ".fmt(f)?;
            }

            value.fmt(f)?;
        }

        ']'.fmt(f)?;

        super::fmt_display_other_fields(f, self.other_fields())?;
        super::fmt_display_suffix(f)?;

        Ok(())
    }
}

impl TryFrom<Fields> for Map<Meta> {
    type Error = TryFromFieldsError;

    fn try_from(fields: Fields) -> Result<Self, Self::Error> {
        let mut other_fields = super::init_other_fields(fields.len());

        let mut id = None;
        let mut ty = None;
        let mut number = None;
        let mut values = None;

        for (key, value) in fields {
            match Tag::from(key) {
                Tag::Standard(StandardTag::Id) => super::parse_id(&value, &mut id)?,
                Tag::Standard(StandardTag::Type) => parse_type(value, &mut ty)?,
                Tag::Standard(StandardTag::Number) => super::parse_number(&value, &mut number)?,
                Tag::Standard(StandardTag::Values) => parse_values(&value, &mut values)?,
                Tag::Other(t) => super::insert_other_field(&mut other_fields, t, value)?,
            }
        }

        let id = id.ok_or(TryFromFieldsError::MissingField("ID"))?;
        let _ = ty.ok_or(TryFromFieldsError::MissingField("Type"))?;
        let _ = number.ok_or(TryFromFieldsError::MissingField("Number"))?;
        let values = values.ok_or(TryFromFieldsError::MissingField("Values"))?;

        Ok(Self {
            id,
            inner: Meta { values },
            other_fields,
        })
    }
}

fn parse_type(s: String, ty: &mut Option<String>) -> Result<(), TryFromFieldsError> {
    if ty.replace(s).is_none() {
        Ok(())
    } else {
        Err(TryFromFieldsError::DuplicateTag)
    }
}

fn parse_values(s: &str, values: &mut Option<Vec<String>>) -> Result<(), TryFromFieldsError> {
    let value = s.split(',').map(|t| t.trim().into()).collect();

    if values.replace(value).is_none() {
        Ok(())
    } else {
        Err(TryFromFieldsError::DuplicateTag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt() {
        let map = Map::<Meta>::new(
            "Assay",
            vec![String::from("WholeGenome"), String::from("Exome")],
        );

        let expected = r#"<ID=Assay,Type=String,Number=.,Values=[WholeGenome, Exome]>"#;

        assert_eq!(map.to_string(), expected);
    }

    #[test]
    fn test_try_from_fields_for_map_meta() -> Result<(), TryFromFieldsError> {
        let actual = Map::<Meta>::try_from(vec![
            (String::from("ID"), String::from("Assay")),
            (String::from("Type"), String::from("String")),
            (String::from("Number"), String::from(".")),
            (String::from("Values"), String::from("WholeGenome, Exome")),
        ])?;

        let expected = Map::<Meta>::new(
            "Assay",
            vec![String::from("WholeGenome"), String::from("Exome")],
        );

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn test_try_from_fields_for_map_meta_with_missing_fields() {
        assert_eq!(
            Map::<Meta>::try_from(vec![
                (String::from("Type"), String::from("String")),
                (String::from("Number"), String::from(".")),
                (String::from("Values"), String::from("WholeGenome, Exome")),
            ]),
            Err(TryFromFieldsError::MissingField("ID"))
        );

        assert_eq!(
            Map::<Meta>::try_from(vec![
                (String::from("ID"), String::from("Assay")),
                (String::from("Number"), String::from(".")),
                (String::from("Values"), String::from("WholeGenome, Exome")),
            ]),
            Err(TryFromFieldsError::MissingField("Type"))
        );

        assert_eq!(
            Map::<Meta>::try_from(vec![
                (String::from("ID"), String::from("Assay")),
                (String::from("Type"), String::from("String")),
                (String::from("Values"), String::from("WholeGenome, Exome")),
            ]),
            Err(TryFromFieldsError::MissingField("Number"))
        );

        assert_eq!(
            Map::<Meta>::try_from(vec![
                (String::from("ID"), String::from("Assay")),
                (String::from("Type"), String::from("String")),
                (String::from("Number"), String::from(".")),
            ]),
            Err(TryFromFieldsError::MissingField("Values"))
        );
    }
}
