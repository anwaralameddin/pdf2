Dictionary::new([
    ("Type".into(), Name::from(("XRef", Span::new(19, 24))).into()),
    ("Size".into(), Integer::new(750, Span::new(46, 49)).into()),
    ("Root".into(), unsafe {
        Reference::new_unchecked(747, 0, 67, 74).into()
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
            Integer::new(750, Span::new(35, 38)).into(),
        ], Span::new(32, 39)).into()
    ),
    ("Info".into(), unsafe {
        Reference::new_unchecked(748, 0, 81, 88).into()
    }),
    (
        "ID".into(),
        Array::new([
            Hexadecimal::from(("1F0F80D27D156F7EF35B1DF40B1BD3E8", Span::new(94, 128))).into(),
            Hexadecimal::from(("1F0F80D27D156F7EF35B1DF40B1BD3E8", Span::new(129, 163))).into(),
        ], Span::new(93, 164)).into(),
    ),
    ("Length".into(), Integer::new(1760, Span::new(173, 177)).into()),
    ("Filter".into(), Name::from(("FlateDecode", Span::new(192, 204))).into()),
], Span::new(10, 207))
