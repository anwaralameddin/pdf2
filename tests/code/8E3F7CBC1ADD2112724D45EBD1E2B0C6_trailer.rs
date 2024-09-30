unsafe {
    Trailer::new(311, Span::new(0, 0), &dictionary)
    .set_root(Reference::new_unchecked(303, 0))
    .set_id([
        Hexadecimal::from("8E3F7CBC1ADD2112724D45EBD1E2B0C6")
        .into(),
        Hexadecimal::from("8E3F7CBC1ADD2112724D45EBD1E2B0C6")
        .into(),
    ])
}
