use prost::Message;
use prost_reflect::bytes::Bytes;
use prost_reflect::{DescriptorPool, DynamicMessage, ReflectMessage, Value};
use std::fs;
use std::path::Path;

/// Helper to load the descriptor set for test_all_types.proto
fn load_descriptor_pool() -> DescriptorPool {
    let descriptor_bytes = fs::read("src/descriptor_set.bin")
        .expect("Failed to read descriptor_set.bin (did you build the crate?)");
    let file_descriptor_set = prost_types::FileDescriptorSet::decode(&*descriptor_bytes)
        .expect("Failed to decode descriptor set");
    DescriptorPool::from_file_descriptor_set(file_descriptor_set)
        .expect("Failed to build DescriptorPool")
}

#[test]
fn test_reflect_all_messages_and_fields() {
    let pool = load_descriptor_pool();

    // Check that all expected messages are present
    let expected_messages = [
        "testall.ScalarTypes",
        "testall.EnumAndRepeated",
        "testall.Outer",
        "testall.Outer.Inner",
        "testall.OptionalFields",
        "testall.Defaults",
        "testall.Everything",
    ];

    for msg in &expected_messages {
        assert!(
            pool.get_message_by_name(msg).is_some(),
            "Message {} not found in descriptor pool",
            msg
        );
    }

    // Check that all expected enums are present
    let expected_enums = ["testall.TestEnum"];
    for en in &expected_enums {
        assert!(
            pool.get_enum_by_name(en).is_some(),
            "Enum {} not found in descriptor pool",
            en
        );
    }

    // Check fields for ScalarTypes
    let scalar = pool.get_message_by_name("testall.ScalarTypes").unwrap();
    let expected_fields = [
        "str_field",
        "int32_field",
        "int64_field",
        "uint32_field",
        "uint64_field",
        "sint32_field",
        "sint64_field",
        "fixed32_field",
        "fixed64_field",
        "sfixed32_field",
        "sfixed64_field",
        "bool_field",
        "bytes_field",
        "float_field",
        "double_field",
    ];
    for field in &expected_fields {
        assert!(
            scalar.get_field_by_name(field).is_some(),
            "Field {} not found in ScalarTypes",
            field
        );
    }
}

