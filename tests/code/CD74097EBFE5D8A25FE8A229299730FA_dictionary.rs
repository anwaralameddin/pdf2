Dictionary::new([
    ("Type".into(), Name::from(("XRef", Span::new(19, 24))).into()),
    ("Size".into(), Integer::new(191, Span::new(46, 49)).into(),),
    ("Root".into(), unsafe {
        Reference::new_unchecked(188, 0, 67, 74).into()
    }),
    (
        "W".into(),
        Array::new([
            Integer::new(1, Span::new(54, 55)).into(),
            Integer::new(3, Span::new(56, 57)).into(),
            Integer::new(1, Span::new(58, 59)).into(),
        ], Span::new(53, 60)).into(),
    ),
    (
        "Index".into(),
        Array::new([
            Integer::new(0, Span::new(33, 34)).into(),
            Integer::new(191, Span::new(35, 38)).into(),
        ], Span::new(32, 39)).into(),
    ),
    ("Info".into(), unsafe {
        Reference::new_unchecked(189, 0, 81, 88).into()
    }),
    (
        "ID".into(),
        Array::new([
            Hexadecimal::from(("CD74097EBFE5D8A25FE8A229299730FA", Span::new(94, 128))).into(),
            Hexadecimal::from(("CD74097EBFE5D8A25FE8A229299730FA", Span::new(129, 163))).into(),
        ], Span::new(93, 164)).into(),
    ),
    ("Length".into(), Integer::new(502, Span::new(173, 176)).into(),),
    ("Filter".into(), Name::from(("FlateDecode", Span::new(192, 204))).into()),
], Span::new(10, 207))
