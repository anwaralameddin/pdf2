Stream::new(
    Dictionary::from_iter([
        ("Type".into(), Name::from(("XRef", Span::new(0, 0))).into()),
        ("Size".into(), Integer::new(60, Span::new(0, 2)).into()),
        (
            "W".into(),
            Array::from_iter([
                Integer::new(1, Span::new(0, 1)).into(),
                Integer::new(4, Span::new(2, 1)).into(),
                Integer::new(2, Span::new(4, 1)).into(),
            ])
            .into(),
        ),
        ("Root".into(), unsafe {
            Reference::new_unchecked(1, 0, 0, 0).into()
        }),
        ("Info".into(), unsafe {
            Reference::new_unchecked(24, 0, 0, 0).into()
        }),
        (
            "ID".into(),
            Array::from_iter([
                Hexadecimal::from(("B72168B54640B245A7CCF42DCDC8C026", Span::new(0 , 32))).into(),
                Hexadecimal::from(("B72168B54640B245A7CCF42DCDC8C026", Span::new(0 , 32))).into(),
            ])
            .into(),
        ),
        ("Filter".into(), Name::from(("FlateDecode", Span::new(0, 0))).into()),
        ("Length".into(), Integer::new(193, Span::new(0, 3)).into()),
    ]),
    &buffer[170..363],
    Span::new(0, buffer.len()),
)
