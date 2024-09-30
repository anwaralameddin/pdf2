Dictionary::from_iter([
    ("Type".into(), Name::from(("XRef", Span::new(0, 0))).into()),
    ("Size".into(), Integer::new(191, Span::new(0, 3)).into(),),
    ("Root".into(), unsafe {
        Reference::new_unchecked(188, 0, 0, 0).into()
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
            Integer::new(191, Span::new(2, 3)).into(),
        ]).into(),
    ),
    ("Info".into(), unsafe {
        Reference::new_unchecked(189, 0, 0, 0).into()
    }),
    (
        "ID".into(),
        Array::from_iter([
            Hexadecimal::from(("CD74097EBFE5D8A25FE8A229299730FA", Span::new(0, 32))).into(),
            Hexadecimal::from(("CD74097EBFE5D8A25FE8A229299730FA", Span::new(0, 32))).into(),
        ])
        .into(),
    ),
    ("Length".into(), Integer::new(502, Span::new(0, 3)).into(),),
        ("Filter".into(), Name::from(("FlateDecode", Span::new(0, 0))).into()),
])
