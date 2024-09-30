unsafe {
    Trailer::new(1660, Span::new(0, 0), &dictionary)
    .set_root(Reference::new_unchecked(1, 0, 0, 0))
    .set_info(Reference::new_unchecked(6, 0, 0, 0))
    .set_id([
        Hexadecimal::from(("8401FBC530C8AE9B8EC1425170A70921", Span::new(0, 32))).into(),
        Hexadecimal::from(("8401FBC530C8AE9B8EC1425170A70921", Span::new(0, 32))).into(),
    ])
    .set_others(HashMap::from_iter([
        (&key_rigid, &vale_rigid),
        (&key_habibi, &val_habibi),
        (&key_comunity, &val_comunity),
        (&key_worker, &val_worker),
        (&key_dd, &val_dd),
    ]))
}
