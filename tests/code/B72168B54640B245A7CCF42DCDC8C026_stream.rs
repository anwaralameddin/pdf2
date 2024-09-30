Stream::new(
    Dictionary::new([
        ("Type".into(), Name::from(("XRef", Span::new(7, 5))).into()),
        ("Size".into(), Integer::new(60, Span::new(18, 2)).into()),
        (
            "W".into(),
            Array::new(vec![
                Integer::new(1, Span::new(24, 1)).into(),
                Integer::new(4, Span::new(26, 1)).into(),
                Integer::new(2, Span::new(28, 1)).into(),
            ], Span::new(22, 8)).into(),
        ),
        ("Root".into(), unsafe {
            Reference::new_unchecked(1, 0, 37, 5).into()
        }),
        ("Info".into(), unsafe {
            Reference::new_unchecked(24, 0, 48, 6).into()
        }),
        (
            "ID".into(),
            Array::new(vec![
                Hexadecimal::from(("B72168B54640B245A7CCF42DCDC8C026", Span::new(58 , 34))).into(),
                Hexadecimal::from(("B72168B54640B245A7CCF42DCDC8C026", Span::new(92 , 34))).into(),
            ], Span::new(57, 70)).into(),
        ),
        ("Filter".into(), Name::from(("FlateDecode", Span::new(135, 12))).into()),
        ("Length".into(), Integer::new(193, Span::new(155, 3)).into()),
    ], Span::new(0, 160)),
    &buffer[170..363],
    Span::new(0, 376),
)
