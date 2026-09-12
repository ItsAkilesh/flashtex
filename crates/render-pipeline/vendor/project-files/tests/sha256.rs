use flashtex_project_files::{Sha256, sha256_from_hex, sha256_hex, sha256_to_hex};

#[test]
fn fips_vectors() {
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
    let million_a = vec![b'a'; 1_000_000];
    assert_eq!(
        sha256_hex(&million_a),
        "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
    );
}

#[test]
fn streaming_matches_one_shot_for_every_chunking() {
    let data: Vec<u8> = (0..300u32).map(|i| (i * 7 % 251) as u8).collect();
    let expected = sha256_hex(&data);
    for chunk in [1usize, 3, 55, 56, 63, 64, 65, 127, 128, 200] {
        let mut h = Sha256::new();
        for part in data.chunks(chunk) {
            h.update(part);
        }
        assert_eq!(sha256_to_hex(&h.finalize()), expected, "chunk size {chunk}");
    }
}

#[test]
fn hex_round_trip() {
    let d = flashtex_project_files::sha256(b"abc");
    assert_eq!(sha256_from_hex(&sha256_to_hex(&d)), Some(d));
    assert_eq!(sha256_from_hex("zz"), None);
}