#[test]
fn test_round_trip_everything() {
    let pool = load_descriptor_pool();
    let everything_desc = pool
        .get_message_by_name("testall.Everything")
        .expect("Everything message not found");

    // Construct a DynamicMessage with all fields set
    let mut everything = DynamicMessage::new(everything_desc.clone());

    // Set scalars
    let scalars_desc = pool.get_message_by_name("testall.ScalarTypes").unwrap();
    let mut scalars = DynamicMessage::new(scalars_desc.clone());
    scalars.set_field_by_name("str_field", Value::String("hello".to_string()));
    scalars.set_field_by_name("int32_field", Value::I32(42));
    scalars.set_field_by_name("bool_field", Value::Bool(true));
    scalars.set_field_by_name("float_field", Value::F32(3.14));
    scalars.set_field_by_name("double_field", Value::F64(2.71828));

    // Set enums and repeated
    let enums_rep_desc = pool.get_message_by_name("testall.EnumAndRepeated").unwrap();
    let mut enums_rep = DynamicMessage::new(enums_rep_desc.clone());
    let enum_type = pool.get_enum_by_name("testall.TestEnum").unwrap();
    enums_rep.set_field_by_name(
        "enum_field",
        Value::EnumNumber(enum_type.get_value_by_name("SECOND").unwrap().number()),
    );
    enums_rep.set_field_by_name(
        "repeated_enum",
        Value::List(vec![
            Value::EnumNumber(enum_type.get_value_by_name("FIRST").unwrap().number()),
            Value::EnumNumber(enum_type.get_value_by_name("THIRD").unwrap().number()),
        ]),
    );
    enums_rep.set_field_by_name(
        "repeated_int32",
        Value::List(vec![Value::I32(1), Value::I32(2), Value::I32(3)]),
    );
    enums_rep.set_field_by_name(
        "repeated_string",
        Value::List(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
        ]),
    );

    // Set nested
    let inner_desc = pool.get_message_by_name("testall.Outer.Inner").unwrap();
    let mut inner = DynamicMessage::new(inner_desc.clone());
    inner.set_field_by_name("inner_field", Value::I32(99));
    inner.set_field_by_name("inner_str", Value::String("nested".to_string()));

    // Set repeated nested
    let mut inner2 = DynamicMessage::new(inner_desc.clone());
    inner2.set_field_by_name("inner_field", Value::I32(100));
    inner2.set_field_by_name("inner_str", Value::String("nested2".to_string()));

    // Set optional
    let opt_desc = pool.get_message_by_name("testall.OptionalFields").unwrap();
    let mut opt = DynamicMessage::new(opt_desc.clone());
    opt.set_field_by_name("opt_str", Value::String("maybe".to_string()));
    opt.set_field_by_name("opt_int32", Value::I32(123));
    opt.set_field_by_name("opt_bool", Value::Bool(true));

    // Set defaults
    let def_desc = pool.get_message_by_name("testall.Defaults").unwrap();
    let mut def = DynamicMessage::new(def_desc.clone());
    def.set_field_by_name("str_field", Value::String("default".to_string()));
    def.set_field_by_name("int32_field", Value::I32(0));
    def.set_field_by_name("bool_field", Value::Bool(false));
    def.set_field_by_name(
        "enum_field",
        Value::EnumNumber(enum_type.get_value_by_name("UNKNOWN").unwrap().number()),
    );

    // Set Everything fields
    everything.set_field_by_name("scalars", Value::Message(scalars));
    everything.set_field_by_name("enums_and_repeated", Value::Message(enums_rep));
    everything.set_field_by_name("nested", Value::Message(inner.clone()));
    everything.set_field_by_name(
        "repeated_nested",
        Value::List(vec![Value::Message(inner), Value::Message(inner2)]),
    );
    everything.set_field_by_name("optional", Value::Message(opt));
    everything.set_field_by_name("defaults", Value::Message(def));
    everything.set_field_by_name("raw_bytes", Value::Bytes(Bytes::from(b"raw".to_vec())));
    everything.set_field_by_name(
        "repeated_bytes",
        Value::List(vec![
            Value::Bytes(Bytes::from(b"foo".to_vec())),
            Value::Bytes(Bytes::from(b"bar".to_vec())),
        ]),
    );
    everything.set_field_by_name(
        "enum_field",
        Value::EnumNumber(enum_type.get_value_by_name("SECOND").unwrap().number()),
    );
    everything.set_field_by_name(
        "repeated_enum",
        Value::List(vec![
            Value::EnumNumber(enum_type.get_value_by_name("FIRST").unwrap().number()),
            Value::EnumNumber(enum_type.get_value_by_name("THIRD").unwrap().number()),
        ]),
    );

    // Encode to bytes
    let bytes = everything.encode_to_vec();

    // Decode back
    let decoded =
        DynamicMessage::decode(everything_desc, &*bytes).expect("Failed to decode Everything");

    // Compare recursively by field values
    fn compare_dynamic_messages(a: &DynamicMessage, b: &DynamicMessage) {
        let desc = a.descriptor();
        assert_eq!(
            desc.full_name(),
            b.descriptor().full_name(),
            "Message types differ"
        );
        for field in desc.fields() {
            let va = a.get_field(&field);
            let vb = b.get_field(&field);
            compare_values(&va, &vb, &field);
        }
    }

    fn compare_values(
        a: &std::borrow::Cow<'_, Value>,
        b: &std::borrow::Cow<'_, Value>,
        field: &prost_reflect::FieldDescriptor,
    ) {
        use prost_reflect::Value;
        match (a.as_ref(), b.as_ref()) {
            (Value::Bool(x), Value::Bool(y)) => assert_eq!(x, y, "Field {} mismatch", field.name()),
            (Value::I32(x), Value::I32(y)) => assert_eq!(x, y, "Field {} mismatch", field.name()),
            (Value::I64(x), Value::I64(y)) => assert_eq!(x, y, "Field {} mismatch", field.name()),
            (Value::U32(x), Value::U32(y)) => assert_eq!(x, y, "Field {} mismatch", field.name()),
            (Value::U64(x), Value::U64(y)) => assert_eq!(x, y, "Field {} mismatch", field.name()),
            (Value::F32(x), Value::F32(y)) => {
                assert!((x - y).abs() < 1e-6, "Field {} mismatch", field.name())
            }
            (Value::F64(x), Value::F64(y)) => {
                assert!((x - y).abs() < 1e-12, "Field {} mismatch", field.name())
            }
            (Value::String(x), Value::String(y)) => {
                assert_eq!(x, y, "Field {} mismatch", field.name())
            }
            (Value::Bytes(x), Value::Bytes(y)) => {
                assert_eq!(x, y, "Field {} mismatch", field.name())
            }
            (Value::EnumNumber(x), Value::EnumNumber(y)) => {
                assert_eq!(x, y, "Field {} mismatch", field.name())
            }
            (Value::Message(x), Value::Message(y)) => compare_dynamic_messages(x, y),
            (Value::List(xs), Value::List(ys)) => {
                assert_eq!(
                    xs.len(),
                    ys.len(),
                    "Field {} list length mismatch",
                    field.name()
                );
                for (vx, vy) in xs.iter().zip(ys.iter()) {
                    compare_values(
                        &std::borrow::Cow::Borrowed(vx),
                        &std::borrow::Cow::Borrowed(vy),
                        field,
                    );
                }
            }
            _ => panic!(
                "Field {} type/value mismatch: {:?} vs {:?}",
                field.name(),
                a,
                b
            ),
        }
    }

    compare_dynamic_messages(&everything, &decoded);
}
