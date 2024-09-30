unsafe {
    Trailer::new(160, Span::new(0, 0), &dictionary)
    .set_root(Reference::new_unchecked(158, 0, 0, 0))
    .set_info(Reference::new_unchecked(159, 0, 0, 0))
    .set_id([
        Hexadecimal::from("483F2EC937A8888A3F98DD1FF73B1F6B").into(),
        Hexadecimal::from("483F2EC937A8888A3F98DD1FF73B1F6B").into(),
    ])
}
