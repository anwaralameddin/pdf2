OwnedDictionary::from_iter([
    ("Type".into(), OwnedName::from("XRef").into()),
    ("Size".into(), 191.into()),
    ("Root".into(), unsafe {
        Reference::new_unchecked(188, 0).into()
    }),
    (
        "W".into(),
        OwnedArray::from_iter([1.into(), 3.into(), 1.into()]).into(),
    ),
    (
        "Index".into(),
        OwnedArray::from_iter([0.into(), 191.into()]).into(),
    ),
    ("Info".into(), unsafe {
        Reference::new_unchecked(189, 0).into()
    }),
    (
        "ID".into(),
        OwnedArray::from_iter([
            OwnedHexadecimal::from("CD74097EBFE5D8A25FE8A229299730FA").into(),
            OwnedHexadecimal::from("CD74097EBFE5D8A25FE8A229299730FA").into(),
        ])
        .into(),
    ),
    ("Length".into(), 502.into()),
    ("Filter".into(), OwnedName::from("FlateDecode").into()),
])
