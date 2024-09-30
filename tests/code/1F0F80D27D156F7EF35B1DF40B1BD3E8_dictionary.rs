Dictionary::from_iter([
    ("Type".into(), Name::from(("XRef", Span::new(0, 0))).into()),
    ("Size".into(), Integer::new(750, Span::new(0, 3)).into()),
    ("Root".into(), unsafe {
        Reference::new_unchecked(747, 0, 0, 0).into()
    }),
    (
        "W".into(),
        Array::new(vec![
            Integer::new(1, Span::new(0, 1)).into(),
            Integer::new(3, Span::new(2, 1)).into(),
            Integer::new(1, Span::new(4, 1)).into(),
        ], Span::new(0, 6)).into(),
    ),
    (
        "Index".into(),
        Array::new(vec![
            Integer::new(0, Span::new(0, 1)).into(),
            Integer::new(750, Span::new(2, 3)).into(),
        ], Span::new(0, 6)).into()
    ),
    ("Info".into(), unsafe {
        Reference::new_unchecked(748, 0, 0, 0).into()
    }),
    (
        "ID".into(),
        Array::new(vec![
            Hexadecimal::from(("1F0F80D27D156F7EF35B1DF40B1BD3E8", Span::new(0, 32))).into(),
            Hexadecimal::from(("1F0F80D27D156F7EF35B1DF40B1BD3E8", Span::new(0, 32))).into(),
        ], Span::new(0, 65)).into(),
    ),
    ("Length".into(), Integer::new(1760, Span::new(0, 4)).into()),
    ("Filter".into(), Name::from(("FlateDecode", Span::new(0, 0))).into()),
])
