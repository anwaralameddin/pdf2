unsafe {
    Trailer::new(100, Span::new(0, 45))
    .set_root(Reference::new_unchecked(99, 0, 21, 27))
    .set_info(Reference::new_unchecked(98, 0, 35, 41))
}
