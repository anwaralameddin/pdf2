use super::increment::Increment;
use super::startxref::StartXRef;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::Byte;

/// REFERENCE: [7.5.4 Cross-reference table, p55] and [7.5.6 Incremental updates, p60]
#[derive(Debug, PartialEq, Default)]
pub(crate) struct PreTable(Vec<Increment>);

impl Parser<'_> for PreTable {
    fn parse(buffer: &[Byte]) -> ParseResult<(&[Byte], Self)> {
        let (_, startxref) = StartXRef::parse(buffer)?;

        let mut increments = Vec::default();

        let mut prev = Some(*startxref);

        while let Some(offset) = prev {
            let remains = &buffer[offset..];
            let (_, increment) = Increment::parse(remains)?;

            // FIXME This does not take intoaccount the notes on
            // hybrid-reference file’s trailer dictionary in
            // REFERENCE: [7.5.8.4 Compatibility with applications that do not
            // support compressed reference streams, p68]
            prev = increment.trailer().prev();

            // We first read the last section and then read the previous one. We
            // use `push` to preserve the order of the sections, which
            // simplifies iterating over them and merging them so that later
            // sections override earlier ones.
            increments.push(increment);
        }

        Ok((buffer, Self(increments)))
    }
}

mod process {
    use super::*;
    use crate::process::error::NewProcessResult;
    use crate::xref::Table;
    use crate::xref::ToTable;

    impl ToTable for PreTable {
        fn to_table(&self) -> NewProcessResult<Table> {
            self.iter()
                .try_fold(Table::default(), |mut table, increment| {
                    table.extend(increment.to_table()?);
                    Ok(table)
                })
        }
    }
}

mod convert {
    use ::std::ops::Deref;

    use super::*;

    impl From<Vec<Increment>> for PreTable {
        fn from(value: Vec<Increment>) -> Self {
            Self(value)
        }
    }

    impl FromIterator<Increment> for PreTable {
        fn from_iter<I: IntoIterator<Item = Increment>>(iter: I) -> Self {
            Self(Vec::from_iter(iter))
        }
    }

    impl Deref for PreTable {
        type Target = Vec<Increment>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl IntoIterator for PreTable {
        type Item = Increment;
        type IntoIter = <Vec<Increment> as IntoIterator>::IntoIter;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }
}
