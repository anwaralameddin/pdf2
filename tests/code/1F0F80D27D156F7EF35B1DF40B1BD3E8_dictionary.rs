OwnedDictionary::from_iter([
    ("Type".into(), OwnedName::from("XRef").into()),
    ("Size".into(), 750.into()),
    ("Root".into(), unsafe {
        Reference::new_unchecked(747, 0).into()
    }),
    (
        "W".into(),
        OwnedArray::from_iter([1.into(), 3.into(), 1.into()]).into(),
    ),
    (
        "Index".into(),
        OwnedArray::from_iter([0.into(), 750.into()]).into(),
    ),
    ("Info".into(), unsafe {
        Reference::new_unchecked(748, 0).into()
    }),
    (
        "ID".into(),
        OwnedArray::from_iter([
            OwnedHexadecimal::from("1F0F80D27D156F7EF35B1DF40B1BD3E8").into(),
            OwnedHexadecimal::from("1F0F80D27D156F7EF35B1DF40B1BD3E8").into(),
        ])
        .into(),
    ),
    ("Length".into(), 1760.into()),
    ("Filter".into(), OwnedName::from("FlateDecode").into()),
])
