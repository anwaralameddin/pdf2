unsafe {
    Trailer::new(160, Span::new(0, 119))
    .set_root(Reference::new_unchecked(158, 0, 19, 7))
    .set_info(Reference::new_unchecked(159, 0, 33, 7))
    .set_id([
        Hexadecimal::from(("483F2EC937A8888A3F98DD1FF73B1F6B", Span::new(46, 34))).into(),
        Hexadecimal::from(("483F2EC937A8888A3F98DD1FF73B1F6B", Span::new(81, 34))).into(),
    ])
}
