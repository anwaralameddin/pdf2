use ::nom::branch::alt;
use ::nom::bytes::complete::tag;
use ::nom::bytes::complete::take_till;
use ::nom::bytes::complete::take_while;
use ::nom::bytes::complete::take_while1;
use ::nom::character::complete::char;
use ::nom::combinator::opt;
use ::nom::combinator::recognize;
use ::nom::multi::many1;
use ::nom::sequence::delimited;
use ::nom::sequence::preceded;
use ::nom::IResult;

use crate::fmt::debug_bytes;
use crate::Byte;

/// REFERENCE: [3.68 white-space character, p14] and ["Table 1 — White-space
/// characters" in 7.2.3, "Character set", p22]
pub(crate) const fn is_white_space(byte: Byte) -> bool {
    byte == b'\x09' // HORIZONTAL TABULATION
        || byte == b'\x0A' // LINE FEED
        || byte == b'\x0C' // FORM FEED
        || byte == b'\x0D' // CARRIAGE RETURN
        || byte == b'\x20' // SPACE
        || byte == b'\x00' // NULL
}

/// REFERENCE: [7.2.3 Character set, p22]
pub(crate) fn white_space(buffer: &[Byte]) -> IResult<&[Byte], &[Byte]> {
    take_while1(is_white_space)(buffer)
}

/// REFERENCE: [7.2.4 Comments, p23]
pub(crate) fn comment(buffer: &[Byte]) -> IResult<&[Byte], &[Byte]> {
    let (buffer, comment) =
        preceded(char('%'), take_till(|byte| byte == b'\n' || byte == b'\r'))(buffer)?;

    // TODO Warn/store comments as they are included in the array, dictionary,
    // indirect object, indirect reference or hexadecimal string structures.
    // Print only if verbose mode is enabled.
    eprintln!("Comment: {}", debug_bytes(comment));
    Ok((buffer, comment))
}

/// REFERENCE: [7.2.4 Comments, p23]
pub(crate) fn white_space_or_comment(buffer: &[Byte]) -> IResult<&[Byte], &[Byte]> {
    // A comment is treated as a single white-space character.
    recognize(many1(alt((white_space, comment))))(buffer)
}

/// REFERENCE: [7.2.3 Character set, p22]
pub(crate) fn eol(buffer: &[Byte]) -> IResult<&[Byte], &[Byte]> {
    // HACK Some example PDFs contain a white space before or after the EOL
    // marker that is not accounted for in the standard
    delimited(
        take_while(|byte| byte == b'\x09' || byte == b'\x0C' || byte == b'\x20' || byte == b'\x00'),
        alt((tag(b"\r\n"), tag(b"\n"), tag(b"\r"))),
        // Allowing with_space_or_comment here prevents the parser from
        // detecting the EOF marker
        opt(white_space),
    )(buffer)
}

/// REFERENCE: [7.2.3 Character set, p22]
const fn is_delimiter(byte: Byte) -> bool {
    byte == b'('
        || byte == b')'
        || byte == b'<'
        || byte == b'>'
        || byte == b'['
        || byte == b']'
        || byte == b'{'
        || byte == b'}'
        || byte == b'/'
        || byte == b'%'
}

pub(crate) fn delimiter(buffer: &[Byte]) -> IResult<&[Byte], &[Byte]> {
    take_while1(is_delimiter)(buffer)
}

/// REFERENCE: [7.2.3 Character set, p23] indicates that regular characters are
/// not restricted to the ASCII range.
const fn is_regular(byte: Byte) -> bool {
    !is_white_space(byte) && !is_delimiter(byte)
}

const fn is_printable_regular(byte: Byte) -> bool {
    is_regular(byte) && byte.is_ascii_graphic()
}

pub(crate) fn printable_token(buffer: &[Byte]) -> IResult<&[Byte], &[Byte]> {
    take_while1(is_printable_regular)(buffer)
}

#[cfg(test)]
mod tests {
    use ::nom::error::Error as NomError;
    use ::nom::error::ErrorKind;
    use ::nom::Err as NomErr;

    use super::*;

