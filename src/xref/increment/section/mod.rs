pub(crate) mod entry;
pub(crate) mod subsection;

use ::nom::bytes::complete::tag;
use ::nom::combinator::opt;
use ::nom::sequence::delimited;
use ::nom::sequence::terminated;
use ::nom::Err as NomErr;
use ::std::collections::VecDeque;
use ::std::fmt::Display;
use ::std::fmt::Formatter;
use ::std::fmt::Result as FmtResult;

use self::subsection::Subsection;
use super::trailer::Trailer;
use crate::object::direct::dictionary::Dictionary;
use crate::parse::character_set::eol;
use crate::parse::character_set::white_space;
use crate::parse::character_set::white_space_or_comment;
use crate::parse::error::ParseErr;
use crate::parse::error::ParseErrorCode;
use crate::parse::error::ParseFailure;
use crate::parse::error::ParseRecoverable;
use crate::parse::error::ParseResult;
use crate::parse::Parser;
use crate::parse::KW_TRAILER;
use crate::parse::KW_XREF;
use crate::parse_failure;
use crate::parse_recoverable;
use crate::Byte;

/// REFERENCE: [7.5.4 Cross-reference table, p57]
#[derive(Debug, PartialEq)]
pub(crate) struct Section<'buffer> {
    pub(crate) subsections: VecDeque<Subsection>,
    pub(crate) trailer: Trailer<'buffer>,
}

impl Display for Section<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(f, "{}", KW_XREF)?;
        for subsection in &self.subsections {
            write!(f, "{}", subsection)?;
        }
        write!(f, "{}", self.trailer)
    }
}

impl<'buffer> Parser<'buffer> for Section<'buffer> {
    /// REFERENCE: [7.5.4 Cross-reference table, p56]
    fn parse(buffer: &'buffer [Byte]) -> ParseResult<(&[Byte], Self)> {
        let (mut buffer, _) = terminated(tag(KW_XREF), eol)(buffer).map_err(parse_recoverable!(
            e,
            ParseRecoverable {
                buffer: e.input,
                object: stringify!(Section),
                code: ParseErrorCode::NotFound(e.code)
            }
        ))?;
        // Here, we know that the buffer starts with a cross-reference section,
        // not a cross-reference stream, and the following errors should be
        // propagated as SectionFail
        let mut subsections = VecDeque::new();
        let mut subsection: Subsection;
        while let Some(result) = Subsection::parse_suppress_recoverable::<Subsection>(buffer) {
            // try_parse propagates only Failure errors
            (buffer, subsection) = result?;
            subsections.push_back(subsection);
        }
        // HACK The below addresses the issue with the example PDFs that contain
        // a white space before the trailer keyword that is not accounted for in
        // the standard
        let (buffer, _) = delimited(
            opt(white_space), // No comments are allowed between xref and trailer
            tag(KW_TRAILER),
            opt(white_space_or_comment),
        )(buffer)
        .map_err(parse_failure!(
            e,
            ParseFailure {
                buffer: e.input,
                object: stringify!(Section),
                code: ParseErrorCode::MissingClosing(e.code)
            }
        ))?;
        // REFERENCE: [7.5.5 File trailer, p58-59]
        let (remains, trailer) = Dictionary::parse(buffer).map_err(|err| ParseFailure {
            buffer: err.buffer(),
            object: stringify!(Section),
            code: ParseErrorCode::RecMissingSubobject(stringify!(Trailer), Box::new(err.code())),
        })?;

        let trailer = Trailer::try_from(trailer).map_err(|err| ParseFailure {
            buffer, // TODO (TEMP) Replace with trailer.as_bytes() when implemented
            object: stringify!(Section),
            code: ParseErrorCode::InvalidTrailerDictionary(err),
        })?;
        let buffer = remains;

        let section = Section {
            subsections,
            trailer,
        };

        Ok((buffer, section))
    }
}

mod process {
    use ::std::collections::HashSet;

    use super::entry::Entry;
    use super::Section;
    use crate::process::error::NewProcessResult;
    use crate::xref::increment::error::IncrementError;
    use crate::xref::Table;
    use crate::xref::ToTable;
    use crate::ObjectNumberOrZero;

