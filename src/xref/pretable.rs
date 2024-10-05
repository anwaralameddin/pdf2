use ::std::collections::VecDeque;

use super::increment::Increment;
use super::startxref::StartXRef;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseFailure;
use crate::parse::error::ParseResult;
use crate::parse::ObjectParser;
use crate::parse::Parser;
use crate::parse::Span;
use crate::Byte;

/// REFERENCE: [7.5.4 Cross-reference table, p55] and [7.5.6 Incremental updates, p60]
#[derive(Debug, PartialEq, Default)]
pub(crate) struct PreTable<'buffer>(VecDeque<Increment<'buffer>>);

impl<'buffer> Parser<'buffer> for PreTable<'buffer> {
    fn parse(buffer: &'buffer [Byte]) -> ParseResult<Self> {
        let startxref = <StartXRef as Parser>::parse(buffer)?;

        let mut increments = VecDeque::default();

        let mut prev = Some(*startxref);

        while let Some(offset) = prev {
            let increment = Increment::parse(buffer, offset)?;

            prev = increment.prev().map_err(|err| {
                ParseFailure::new(
                    &buffer[err.dictionary_span],
                    stringify!(PreTable),
                    ParseErrorCode::Object(err),
                )
            })?;

            // dictionary in
            // REFERENCE: [7.5.8.4 Compatibility with applications that do not
            // support compressed reference streams, p68]
            if let Some(xref_stm) = increment.xref_stm().map_err(|err| {
                ParseFailure::new(
                    &buffer[err.dictionary_span],
                    stringify!(PreTable),
                    ParseErrorCode::Object(err),
                )
            })? {
                prev = Some(xref_stm);
            }

            // We first read the last section and then read the previous one. We
            // use `push_front` to preserve the order of the sections, which
            // simplifies iterating over them and merging them so that later
            // sections override earlier ones.
            increments.push_front(increment);
        }

        Ok(Self(increments))
    }

    fn spans(&self) -> Vec<Span> {
        self.iter().map(Increment::span).collect()
    }
}

mod table {
    use super::*;
    use crate::xref::error::XRefResult;
    use crate::xref::Table;
    use crate::xref::ToTable;

    impl ToTable for PreTable<'_> {
        fn to_table(&self) -> XRefResult<Table> {
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
    use ::std::ops::DerefMut;

    use super::*;

    impl<'buffer> From<VecDeque<Increment<'buffer>>> for PreTable<'buffer> {
        fn from(value: VecDeque<Increment<'buffer>>) -> Self {
            Self(value)
        }
    }

    impl<'buffer> FromIterator<Increment<'buffer>> for PreTable<'buffer> {
        fn from_iter<I: IntoIterator<Item = Increment<'buffer>>>(iter: I) -> Self {
            Self(VecDeque::from_iter(iter))
        }
    }

    impl<'buffer> Deref for PreTable<'buffer> {
        type Target = VecDeque<Increment<'buffer>>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<'buffer> DerefMut for PreTable<'buffer> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl<'buffer> IntoIterator for PreTable<'buffer> {
        type Item = Increment<'buffer>;
        type IntoIter = <VecDeque<Increment<'buffer>> as IntoIterator>::IntoIter;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }
}
