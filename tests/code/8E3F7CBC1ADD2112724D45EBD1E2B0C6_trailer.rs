unsafe {
    Trailer::new(311, Span::new(0, 73))
    .set_root(Reference::new_unchecked(303, 0, 53, 7))
    .set_id([
        Literal::new(b"\x8E\x3F\x7C\xBC\x1A\xDD\x21\x12\x72\x4D\x45\xEB\xD1\xE2\xB0\xC6", Span::new(8, 18)).into(),
        Literal::new(b"\x8E\x3F\x7C\xBC\x1A\xDD\x21\x12\x72\x4D\x45\xEB\xD1\xE2\xB0\xC6", Span::new(27, 18)).into(),
    ])
}
