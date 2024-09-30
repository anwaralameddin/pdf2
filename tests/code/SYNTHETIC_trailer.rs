unsafe {
    Trailer::new(100, Span::new(0, 0), &dictionary)
    .set_root(Reference::new_unchecked(99, 0, 0, 0))
    .set_info(Reference::new_unchecked(98, 0, 0, 0))
}
