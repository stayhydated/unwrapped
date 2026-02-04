use unwrapped::{Unwrapped, Wrapped};

#[test]
fn test_unwrapped_from_no_defaults() {
    #[derive(Debug, PartialEq, Unwrapped)]
    struct WithValues {
        val1: Option<i32>,
        val2: Option<String>,
        val3: String,
        val4: Option<Vec<u8>>,
    }

    // From now requires all Option fields to be Some (no defaults!)
    let original = WithValues {
        val1: Some(42),
        val2: Some("hello".to_string()),
        val3: "world".to_string(),
        val4: Some(vec![1, 2, 3]),
    };

    let unwrapped = WithValuesUw::try_from(original).unwrap();
    assert_eq!(unwrapped.val1, 42);
    assert_eq!(unwrapped.val2, "hello".to_string());
    assert_eq!(unwrapped.val3, "world".to_string());
    assert_eq!(unwrapped.val4, vec![1, 2, 3]);

    let converted_back: WithValues = unwrapped.into();
    assert_eq!(
        converted_back,
        WithValues {
            val1: Some(42),
            val2: Some("hello".to_string()),
            val3: "world".to_string(),
            val4: Some(vec![1, 2, 3]),
        }
    );
}

#[test]
fn test_unwrapped_simple_struct() {
    #[derive(Debug, PartialEq, Unwrapped)]
    struct Simple {
        field1: Option<i32>,
        field2: String,
        field3: Option<u64>,
    }

    let original = Simple {
        field1: Some(10),
        field2: "hello".to_string(),
        field3: Some(100),
    };

    let unwrapped = SimpleUw::try_from(original).unwrap();
    assert_eq!(unwrapped.field1, 10);
    assert_eq!(unwrapped.field2, "hello".to_string());
    assert_eq!(unwrapped.field3, 100);

    let converted_back: Simple = unwrapped.into();
    assert_eq!(
        converted_back,
        Simple {
            field1: Some(10),
            field2: "hello".to_string(),
            field3: Some(100),
        }
    );

    let original_fail = Simple {
        field1: None,
        field2: "world".to_string(),
        field3: Some(200),
    };

    let result = SimpleUw::try_from(original_fail);
    assert!(result.is_err());
    match result {
        Err(e) => assert_eq!(e.field_name, "field1"),
        Ok(_) => panic!("Expected error"),
    }
}

#[test]
fn test_unwrapped_with_custom_name() {
    #[derive(Debug, PartialEq, Unwrapped)]
    #[unwrapped(prefix = "A", name = UserUnwrapped, suffix = c)]
    struct User0;

    #[allow(dead_code)]
    type Works0 = AUserUnwrappedc;

    #[derive(Debug, PartialEq, Unwrapped)]
    #[unwrapped(prefix = Bad)]
    struct User1;

    #[allow(dead_code)]
    type Works1 = BadUser1;

    #[derive(Debug, PartialEq, Unwrapped)]
    #[unwrapped(suffix = "Something")]
    struct User2;

    #[allow(dead_code)]
    type Works2 = User2Something;

    #[derive(Debug, PartialEq, Unwrapped)]
    #[unwrapped(prefix = Bad, suffix = Something)]
    struct User3;

    #[allow(dead_code)]
    type Works3 = BadUser3Something;
}

#[test]
fn test_unwrapped_with_generics() {
    #[derive(Debug, PartialEq, Unwrapped)]
    struct Generic<T: Clone + PartialEq + std::fmt::Debug + Default> {
        value: Option<T>,
        id: i32,
    }

    let original = Generic {
        value: Some(true),
        id: 123,
    };

    let unwrapped = GenericUw::try_from(original).unwrap();
    assert_eq!(unwrapped.value, true);
    assert_eq!(unwrapped.id, 123);

    let converted_back: Generic<bool> = unwrapped.into();
    assert_eq!(
        converted_back,
        Generic {
            value: Some(true),
            id: 123
        }
    );

    let original_fail = Generic::<bool> {
        value: None,
        id: 456,
    };
    let result = GenericUw::try_from(original_fail);
    assert!(result.is_err());
}

