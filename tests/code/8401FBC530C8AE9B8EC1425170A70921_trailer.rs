unsafe {
    Trailer::new(1660, Span::new(0, 270), &dictionary)
    .set_root(Reference::new_unchecked(1, 0, 20, 5))
    .set_info(Reference::new_unchecked(6, 0, 32, 5))
    .set_id([
        Hexadecimal::from(("8401FBC530C8AE9B8EC1425170A70921", Span::new(43, 34))).into(),
        Hexadecimal::from(("8401FBC530C8AE9B8EC1425170A70921", Span::new(78, 34))).into(),
    ])
    .set_others([
        (&key_rigid, &vale_rigid),
        (&key_habibi, &val_habibi),
        (&key_comunity, &val_comunity),
        (&key_worker, &val_worker),
        (&key_dd, &val_dd),
    ])
}
