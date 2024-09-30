Stream::new(
    Dictionary::from_iter([
        ("Type".into(), Name::from(("XRef", Span::new(0, 0))).into()),
        (
            "Index".into(),
            Array::new(vec![
                Integer::new(0, Span::new(0, 1)).into(),
                Integer::new(440, Span::new(2, 3)).into(),
            ], Span::new(0, 0)).into(),
        ),
        ("Size".into(), Integer::new(440, Span::new(2, 3)).into()),
        (
            "W".into(),
            Array::new(vec![
                Integer::new(1, Span::new(0, 1)).into(),
                Integer::new(3, Span::new(0, 1)).into(),
                Integer::new(1, Span::new(2, 1)).into(),
            ], Span::new(0, 0)).into(),
        ),
        ("Root".into(), unsafe {
            Reference::new_unchecked(437, 0, 0, 0).into()
        }),
        ("Info".into(), unsafe {
            Reference::new_unchecked(438, 0, 0, 0).into()
        }),
        (
            "ID".into(),
            Array::new(vec![
                Hexadecimal::from(("3AB9790B3CB9A73CF4BF095B2CE17671", Span::new(0, 32))).into(),
                Hexadecimal::from(("3AB9790B3CB9A73CF4BF095B2CE17671", Span::new(0, 32))).into(),
            ], Span::new(0, 0)).into(),
        ),
        ("Length".into(), Integer::new(1089, Span::new(2, 4)).into()),
        ("Filter".into(), Name::from(("FlateDecode", Span::new(0, 0))).into()),
    ]),
    &buffer[205..1294],
    Span::new(0, buffer.len()),
)