#[test]
fn test_struct_with_no_options() {
    #[derive(Clone, Debug, PartialEq, Unwrapped)]
    struct NoOptions {
        a: i32,
        b: bool,
    }

    let original = NoOptions { a: 1, b: false };

    let unwrapped = <NoOptions as Unwrapped>::Unwrapped::try_from(original.clone()).unwrap();
    assert_eq!(unwrapped.a, 1);
    assert_eq!(unwrapped.b, false);

    let converted_back: NoOptions = unwrapped.into();
    assert_eq!(converted_back, original);
}

#[test]
fn test_skip_field() {
    #[derive(Debug, PartialEq, Unwrapped)]
    #[unwrapped(name = SkippedUw)]
    struct Skipped {
        field_a: Option<u32>,
        #[unwrapped(skip)]
        field_b: Option<String>,
        field_c: bool,
    }

    // With skip, field_b is removed from the generated struct
    // SkippedUw only has field_a and field_c
    let unwrapped = SkippedUw {
        field_a: 10,
        field_c: true,
    };
    assert_eq!(unwrapped.field_a, 10);
    assert_eq!(unwrapped.field_c, true);

    // try_from converts Original -> Unwrapped, ignoring skipped fields
    let original = Skipped {
        field_a: Some(123),
        field_b: Some("this will be ignored".to_string()),
        field_c: false,
    };
    let unwrapped2 = SkippedUw::try_from(original).unwrap();
    assert_eq!(unwrapped2.field_a, 123);
    assert_eq!(unwrapped2.field_c, false);

    // try_from fails if non-skipped Option field is None (no defaults!)
    let original_fail = Skipped {
        field_a: None,
        field_b: Some("ignored".to_string()),
        field_c: true,
    };
    let unwrapped_fail = SkippedUw::try_from(original_fail);
    assert!(unwrapped_fail.is_err());
    match unwrapped_fail {
        Err(e) => assert_eq!(e.field_name, "field_a"),
        Ok(_) => panic!("Expected error"),
    }

    // Note: From<Skipped> for SkippedUw is NOT generated when skip is used
    // because we can't convert between structs with different field counts.
    // Only try_from() is available for one-way conversion.
}

