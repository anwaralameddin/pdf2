Stream::new(
    Dictionary::new([
        ("Type".into(), Name::from(("XRef", Span::new(9, 14))).into()),
        (
            "Index".into(),
            Array::new([
                Integer::new(0, Span::new(23, 24)).into(),
                Integer::new(440, Span::new(25, 28)).into(),
            ], Span::new(22, 29)).into(),
        ),
        ("Size".into(), Integer::new(440, Span::new(36, 39)).into()),
        (
            "W".into(),
            Array::new([
                Integer::new(1, Span::new(44, 45)).into(),
                Integer::new(3, Span::new(46, 47)).into(),
                Integer::new(1, Span::new(48, 49)).into(),
            ], Span::new(43, 50)).into(),
        ),
        ("Root".into(), unsafe {
            Reference::new_unchecked(437, 0, 57, 64).into()
        }),
        ("Info".into(), unsafe {
            Reference::new_unchecked(438, 0, 71, 78).into()
        }),
        (
            "ID".into(),
            Array::new([
                Hexadecimal::from(("3AB9790B3CB9A73CF4BF095B2CE17671", Span::new(84, 118))).into(),
                Hexadecimal::from(("3AB9790B3CB9A73CF4BF095B2CE17671", Span::new(119, 153))).into(),
            ], Span::new(83, 154)).into(),
        ),
        ("Length".into(), Integer::new(1089, Span::new(163, 167)).into()),
        ("Filter".into(), Name::from(("FlateDecode", Span::new(182, 194))).into()),
    ], Span::new(0, 197)),
    &buffer[205..1294],
    Span::new(0, 1305),
)
