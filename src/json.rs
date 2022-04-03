use serde_json::{json, Map, Value};

pub fn describe_value(value: &Value) -> Value {
    match value {
        Value::Null => json!({ "const": null }),
        Value::Bool(b) => json!({ "const": b }),
        Value::Number(n) => json!({ "const": n }),
        Value::String(s) => json!({ "const": s }),
        Value::Array(v) => {
            json!({"type": "array", "items": v.iter().map(describe_value).collect::<Vec<_>>()})
        }
        Value::Object(m) => {
            let properties = m
                .iter()
                .map(|(k, v)| (k.clone(), describe_value(v).into()))
                .collect::<Map<_, _>>();
            json!({"type": "object", "required": properties.keys().collect::<Vec<_>>(), "properties": properties})
        }
    }
}

#[cfg(test)]
mod tests {
    use super::describe_value;
    use serde_json::json;
    #[test]
    fn null() {
        let schema = describe_value(&json!(null));
        assert!(jsonschema::is_valid(&schema, &json!(null)));
        assert!(!jsonschema::is_valid(&schema, &json!(true)));
    }
    #[test]
    fn bool() {
        let schema = describe_value(&json!(true));
        assert!(jsonschema::is_valid(&schema, &json!(true)));
        assert!(!jsonschema::is_valid(&schema, &json!(false)));
    }
    #[test]
    fn string() {
        let schema = describe_value(&json!("hello"));
        assert!(jsonschema::is_valid(&schema, &json!("hello")));
        assert!(!jsonschema::is_valid(&schema, &json!("world")));
    }
    #[test]
    fn map() {
        let schema = describe_value(&json!({"hello": "world"}));
        println!("{schema:?}");
        assert!(jsonschema::is_valid(&schema, &json!({"hello": "world"})));
        assert!(jsonschema::is_valid(
            &schema,
            &json!({"hello": "world", "foo": "bar"})
        ));
        assert!(!jsonschema::is_valid(&schema, &json!({"hello": "mars"})));
        assert!(!jsonschema::is_valid(&schema, &json!({"'ello": "world"})));
    }
    #[test]
    fn nested_map() {
        let schema = describe_value(&json!({"hello": "world"}));
        println!("{schema:?}");
        assert!(jsonschema::is_valid(&schema, &json!({"hello": "world"})));
        assert!(jsonschema::is_valid(
            &schema,
            &json!({"hello": "world", "foo": "bar"})
        ));
        assert!(!jsonschema::is_valid(&schema, &json!({"hello": "mars"})));
        assert!(!jsonschema::is_valid(&schema, &json!({"'ello": "world"})));
    }
    #[test]
    fn array() {
        let schema = describe_value(&json!(["hello", "world"]));
        assert!(jsonschema::is_valid(&schema, &json!(["hello", "world"])));
        assert!(jsonschema::is_valid(
            &schema,
            &json!(["hello", "world", "and", "mars"])
        ));
        assert!(!jsonschema::is_valid(&schema, &json!(["world", "hello"])));
        assert!(!jsonschema::is_valid(
            &schema,
            &json!(["hello", "hello", "world"])
        ));
    }
}