#[test]
fn test_skip_field_into_original() {
    #[derive(Debug, PartialEq, Unwrapped)]
    #[unwrapped(name = UserFormUw)]
    struct UserForm {
        name: Option<String>,
        email: Option<String>,
        #[unwrapped(skip)]
        created_at: i64,
        #[unwrapped(skip)]
        id: u64,
    }

    // Create an unwrapped struct (without skipped fields)
    let form = UserFormUw {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    // Convert back to original using into_original, providing skipped fields as parameters
    let original = form.into_original(1234567890, 42);

    assert_eq!(original.name, Some("Alice".to_string()));
    assert_eq!(original.email, Some("alice@example.com".to_string()));
    assert_eq!(original.created_at, 1234567890);
    assert_eq!(original.id, 42);

    // Verify full round-trip works
    let original2 = UserForm {
        name: Some("Bob".to_string()),
        email: Some("bob@example.com".to_string()),
        created_at: 9999999999,
        id: 100,
    };

    let unwrapped = UserFormUw::try_from(original2).unwrap();
    assert_eq!(unwrapped.name, "Bob".to_string());
    assert_eq!(unwrapped.email, "bob@example.com".to_string());

    // Convert back with different skipped field values
    let reconstructed = unwrapped.into_original(1111111111, 200);
    assert_eq!(reconstructed.name, Some("Bob".to_string()));
    assert_eq!(reconstructed.email, Some("bob@example.com".to_string()));
    assert_eq!(reconstructed.created_at, 1111111111); // New value
    assert_eq!(reconstructed.id, 200); // New value
}

#[test]
fn test_skip_field_with_bon_builder_pattern() {
    // This test demonstrates a partial builder helper using bon's typestate API
    #[derive(Debug, PartialEq, Unwrapped, bon::Builder)]
    #[unwrapped(name = UserFormUw)]
    #[builder(on(Option<String>, into))]
    struct UserForm {
        name: Option<String>,
        email: Option<String>,
        #[unwrapped(skip)]
        created_at: i64,
        #[unwrapped(skip)]
        id: u64,
    }

    // Create an unwrapped struct (without skipped fields)
    let form = UserFormUw {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    // Use the macro-generated partial builder helper to pre-fill the non-skipped fields.
    let original = UserForm::builder()
        .from_unwrapped(form)
        .created_at(1234567890)  // Skipped fields
        .id(42)
        .build();

    assert_eq!(original.name, Some("Alice".to_string()));
    assert_eq!(original.email, Some("alice@example.com".to_string()));
    assert_eq!(original.created_at, 1234567890);
    assert_eq!(original.id, 42);

    // The bon builder allows setting fields in any order
    let form2 = UserFormUw {
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
    };

    let original2 = UserForm::builder()
        .id(999) // Set skipped fields first
        .created_at(5555555555)
        .from_unwrapped(form2) // Then non-skipped fields
        .build();

    assert_eq!(original2.name, Some("Bob".to_string()));
    assert_eq!(original2.email, Some("bob@example.com".to_string()));
    assert_eq!(original2.created_at, 5555555555);
    assert_eq!(original2.id, 999);
}

// ==================== Wrapped Tests ====================

#[test]
fn test_wrapped_simple_struct() {
    #[derive(Debug, PartialEq, Wrapped)]
    struct Simple {
        field1: i32,
        field2: String,
        field3: Option<u64>,
    }

    let original = Simple {
        field1: 10,
        field2: "hello".to_string(),
        field3: Some(100),
    };

    // Convert to wrapped - non-Option fields become Option
    let wrapped = SimpleW::from(original);
    assert_eq!(wrapped.field1, Some(10));
    assert_eq!(wrapped.field2, Some("hello".to_string()));
    assert_eq!(wrapped.field3, Some(100)); // Already Option, stays as-is

    // Convert back
    let converted_back: Simple = SimpleW::try_from(wrapped).unwrap();
    assert_eq!(converted_back.field1, 10);
    assert_eq!(converted_back.field2, "hello".to_string());
    assert_eq!(converted_back.field3, Some(100));
}

#[test]
fn test_wrapped_with_none_values() {
    #[derive(Debug, PartialEq, Wrapped)]
    struct WithOptionals {
        required: i32,
        optional: String,
    }

    let wrapped = WithOptionalsW {
        required: Some(42),
        optional: Some("value".to_string()),
    };

    // Conversion from wrapped requires all fields to be Some (no defaults!)
    let converted: WithOptionals = WithOptionalsW::try_from(wrapped).unwrap();
    assert_eq!(converted.required, 42);
    assert_eq!(converted.optional, "value".to_string());
}

#[test]
fn test_wrapped_try_from() {
    #[derive(Debug, PartialEq, Wrapped)]
    struct Config {
        timeout: u64,
        retries: i32,
        name: String,
    }

    let wrapped_all_some = ConfigW {
        timeout: Some(30),
        retries: Some(3),
        name: Some("test".to_string()),
    };

    // try_from should succeed when all fields are Some
    let result = ConfigW::try_from(wrapped_all_some);
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.timeout, 30);
    assert_eq!(config.retries, 3);
    assert_eq!(config.name, "test".to_string());

    let wrapped_missing = ConfigW {
        timeout: Some(30),
        retries: None,
        name: Some("test".to_string()),
    };

    // try_from should fail when any field is None
    let result = ConfigW::try_from(wrapped_missing);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().field_name, "retries");
}

#[test]
fn test_wrapped_with_generics() {
    #[derive(Clone, Debug, PartialEq, Wrapped)]
    struct Generic<T: Clone + PartialEq + std::fmt::Debug + Default> {
        value: T,
        id: i32,
    }

    let original = Generic {
        value: true,
        id: 123,
    };

    let wrapped = GenericW::from(original);
    assert_eq!(wrapped.value, Some(true));
    assert_eq!(wrapped.id, Some(123));

    let converted_back: Generic<bool> = GenericW::try_from(wrapped).unwrap();
    assert_eq!(converted_back.value, true);
    assert_eq!(converted_back.id, 123);
}

#[test]
fn test_wrapped_trait() {
    #[derive(Debug, PartialEq, Wrapped)]
    struct MyStruct {
        data: String,
    }

    fn check_wrapped<T: Wrapped<Wrapped = W>, W>(_: T) {}
    check_wrapped(MyStruct {
        data: "test".to_string(),
    });
}

