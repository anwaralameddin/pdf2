unsafe {
    Trailer::new(100)
    .set_root(Reference::new_unchecked(99, 0))
    .set_info(Reference::new_unchecked(98, 0))
}
