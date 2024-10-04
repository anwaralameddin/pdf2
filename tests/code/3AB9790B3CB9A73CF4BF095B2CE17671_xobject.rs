Stream::new(
    Dictionary::new([
        (
            "Type".into(),
            Name::from(("XObject", Span::new(9, 17))).into(),
        ),
        (
            "Subtype".into(),
            Name::from(("Form", Span::new(27, 32))).into(),
        ),
        (
            "BBox".into(),
            Array::new([
                Integer::new(0, Span::new(40, 41)).into(),
                Integer::new(0, Span::new(42, 43)).into(),
                Integer::new(100, Span::new(44, 47)).into(),
                Integer::new(100, Span::new(48, 51)).into(),
            ], Span::new(39, 52)).into(),
        ),
        (
            "FormType".into(),
            Integer::new(1, Span::new(63, 64)).into(),
        ),
        (
            "Matrix".into(),
            Array::new([
                Integer::new(1, Span::new(74, 75)).into(),
                Integer::new(0, Span::new(76, 77)).into(),
                Integer::new(0, Span::new(78, 79)).into(),
                Integer::new(1, Span::new(80, 81)).into(),
                Integer::new(0, Span::new(82, 83)).into(),
                Integer::new(0, Span::new(84, 85)).into(),
            ], Span::new(73, 86)).into(),
        ),
        (
            "Resources".into(),
            unsafe { Reference::new_unchecked(11, 0, 98, 104).into() },
        ),
        ("Length".into(), Integer::new(15, Span::new(113, 115)).into(),),
        ("Filter".into(), Name::from(("FlateDecode", Span::new(132, 144))).into()),
    ], Span::new(0, 147)),
    &buffer[155..170],
    Span::new(0, 181),
)
