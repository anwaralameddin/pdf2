Stream::new(
    Dictionary::new([
        ("Type".into(), Name::from(("XRef", Span::new(7, 12))).into()),
        ("Size".into(), Integer::new(60, Span::new(18, 20)).into()),
        (
            "W".into(),
            Array::new([
                Integer::new(1, Span::new(24, 25)).into(),
                Integer::new(4, Span::new(26, 27)).into(),
                Integer::new(2, Span::new(28, 29)).into(),
            ], Span::new(22, 30)).into(),
        ),
        ("Root".into(), unsafe {
            Reference::new_unchecked(1, 0, 37, 42).into()
        }),
        ("Info".into(), unsafe {
            Reference::new_unchecked(24, 0, 48, 54).into()
        }),
        (
            "ID".into(),
            Array::new([
                Hexadecimal::from(("B72168B54640B245A7CCF42DCDC8C026", Span::new(58 , 92))).into(),
                Hexadecimal::from(("B72168B54640B245A7CCF42DCDC8C026", Span::new(92 , 126))).into(),
            ], Span::new(57, 127)).into(),
        ),
        ("Filter".into(), Name::from(("FlateDecode", Span::new(135, 147))).into()),
        ("Length".into(), Integer::new(193, Span::new(155, 158)).into()),
    ], Span::new(0, 160)),
    &buffer[170..363],
    Span::new(0, 376),
)
