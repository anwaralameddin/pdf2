OwnedStream::new(
    OwnedDictionary::from_iter([
        ("Type".into(), OwnedName::from("XRef").into()),
        (
            "Index".into(),
            OwnedArray::from_iter([0.into(), 440.into()]).into(),
        ),
        ("Size".into(), 440.into()),
        (
            "W".into(),
            OwnedArray::from_iter([
                1.into(),
                3.into(),
                1.into(),
            ])
            .into(),
        ),
        ("Root".into(), unsafe {
            Reference::new_unchecked(437, 0).into()
        }),
        ("Info".into(), unsafe {
            Reference::new_unchecked(438, 0).into()
        }),
        (
            "ID".into(),
            OwnedArray::from_iter([
                OwnedHexadecimal::from("3AB9790B3CB9A73CF4BF095B2CE17671").into(),
                OwnedHexadecimal::from("3AB9790B3CB9A73CF4BF095B2CE17671").into(),
            ])
            .into(),
        ),
        ("Length".into(), 1089.into()),
        ("Filter".into(), OwnedName::from("FlateDecode").into()),
    ]),
    &buffer[205..1294],
)
