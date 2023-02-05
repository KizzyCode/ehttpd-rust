use ehttpd::bytes::{Data, DataSliceExt};

/// Tests the data represenataion
fn test_data(bytes: Data, as_ref: &[u8], as_debug: &str) {
    // Test data
    assert_eq!(bytes.as_ref(), as_ref);

    // Validate debug representation
    let debug = format!("{bytes:?}");
    assert_eq!(debug, as_debug);

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
    let bytes = Data::Empty;
    test_data(bytes, b"", "Empty")
}

/// Tests static data
#[test]
fn static_() {
    let bytes = Data::Static(b"Testolope");
    test_data(bytes, b"Testolope", "Static([84, 101, 115, 116, 111, 108, 111, 112, 101])")
}

/// Tests RcVec data
#[test]
fn rc_vec() {
    let bytes = Data::new_arcvec(*b"Testolope");
    test_data(bytes, b"Testolope", "RcVec { backing: [84, 101, 115, 116, 111, 108, 111, 112, 101], range: 0..9 }")
}

/// Tests other data
#[test]
fn other() {
    /// Some other data
    #[derive(Debug, Clone)]
    struct StringData {
        /// The underlying string
        string: String,
    }
    impl AsRef<[u8]> for StringData {
        fn as_ref(&self) -> &[u8] {
            self.string.as_bytes()
        }
    }

    // Test the bytes
    let string_data = StringData { string: "Testolope".to_string() };
    let bytes = Data::new_other(string_data);
    test_data(bytes, b"Testolope", r#"Other { data: StringData { string: "Testolope" }, range: 0..9 }"#)
}
