use unwrapped::{Unwrapped, Wrapped};

#[test]
fn test_unwrapped_from_defaults() {
    #[derive(Debug, PartialEq, Unwrapped)]
    struct WithDefaults {
        val1: Option<i32>,
        val2: Option<String>,
        val3: String,
        val4: Option<Vec<u8>>,
    }

    let original = WithDefaults {
        val1: None,
        val2: Some("hello".to_string()),
        val3: "world".to_string(),
        val4: None,
    };

    let unwrapped: WithDefaultsUw = original.into();
    assert_eq!(unwrapped.val1, 0);
    assert_eq!(unwrapped.val2, "hello".to_string());
    assert_eq!(unwrapped.val3, "world".to_string());
    assert_eq!(unwrapped.val4, Vec::<u8>::new());

    let converted_back: WithDefaults = unwrapped.into();
    assert_eq!(
        converted_back,
        WithDefaults {
            val1: Some(0),
            val2: Some("hello".to_string()),
            val3: "world".to_string(),
            val4: Some(Vec::new()),
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

    let original1 = Skipped {
        field_a: Some(10),
        field_b: None,
        field_c: true,
    };
    let unwrapped1 = SkippedUw::from(original1);
    assert_eq!(unwrapped1.field_a, 10);
    assert_eq!(unwrapped1.field_b, None);
    assert_eq!(unwrapped1.field_c, true);

    let original2 = Skipped {
        field_a: None,
        field_b: None,
        field_c: false,
    };
    let unwrapped2 = SkippedUw::from(original2);
    assert_eq!(unwrapped2.field_a, 0);
    assert_eq!(unwrapped2.field_b, None);
    assert_eq!(unwrapped2.field_c, false);

    let unwrapped3 = SkippedUw {
        field_a: 99,
        field_b: None,
        field_c: true,
    };
    let original3 = Skipped::from(unwrapped3);
    assert_eq!(
        original3,
        Skipped {
            field_a: Some(99),
            field_b: None,
            field_c: true,
        }
    );

    let original4 = Skipped {
        field_a: Some(123),
        field_b: None,
        field_c: false,
    };
    let unwrapped4_res = SkippedUw::try_from(original4);
    assert!(unwrapped4_res.is_ok());
    let unwrapped4 = unwrapped4_res.unwrap();
    assert_eq!(unwrapped4.field_a, 123);
    assert_eq!(unwrapped4.field_b, None);
    assert_eq!(unwrapped4.field_c, false);

    let original5 = Skipped {
        field_a: None,
        field_b: Some("This should fail".to_string()),
        field_c: true,
    };
    let unwrapped5_res = SkippedUw::try_from(original5);
    assert!(unwrapped5_res.is_err());
    match unwrapped5_res {
        Err(e) => assert_eq!(e.field_name, "field_a"),
        Ok(_) => panic!("Expected error"),
    }
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
    let wrapped: SimpleW = original.into();
    assert_eq!(wrapped.field1, Some(10));
    assert_eq!(wrapped.field2, Some("hello".to_string()));
    assert_eq!(wrapped.field3, Some(100)); // Already Option, stays as-is

    // Convert back - unwrap with defaults
    let converted_back: Simple = wrapped.into();
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
        required: None,
        optional: Some("value".to_string()),
    };

    // Conversion from wrapped with None uses default
    let converted: WithOptionals = wrapped.into();
    assert_eq!(converted.required, 0); // Default i32
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

    let wrapped: GenericW<bool> = original.into();
    assert_eq!(wrapped.value, Some(true));
    assert_eq!(wrapped.id, Some(123));

    let converted_back: Generic<bool> = wrapped.into();
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

    let original = Skipped {
        field_a: 10,
        field_b: None,
        field_c: true,
    };

    let wrapped = SkippedW::from(original);
    assert_eq!(wrapped.field_a, Some(10));
    assert_eq!(wrapped.field_b, None); // Skipped, stays as-is
    assert_eq!(wrapped.field_c, Some(true));

    let converted_back: Skipped = wrapped.into();
    assert_eq!(converted_back.field_a, 10);
    assert_eq!(converted_back.field_b, None);
    assert_eq!(converted_back.field_c, true);
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