    #[test]
    fn white_space_valid() {
        assert_eq!(
            white_space(b" .").unwrap(),
            (b".".as_slice(), b" ".as_slice())
        );
        assert_eq!(
            white_space(b"\t.").unwrap(),
            (b".".as_slice(), b"\t".as_slice())
        );
        assert_eq!(
            white_space(b"\n.").unwrap(),
            (b".".as_slice(), b"\n".as_slice())
        );
        assert_eq!(
            white_space(b"\x0C.").unwrap(),
            (b".".as_slice(), b"\x0C".as_slice())
        );
        assert_eq!(
            white_space(b"\r.").unwrap(),
            (b".".as_slice(), b"\r".as_slice())
        );
        assert_eq!(
            white_space(b"\x00.").unwrap(),
            (b".".as_slice(), b"\x00".as_slice())
        );
        // Comments
        assert_eq!(
            white_space(b"  %%EOF\n").unwrap(),
            (b"%%EOF\n".as_slice(), b"  ".as_slice())
        );
        // Delimiters
        assert_eq!(
            white_space(b"  <").unwrap(),
            (b"<".as_slice(), b"  ".as_slice())
        );
        // Regular characters
        assert_eq!(
            white_space(b" R").unwrap(),
            (b"R".as_slice(), b" ".as_slice())
        );
    }

    #[test]
    fn white_space_invalid() {
        let parse_result = white_space(b"%%EOF");
        let expected_error = Err(NomErr::Error(NomError::new(
            "%%EOF".as_bytes(),
            ErrorKind::TakeWhile1,
        )));
        assert_eq!(parse_result, expected_error);

        let parse_result = white_space(b">");
        let expected_error = Err(NomErr::Error(NomError::new(
            ">".as_bytes(),
            ErrorKind::TakeWhile1,
        )));
        assert_eq!(parse_result, expected_error);

        let parse_result = white_space(b"R");
        let expected_error = Err(NomErr::Error(NomError::new(
            "R".as_bytes(),
            ErrorKind::TakeWhile1,
        )));
        assert_eq!(parse_result, expected_error);
    }

    #[test]
    fn comment_valid() {
        assert_eq!(
            comment(b"%\r\n").unwrap(),
            ("\r\n".as_bytes(), "".as_bytes())
        );

        assert_eq!(
            comment(b"%%EOF\n").unwrap(),
            ("\n".as_bytes(), "%EOF".as_bytes())
        );

        assert_eq!(
            comment(b"%PDF\r\nLINE2").unwrap(),
            ("\r\nLINE2".as_bytes(), "PDF".as_bytes())
        );

        assert_eq!(
            comment(b"%PDF\rLINE2%ANOTHER COMMENT\n").unwrap(),
            ("\rLINE2%ANOTHER COMMENT\n".as_bytes(), "PDF".as_bytes())
        );

        // Non-printable characters but valid UTF-8
        assert_eq!(
            comment(b"%\x20\x03\x16\xC3\x80\r\n").unwrap(),
            ("\r\n".as_bytes(), b"\x20\x03\x16\xC3\x80".as_slice())
        );

        // Invalid UTF-8
        assert_eq!(
            comment(b"%\x00\x9F\x92\x96\r\n").unwrap(),
            ("\r\n".as_bytes(), b"\x00\x9F\x92\x96".as_slice())
        );

        assert_eq!(
            comment(b"%START OF COMMENT... %STILL WITHIN THE SAME COMMENT...\nLINE2").unwrap(),
            (
                "\nLINE2".as_bytes(),
                "START OF COMMENT... %STILL WITHIN THE SAME COMMENT...".as_bytes()
            )
        );
    }

    #[test]
    fn white_space_or_comment_valid() {
        assert_eq!(
            white_space_or_comment(b" .").unwrap(),
            (b".".as_slice(), b" ".as_slice())
        );
        assert_eq!(
            white_space_or_comment(b"\t.").unwrap(),
            (b".".as_slice(), b"\t".as_slice())
        );
        assert_eq!(
            white_space_or_comment(b"\n.").unwrap(),
            (b".".as_slice(), b"\n".as_slice())
        );
        assert_eq!(
            white_space_or_comment(b"\x0C.").unwrap(),
            (b".".as_slice(), b"\x0C".as_slice())
        );
        assert_eq!(
            white_space_or_comment(b"\r.").unwrap(),
            (b".".as_slice(), b"\r".as_slice())
        );
        assert_eq!(
            white_space_or_comment(b"\x00.").unwrap(),
            (b".".as_slice(), b"\x00".as_slice())
        );
        // Comments
        assert_eq!(
            white_space_or_comment(b"  %A COMMENT\n<").unwrap(),
            (b"<".as_slice(), b"  %A COMMENT\n".as_slice())
        );
        // Delimiters
        assert_eq!(
            white_space_or_comment(b"  <").unwrap(),
            (b"<".as_slice(), b"  ".as_slice())
        );
        // Regular characters
        assert_eq!(
            white_space_or_comment(b" R").unwrap(),
            (b"R".as_slice(), b" ".as_slice())
        );
    }

