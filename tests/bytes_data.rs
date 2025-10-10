use ehttpd::bytes::{Data, DataSliceExt};

/// Tests the data representation
fn test_data(bytes: Data, as_ref: &[u8]) {
    // Test data
    assert_eq!(bytes.as_ref(), as_ref);

    // Validate cloning
    let clone = bytes.clone();
    assert_eq!(clone.as_ref(), as_ref);

    // Validate valid subcopy
    let range = match bytes.len() {
        0 | 1 => 0..0,
        2 => 0..1,
        _ => 1..2,
    };
    let subcopy_valid = bytes.subcopy(range.start..range.end).expect("failed to create valid subcopy");
    assert_eq!(subcopy_valid.as_ref(), &as_ref[range.start..range.end]);

    // Validate invalid subcopy
    let subcopy_invalid = bytes.subcopy(..=as_ref.len());
    assert!(subcopy_invalid.is_none());
}

/// Tests empty data
#[test]
fn empty() {
    let bytes = Data::new_empty();
    test_data(bytes, b"")
}

/// Tests static data
#[test]
fn static_() {
    let bytes = Data::new_static(b"Testolope");
    test_data(bytes, b"Testolope")
}

/// Tests other data
#[test]
fn heap() {
    let bytes = Data::new("Testolope".as_bytes().to_vec());
    test_data(bytes, b"Testolope")
}
