Dictionary::from_iter([
    ("Type".into(), Name::from("XRef").into()),
    ("Size".into(), 440.into()),
    ("Root".into(), unsafe {
        Reference::new_unchecked(437, 0).into()
    }),
    (
        "W".into(),
        Array::from_iter([1.into(), 3.into(), 1.into()]).into(),
    ),
    (
        "Index".into(),
        Array::from_iter([0.into(), 440.into()]).into(),
    ),
    ("Info".into(), unsafe {
        Reference::new_unchecked(438, 0).into()
    }),
    (
        "ID".into(),
        Array::from_iter([
            Hexadecimal::from("3AB9790B3CB9A73CF4BF095B2CE17671").into(),
            Hexadecimal::from("3AB9790B3CB9A73CF4BF095B2CE17671").into(),
        ])
        .into(),
    ),
    ("Length".into(), 1089.into()),
    ("Filter".into(), Name::from("FlateDecode").into()),
])
