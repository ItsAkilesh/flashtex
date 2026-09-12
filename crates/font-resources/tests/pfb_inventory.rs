use flashtex_font_resources::{pfb::*, sha256, EmbeddingPermission, LicenseMetadata};
#[test]
#[ignore = "set FLASHTEX_LM_TFM_DIR to existing official2.004 directory containing lmr10.pfb and LICENSE"]
fn pinned_official_type1_container_only() {
    let p = std::path::PathBuf::from(std::env::var("FLASHTEX_LM_TFM_DIR").unwrap());
    let bytes = std::fs::read(p.join("lmr10.pfb")).unwrap();
    let license = std::fs::read(p.join("LICENSE")).unwrap();
    let provenance: serde_json::Value = serde_json::from_str(include_str!(
        "../fixtures/lm-required-metrics-provenance.json"
    ))
    .unwrap();
    assert_eq!(
        sha256(&bytes),
        "84eb01245abb17c0530ca3909427256d73df8bed2d7243b9f2717ca08c010ac8"
    );
    assert_eq!(
        sha256(&license),
        provenance["license"]["sha256"].as_str().unwrap()
    );
    let identity = Identity {
        resource_id: "lmr10-type1-container".into(),
        sha256: sha256(&bytes),
        byte_length: bytes.len() as u64,
        license: LicenseMetadata {
            identifier: "LicenseRef-GUST-Font-License".into(),
            copyright: "see pinned license".into(),
            source: "official GUST lm2.004bas.zip".into(),
            text_path: "LICENSE".into(),
            text_sha256: sha256(&license),
            embedding_permission: EmbeddingPermission::Unknown,
        },
    };
    let resource = Resource::from_bytes(&identity, &bytes, &license).unwrap();
    assert_eq!(resource.segments().len(), 3);
    for s in resource.segments() {
        println!("{:?} {:?}", s.kind, s.payload);
    }
    let inspection =
        flashtex_font_resources::eexec::inspect_binary_eexec(&resource, 112949).unwrap();
    assert_eq!(inspection.random_prefix(), [0; 4]);
    assert_eq!(inspection.plaintext().len(), 112949);
    assert_eq!(
        inspection.plaintext_sha256(),
        "bd88b12233faf829fbf86770638e4aec367847bcdb06e285b75e2381787af9a4"
    );
    assert_eq!(
        inspection.encrypted_sha256(),
        "02262ab31d397263650f1ec77c7bef04d0720419f69aa9e6562a52b2ddc85c62"
    );
    assert_eq!(inspection.resource_identity(), resource.identity());
    let records = flashtex_font_resources::type1_records::extract(&inspection).unwrap();
    assert_eq!(records.len_iv(), 4);
    assert_eq!(records.glyphs().len(), 822);
    assert_eq!(records.subrs().len(), 882);
    assert!(records.opaque_other_subrs_present());
    let a = records.decrypted_glyph("A").unwrap();
    let notdef = records.decrypted_glyph(".notdef").unwrap();
    println!(
        "A {} {} notdef {} {}",
        a.len(),
        sha256(&a),
        notdef.len(),
        sha256(&notdef)
    );
    assert_eq!(
        sha256(&a),
        "13883a7a5915c1d3874a112f50a9e269e7663daf1587f4edbd504e381e16cdec"
    );
    assert_eq!(
        sha256(&notdef),
        "eaa5a748d6652e8857daddbbb79acaa6359e20d1eb90a1b1f0c843b3538a00d8"
    );
    for name in records.glyphs().keys() {
        records.decrypted_glyph(name).unwrap();
    }
    for (index, record) in records.subrs().iter().enumerate() {
        if record.is_some() {
            records.decrypted_subr(index).unwrap();
        }
    }
    assert_eq!(
        resource.require_outlines(),
        Err(Error::EncryptedOutlinesUnsupported)
    );
}
