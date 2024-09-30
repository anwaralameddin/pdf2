Dictionary::from_iter([
    ("Type".into(), Name::from(("XRef", Span::new(0, 0))).into()),
    ("Size".into(), Integer::new(440, Span::new(0, 3)).into(),),
    ("Root".into(), unsafe {
        Reference::new_unchecked(437, 0, 0, 0).into()
    }),
    (
        "W".into(),
        Array::from_iter([
            Integer::new(1, Span::new(0, 1)).into(),
            Integer::new(3, Span::new(2, 1)).into(),
            Integer::new(1, Span::new(4, 1)).into(),
        ]).into(),
    ),
    (
        "Index".into(),
        Array::from_iter([
            Integer::new(0, Span::new(0, 1)).into(),
            Integer::new(440, Span::new(2, 3)).into(),
        ]).into(),
    ),
    ("Info".into(), unsafe {
        Reference::new_unchecked(438, 0, 0, 0).into()
    }),
    (
        "ID".into(),
        Array::from_iter([
            Hexadecimal::from(("3AB9790B3CB9A73CF4BF095B2CE17671", Span::new(0, 32))).into(),
            Hexadecimal::from(("3AB9790B3CB9A73CF4BF095B2CE17671", Span::new(0, 32))).into(),
        ])
        .into(),
    ),
    ("Length".into(), Integer::new(1089, Span::new(0, 4)).into(),),
    ("Filter".into(), Name::from(("FlateDecode", Span::new(0, 0))).into()),
])
