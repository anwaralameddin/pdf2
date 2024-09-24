unsafe {
    Trailer::new(1660)
    .set_root(Reference::new_unchecked(1, 0))
    .set_info(Reference::new_unchecked(6, 0))
    .set_id([
        OwnedHexadecimal::from("8401FBC530C8AE9B8EC1425170A70921").into(),
        OwnedHexadecimal::from("8401FBC530C8AE9B8EC1425170A70921").into(),
    ])
    .set_others(HashMap::from_iter([
        (
            "rgid".into(),
            OwnedLiteral::from("PB:318039020_AS:510882528206848@1498815294792").into(),
        ),
        (
            "habibi-version".into(),
            OwnedLiteral::from("8.12.0".as_bytes()).into(),
        ),
        (
            "comunity-version".into(),
            OwnedLiteral::from("v189.11.0".as_bytes()).into(),
        ),
        (
            "worker-version".into(),
            OwnedLiteral::from("8.12.0".as_bytes()).into(),
        ),
        (
            "dd".into(),
            OwnedLiteral::from("1498815349362".as_bytes()).into(),
        ),
    ]))
}