#[test]
fn test_wrapped_skip_field() {
    #[derive(Debug, PartialEq, Wrapped)]
    #[wrapped(name = SkippedW)]
    struct Skipped {
        field_a: u32,
        #[wrapped(skip)]
        field_b: Option<String>,
        field_c: bool,
    }

    // With skip, field_b is removed from the generated struct
    // SkippedW only has field_a and field_c wrapped in Option
    let wrapped = SkippedW {
        field_a: Some(10),
        field_c: Some(true),
    };
    assert_eq!(wrapped.field_a, Some(10));
    assert_eq!(wrapped.field_c, Some(true));

    // Verify we can construct with None values too
    let wrapped2 = SkippedW {
        field_a: Some(20),
        field_c: None,
    };
    assert_eq!(wrapped2.field_a, Some(20));
    assert_eq!(wrapped2.field_c, None);

    // Note: From implementations are NOT generated when skip is used
    // because we can't convert between structs with different field counts.
}

#[test]
fn test_wrapped_with_custom_name() {
    #[derive(Debug, PartialEq, Wrapped)]
    #[wrapped(prefix = "A", name = UserWrapped, suffix = c)]
    struct User0;

    #[allow(dead_code)]
    type Works0 = AUserWrappedc;

    #[derive(Debug, PartialEq, Wrapped)]
    #[wrapped(prefix = Bad)]
    struct User1;

    #[allow(dead_code)]
    type Works1 = BadUser1;

    #[derive(Debug, PartialEq, Wrapped)]
    #[wrapped(suffix = "Something")]
    struct User2;

    #[allow(dead_code)]
    type Works2 = User2Something;

    #[derive(Debug, PartialEq, Wrapped)]
    #[wrapped(prefix = Bad, suffix = Something)]
    struct User3;

    #[allow(dead_code)]
    type Works3 = BadUser3Something;
}

#[test]
fn test_wrapped_skip_field_into_original() {
    #[derive(Debug, PartialEq, Wrapped)]
    #[wrapped(name = ConfigW)]
    struct Config {
        timeout: u64,
        retries: i32,
        #[wrapped(skip)]
        created_at: i64,
        #[wrapped(skip)]
        version: String,
    }

    // Create a wrapped struct (without skipped fields)
    let wrapped = ConfigW {
        timeout: Some(30),
        retries: Some(3),
    };

    // Convert back to original using into_original, providing skipped fields
    let original = wrapped
        .into_original(1234567890, "v1.0".to_string())
        .unwrap();

    assert_eq!(original.timeout, 30);
    assert_eq!(original.retries, 3);
    assert_eq!(original.created_at, 1234567890);
    assert_eq!(original.version, "v1.0".to_string());

    // Test error case when wrapped field is None
    let wrapped_none = ConfigW {
        timeout: None,
        retries: Some(5),
    };

    let result = wrapped_none.into_original(9999999999, "v2.0".to_string());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().field_name, "timeout");

    // Test with manually constructed wrapped struct
    let wrapped2 = ConfigW {
        timeout: Some(60),
        retries: Some(10),
    };

    // Convert back with skipped field values
    let reconstructed = wrapped2
        .into_original(2222222222, "v4.0".to_string())
        .unwrap();
    assert_eq!(reconstructed.timeout, 60);
    assert_eq!(reconstructed.retries, 10);
    assert_eq!(reconstructed.created_at, 2222222222);
    assert_eq!(reconstructed.version, "v4.0".to_string());
}

#[test]
fn test_wrapped_skip_field_with_bon_builder_pattern() {
    #[derive(Debug, PartialEq, Wrapped, bon::Builder)]
    #[wrapped(name = UserFormW)]
    #[builder(on(Option<String>, into))]
    struct UserForm {
        name: String,
        email: String,
        note: Option<String>,
        #[wrapped(skip)]
        created_at: i64,
        #[wrapped(skip)]
        id: u64,
    }

    let wrapped = UserFormW {
        name: Some("Alice".to_string()),
        email: Some("alice@example.com".to_string()),
        note: Some("hello".to_string()),
    };

    let original = UserForm::builder()
        .from_wrapped(wrapped)
        .unwrap()
        .created_at(1234567890)
        .id(42)
        .build();

    assert_eq!(original.name, "Alice".to_string());
    assert_eq!(original.email, "alice@example.com".to_string());
    assert_eq!(original.note, Some("hello".to_string()));
    assert_eq!(original.created_at, 1234567890);
    assert_eq!(original.id, 42);

    let wrapped_missing = UserFormW {
        name: None,
        email: Some("bob@example.com".to_string()),
        note: None,
    };

    let err = UserForm::builder()
        .from_wrapped(wrapped_missing)
        .err()
        .expect("expected error");
    assert_eq!(err.field_name, "name");
}