    #[test]
    fn white_space_or_comment_invalid() {
        let parse_result = white_space_or_comment(b">");
        let expected_error = Err(NomErr::Error(NomError::new(
            ">".as_bytes(),
            ErrorKind::Char,
        )));
        assert_eq!(parse_result, expected_error);

        let parse_result = white_space_or_comment(b"R");
        let expected_error = Err(NomErr::Error(NomError::new(
            "R".as_bytes(),
            ErrorKind::Char,
        )));
        assert_eq!(parse_result, expected_error);
    }

    #[test]
    fn eol_valid() {
        assert_eq!(eol(b"\n<").unwrap(), (b"<".as_slice(), b"\n".as_slice()));
        assert_eq!(eol(b"\r<").unwrap(), (b"<".as_slice(), b"\r".as_slice()));
        assert_eq!(
            eol(b"\r\n<").unwrap(),
            (b"<".as_slice(), b"\r\n".as_slice())
        );
    }

    #[test]
    fn delimiter_valid() {
        assert_eq!(
            delimiter(b"() ").unwrap(),
            (b" ".as_slice(), b"()".as_slice())
        );
        assert_eq!(
            delimiter(b"<<>> ").unwrap(),
            (b" ".as_slice(), b"<<>>".as_slice())
        );
        assert_eq!(
            delimiter(b"<> ").unwrap(),
            (b" ".as_slice(), b"<>".as_slice())
        );
        assert_eq!(
            delimiter(b"[] ").unwrap(),
            (b" ".as_slice(), b"[]".as_slice())
        );
        assert_eq!(
            delimiter(b"{} ").unwrap(),
            (b" ".as_slice(), b"{}".as_slice())
        );
        assert_eq!(
            delimiter(b"/ ").unwrap(),
            (b" ".as_slice(), b"/".as_slice())
        );
        assert_eq!(
            delimiter(b"% ").unwrap(),
            (b" ".as_slice(), b"%".as_slice())
        );
    }

    #[test]
    fn delimiter_invalid() {
        let parse_result = delimiter(b" ");
        let expected_error = Err(NomErr::Error(NomError::new(
            " ".as_bytes(),
            ErrorKind::TakeWhile1,
        )));
        assert_eq!(parse_result, expected_error);

        let parse_result = delimiter(b"R");
        let expected_error = Err(NomErr::Error(NomError::new(
            "R".as_bytes(),
            ErrorKind::TakeWhile1,
        )));
        assert_eq!(parse_result, expected_error);
    }

    #[test]
    fn printable_token_valid() {
        assert_eq!(
            printable_token(b"This#20is#20a#20token").unwrap(),
            (b"".as_slice(), b"This#20is#20a#20token".as_slice())
        );
        assert_eq!(
            printable_token(b"123").unwrap(),
            (b"".as_slice(), b"123".as_slice())
        );
        assert_eq!(
            printable_token(b"00FF").unwrap(),
            (b"".as_slice(), b"00FF".as_slice())
        );
        assert_eq!(
            printable_token(b"0.123").unwrap(),
            (b"".as_slice(), b"0.123".as_slice())
        );
    }

    #[test]
    fn printable_token_invalid() {
        let parse_result = printable_token(&[0x80, 0x81, 0x82, 0x83]);
        let expected_error = Err(NomErr::Error(NomError::new(
            [0x80, 0x81, 0x82, 0x83].as_slice(),
            ErrorKind::TakeWhile1,
        )));
        assert_eq!(parse_result, expected_error);
    }
}
