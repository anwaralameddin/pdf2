Stream::new(
    Dictionary::new([
        ("Type".into(), Name::from(("XRef", Span::new(9, 5))).into()),
        (
            "Index".into(),
            Array::new(vec![
                Integer::new(0, Span::new(23, 1)).into(),
                Integer::new(440, Span::new(25, 3)).into(),
            ], Span::new(22, 7)).into(),
        ),
        ("Size".into(), Integer::new(440, Span::new(36, 3)).into()),
        (
            "W".into(),
            Array::new(vec![
                Integer::new(1, Span::new(44, 1)).into(),
                Integer::new(3, Span::new(46, 1)).into(),
                Integer::new(1, Span::new(48, 1)).into(),
            ], Span::new(43, 7)).into(),
        ),
        ("Root".into(), unsafe {
            Reference::new_unchecked(437, 0, 57, 7).into()
        }),
        ("Info".into(), unsafe {
            Reference::new_unchecked(438, 0, 71, 7).into()
        }),
        (
            "ID".into(),
            Array::new(vec![
                Hexadecimal::from(("3AB9790B3CB9A73CF4BF095B2CE17671", Span::new(84, 34))).into(),
                Hexadecimal::from(("3AB9790B3CB9A73CF4BF095B2CE17671", Span::new(119, 34))).into(),
            ], Span::new(83, 71)).into(),
        ),
        ("Length".into(), Integer::new(1089, Span::new(163, 4)).into()),
        ("Filter".into(), Name::from(("FlateDecode", Span::new(182, 12))).into()),
    ], Span::new(0, 197)),
    &buffer[205..1294],
    Span::new(0, 1305),
)
