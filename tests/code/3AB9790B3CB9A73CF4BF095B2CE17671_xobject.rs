Stream::new(
    Dictionary::new([
        (
            "Type".into(),
            Name::from(("XObject", Span::new(9, 8))).into(),
        ),
        (
            "Subtype".into(),
            Name::from(("Form", Span::new(27, 5))).into(),
        ),
        (
            "BBox".into(),
            Array::new(vec![
                Integer::new(0, Span::new(40, 1)).into(),
                Integer::new(0, Span::new(42, 1)).into(),
                Integer::new(100, Span::new(44, 3)).into(),
                Integer::new(100, Span::new(48, 3)).into(),
            ], Span::new(39, 13)).into(),
        ),
        (
            "FormType".into(),
            Integer::new(1, Span::new(63, 1)).into(),
        ),
        (
            "Matrix".into(),
            Array::new(vec![
                Integer::new(1, Span::new(74, 1)).into(),
                Integer::new(0, Span::new(76, 1)).into(),
                Integer::new(0, Span::new(78, 1)).into(),
                Integer::new(1, Span::new(80, 1)).into(),
                Integer::new(0, Span::new(82, 1)).into(),
                Integer::new(0, Span::new(84, 1)).into(),
            ], Span::new(73, 13)).into(),
        ),
        (
            "Resources".into(),
            unsafe { Reference::new_unchecked(11, 0, 98, 6).into() },
        ),
        ("Length".into(), Integer::new(15, Span::new(113, 2)).into(),),
        ("Filter".into(), Name::from(("FlateDecode", Span::new(132, 12))).into()),
    ], Span::new(0, 147)),
    &buffer[155..170],
    Span::new(0, 181),
)
