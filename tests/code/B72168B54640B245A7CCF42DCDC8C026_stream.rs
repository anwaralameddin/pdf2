Stream::new(
    Dictionary::from_iter([
        ("Type".into(), Name::from("XRef").into()),
        ("Size".into(), 60.into()),
        (
            "W".into(),
            Array::from_iter([
                1.into(),
                4.into(),
                2.into(),
            ])
            .into(),
        ),
        ("Root".into(), unsafe {
            Reference::new_unchecked(1, 0, 0, 0).into()
        }),
        ("Info".into(), unsafe {
            Reference::new_unchecked(24, 0, 0, 0).into()
        }),
        (
            "ID".into(),
            Array::from_iter([
                Hexadecimal::from("B72168B54640B245A7CCF42DCDC8C026").into(),
                Hexadecimal::from("B72168B54640B245A7CCF42DCDC8C026").into(),
            ])
            .into(),
        ),
        ("Filter".into(), Name::from("FlateDecode").into()),
        ("Length".into(), 193.into()),
    ]),
    &buffer[170..363],
    Span::new(0, buffer.len()),
)
