Stream::new(
    Dictionary::from_iter([
        (
            "Type".into(),
            Name::from(("XObject", Span::new(0, 0))).into(),
        ),
        (
            "Subtype".into(),
            Name::from(("Form", Span::new(0, 0))).into(),
        ),
        (
            "BBox".into(),
            Array::new(vec![
                Integer::new(0, Span::new(0, 1)).into(),
                Integer::new(0, Span::new(2, 1)).into(),
                Integer::new(100, Span::new(4, 3)).into(),
                Integer::new(100, Span::new(8, 3)).into(),
            ], Span::new(0, 12)).into(),
        ),
        (
            "FormType".into(),
            Integer::new(1, Span::new(0, 1)).into(),
        ),
        (
            "Matrix".into(),
            Array::new(vec![
                Integer::new(1, Span::new(0, 1)).into(),
                Integer::new(0, Span::new(2, 1)).into(),
                Integer::new(0, Span::new(4, 1)).into(),
                Integer::new(1, Span::new(6, 1)).into(),
                Integer::new(0, Span::new(8, 1)).into(),
                Integer::new(0, Span::new(10, 1)).into(),
            ], Span::new(0, 12)).into(),
        ),
        (
            "Resources".into(),
            unsafe { Reference::new_unchecked(11, 0, 0, 0).into() },
        ),
        ("Length".into(), Integer::new(15, Span::new(10, 2)).into(),),
        ("Filter".into(), Name::from(("FlateDecode", Span::new(0, 0))).into()),
    ]),
    &buffer[155..170],
    Span::new(0, buffer.len()),
)
