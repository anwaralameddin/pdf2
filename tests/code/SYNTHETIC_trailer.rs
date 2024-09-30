unsafe {
    Trailer::new(100, Span::new(0, 45), &dictionary)
    .set_root(Reference::new_unchecked(99, 0, 21, 6))
    .set_info(Reference::new_unchecked(98, 0, 35, 6))
}
