Dictionary::from_iter([
    ("Type".into(), Name::from("XRef").into()),
    ("Size".into(), 191.into()),
    ("Root".into(), unsafe {
        Reference::new_unchecked(188, 0, 0, 0).into()
    }),
    (
        "W".into(),
        Array::from_iter([1.into(), 3.into(), 1.into()]).into(),
    ),
    (
        "Index".into(),
        Array::from_iter([0.into(), 191.into()]).into(),
    ),
    ("Info".into(), unsafe {
        Reference::new_unchecked(189, 0, 0, 0).into()
    }),
    (
        "ID".into(),
        Array::from_iter([
            Hexadecimal::from("CD74097EBFE5D8A25FE8A229299730FA").into(),
            Hexadecimal::from("CD74097EBFE5D8A25FE8A229299730FA").into(),
        ])
        .into(),
    ),
    ("Length".into(), 502.into()),
    ("Filter".into(), Name::from("FlateDecode").into()),
])
