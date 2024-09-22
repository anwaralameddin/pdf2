use ::std::collections::VecDeque;
use error::PreTableCode;

use super::increment::Increment;
use super::startxref::StartXRef;
use crate::parse::error::NewParseFailure;
use crate::parse::error::NewParseResult;
use crate::parse::NewParser;
use crate::Byte;

/// REFERENCE: [7.5.4 Cross-reference table, p55] and [7.5.6 Incremental updates, p60]
#[derive(Debug, PartialEq, Default)]
pub(crate) struct PreTable(VecDeque<Increment>);

impl NewParser<'_> for PreTable {
    fn parse(buffer: &[Byte]) -> NewParseResult<(&[Byte], Self)> {
        let (_, startxref) = StartXRef::parse(buffer)?;

        let mut increments = VecDeque::new();

        let mut prev = Some(*startxref);

        while let Some(offset) = prev {
            let start = usize::try_from(offset).map_err(|err| NewParseFailure {
                buffer: &[],
                code: PreTableCode::U64ToOffSet(offset, err).into(),
            })?;
            let buffer_ref = &buffer[start..];
            let (_, increment) = Increment::parse(buffer_ref)?;

            // FIXME This does not take intoaccount the notes on
            // hybrid-reference file’s trailer dictionary in
            // REFERENCE: [7.5.8.4 Compatibility with applications that do not
            // support compressed reference streams, p68]
            prev = increment.trailer().prev();

            // We first read the last section and then read the previous one. We
            // use `push_front` to preserve the order of the sections, which
            // simplifies iterating over them and merging them so that later
            // sections override earlier ones.
            increments.push_front(increment);
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

    impl From<VecDeque<Increment>> for PreTable {
        fn from(value: VecDeque<Increment>) -> Self {
            Self(value)
        }
    }

    impl FromIterator<Increment> for PreTable {
        fn from_iter<I: IntoIterator<Item = Increment>>(iter: I) -> Self {
            Self(VecDeque::from_iter(iter))
        }
    }

    impl Deref for PreTable {
        type Target = VecDeque<Increment>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl IntoIterator for PreTable {
        type Item = Increment;
        type IntoIter = <VecDeque<Increment> as IntoIterator>::IntoIter;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }
}

pub(crate) mod error {

    use ::std::num::TryFromIntError;
    use ::thiserror::Error;

    #[derive(Debug, Error, PartialEq, Clone, Copy)]
    pub enum PreTableCode {
        #[error("Offset. Code: {1}. Input: {0}")]
        U64ToOffSet(u64, TryFromIntError),
    }
}