    impl ToTable for Section<'_> {
        // REFERENCE: [7.5.4 Cross-reference table, p56]
        fn to_table(&self) -> NewProcessResult<Table> {
            let mut object_numbers: HashSet<ObjectNumberOrZero> = Default::default();
            self.subsections.iter().try_fold(
                Table::default(),
                |mut table, subsection| -> NewProcessResult<Table> {
                    for (index, entry) in subsection.entries.iter().enumerate() {
                        // FIXME (TEMP) Refactor the below to avoid the following error when using
                        // ObjectNumberOrZero::from(index): the trait `std::convert::From<usize>` is not
                        // implemented for `ObjectNumberOrZero`
                        let object_number =
                            subsection.first_object_number + index as ObjectNumberOrZero;
                        // The object number should not appear in multiple
                        // subsections within the same section
                        if !object_numbers.insert(object_number) {
                            return Err(IncrementError::DuplicateObjectNumber(object_number).into());
                        }

                        match entry {
                            Entry::Free(next_free_object_number, generation_number) => {
                                table.insert_free(
                                    object_number,
                                    *generation_number,
                                    *next_free_object_number,
                                );
                            }
                            Entry::InUse(offset, generation_number) => {
                                table.insert_in_use(object_number, *generation_number, *offset)?;
                            }
                        }
                    }
                    Ok(table)
                },
            )
        }
    }
}

mod convert {
    use super::*;

    impl<'buffer> Section<'buffer> {
        pub(crate) fn new(
            subsections: impl Into<VecDeque<Subsection>>,
            trailer: Trailer<'buffer>,
        ) -> Self {
            Self {
                subsections: subsections.into(),
                trailer,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use ::nom::error::ErrorKind;

    use super::entry::Entry;
    use super::*;
    use crate::assert_err_eq;
    use crate::object::direct::string::Hexadecimal;
    use crate::object::indirect::reference::Reference;
    use crate::parse_assert_eq;

    #[test]
    fn section_valid() {
        // Synthetic test
        let buffer = b"xref\r\ntrailer<</Size 1 /Root 1 0 R>>";
        let section = Section::new(
            VecDeque::default(),
            Trailer::new(1).set_root(unsafe { Reference::new_unchecked(1, 0) }),
        );
        parse_assert_eq!(buffer, section, "".as_bytes());

        // Synthetic test
        let buffer = b"xref\r\n0 1\r\n0000000000 65535 f\r\ntrailer<</Size 1>>";
        let section = Section::new(
            [Subsection::new(0, [Entry::Free(0, 65535)])],
            Trailer::new(1),
        );
        parse_assert_eq!(buffer, section, "".as_bytes());

        // PDF produced by pdfunite from PDFs produced by Microsoft Word
        let buffer: &[Byte] =
            include_bytes!("../../../../tests/data/F3D45259CBB36D09F04BF0D65BAAD3ED_section.bin");
        let subsection: Section =
            include!("../../../../tests/code/F3D45259CBB36D09F04BF0D65BAAD3ED_section.rs");
        parse_assert_eq!(
            buffer,
            subsection,
            "\r\nstartxref\r\n38912\r\n%%EOF\r\n".as_bytes()
        );

        // TODO Add tests, especially with multiple subsections
    }

    #[test]
    fn section_invalid() {
        // Synthetic tests

        // Incmplte cross-reference section
        let buffer = b"xref\r\n0 1\r\n0000000000 65535 f\r\n";
        let parse_result = Section::parse(buffer);
        let expected_error = ParseFailure {
            buffer: b"",
            object: stringify!(Section),
            code: ParseErrorCode::MissingClosing(ErrorKind::Tag),
        };
        assert_err_eq!(parse_result, expected_error);

        // Missing cross-reference section
        let buffer = b"trailer<</Size 1>>";
        let parse_result = Section::parse(buffer);
        let expected_error = ParseRecoverable {
            buffer: b"trailer<</Size 1>>",
            object: stringify!(Section),
            code: ParseErrorCode::NotFound(ErrorKind::Tag),
        };
        assert_err_eq!(parse_result, expected_error);

        // Missing trailer
        // TOOD Refactor error messages to avoid the repetition below
        let buffer = b"xref\r\n0 1\r\n0000000000 65535 f\r\n<</Size 1>>";
        let parse_result = Section::parse(buffer);
        let expected_error = ParseFailure {
            buffer: b"<</Size 1>>",
            object: stringify!(Section),
            code: ParseErrorCode::MissingClosing(ErrorKind::Tag),
        };
        assert_err_eq!(parse_result, expected_error);
    }
}
