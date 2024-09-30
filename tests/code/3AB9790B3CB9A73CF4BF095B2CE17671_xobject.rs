Stream::new(
    Dictionary::from_iter([
        (
            "Type".into(),
            Name::from("XObject").into(),
        ),
        (
            "Subtype".into(),
            Name::from("Form").into(),
        ),
        (
            "BBox".into(),
            Array::from_iter([
                0.into(),
                0.into(),
                100.into(),
                100.into(),
            ])
            .into(),
        ),
        (
            "FormType".into(),
            1.into(),
        ),
        (
            "Matrix".into(),
            Array::from_iter([
                1.into(),
                0.into(),
                0.into(),
                1.into(),
                0.into(),
                0.into(),
            ])
            .into(),
        ),
        (
            "Resources".into(),
            unsafe { Reference::new_unchecked(11, 0, 0, 0).into() },
        ),
        ("Length".into(), 15.into()),
        ("Filter".into(), Name::from("FlateDecode").into()),
    ]),
    &buffer[155..170],
    Span::new(0, buffer.len()),
)
