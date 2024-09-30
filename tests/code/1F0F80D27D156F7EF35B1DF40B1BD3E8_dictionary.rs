Dictionary::new([
    ("Type".into(), Name::from(("XRef", Span::new(19, 5))).into()),
    ("Size".into(), Integer::new(750, Span::new(46, 3)).into()),
    ("Root".into(), unsafe {
        Reference::new_unchecked(747, 0, 67, 7).into()
    }),
    (
        "W".into(),
        Array::new(vec![
            Integer::new(1, Span::new(54, 1)).into(),
            Integer::new(3, Span::new(56, 1)).into(),
            Integer::new(1, Span::new(58, 1)).into(),
        ], Span::new(53, 7)).into(),
    ),
    (
        "Index".into(),
        Array::new(vec![
            Integer::new(0, Span::new(33, 1)).into(),
            Integer::new(750, Span::new(35, 3)).into(),
        ], Span::new(32, 7)).into()
    ),
    ("Info".into(), unsafe {
        Reference::new_unchecked(748, 0, 81, 7).into()
    }),
    (
        "ID".into(),
        Array::new(vec![
            Hexadecimal::from(("1F0F80D27D156F7EF35B1DF40B1BD3E8", Span::new(94, 34))).into(),
            Hexadecimal::from(("1F0F80D27D156F7EF35B1DF40B1BD3E8", Span::new(129, 34))).into(),
        ], Span::new(93, 71)).into(),
    ),
    ("Length".into(), Integer::new(1760, Span::new(173, 4)).into()),
    ("Filter".into(), Name::from(("FlateDecode", Span::new(192, 12))).into()),
], Span::new(10, 197))
