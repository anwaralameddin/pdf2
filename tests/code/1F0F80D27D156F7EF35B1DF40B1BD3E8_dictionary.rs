Dictionary::from_iter([
    ("Type".into(), Name::from("XRef").into()),
    ("Size".into(), 750.into()),
    ("Root".into(), unsafe {
        Reference::new_unchecked(747, 0).into()
    }),
    (
        "W".into(),
        Array::from_iter([1.into(), 3.into(), 1.into()]).into(),
    ),
    (
        "Index".into(),
        Array::from_iter([0.into(), 750.into()]).into(),
    ),
    ("Info".into(), unsafe {
        Reference::new_unchecked(748, 0).into()
    }),
    (
        "ID".into(),
        Array::from_iter([
            Hexadecimal::from("1F0F80D27D156F7EF35B1DF40B1BD3E8").into(),
            Hexadecimal::from("1F0F80D27D156F7EF35B1DF40B1BD3E8").into(),
        ])
        .into(),
    ),
    ("Length".into(), 1760.into()),
    ("Filter".into(), Name::from("FlateDecode").into()),
])
