OwnedStream::new(
    OwnedDictionary::from_iter([
        ("Type".into(), OwnedName::from("XRef").into()),
        ("Size".into(), 60.into()),
        (
            "W".into(),
            OwnedArray::from_iter([
                1.into(),
                4.into(),
                2.into(),
            ])
            .into(),
        ),
        ("Root".into(), unsafe {
            Reference::new_unchecked(1, 0).into()
        }),
        ("Info".into(), unsafe {
            Reference::new_unchecked(24, 0).into()
        }),
        (
            "ID".into(),
            OwnedArray::from_iter([
                OwnedHexadecimal::from("B72168B54640B245A7CCF42DCDC8C026").into(),
                OwnedHexadecimal::from("B72168B54640B245A7CCF42DCDC8C026").into(),
            ])
            .into(),
        ),
        ("Filter".into(), OwnedName::from("FlateDecode").into()),
        ("Length".into(), 193.into()),
    ]),
    &buffer[170..363],
)
