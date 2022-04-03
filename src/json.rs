use derive_more::IsVariant;
use regex::Regex;
use std::collections::HashSet;

#[derive(Debug, thiserror::Error)]
pub enum ValueProblem {
    #[error("Value has allowed types {allowed_types:?} but was found to be {actual_type:?}")]
    DisallowedType {
        allowed_types: HashSet<JsonType>,
        actual_type: JsonType,
    },
    #[error("Value was expected to be {expected:?} but was found to be {actual:?}")]
    WrongValue {
        expected: serde_json::Value,
        actual: serde_json::Value,
    },
    #[error("Value {actual:?} doesn't match {regex:?}")]
    NoRegexMatch {
        regex: Regex,
        actual: serde_json::Value,
    },
    #[error("Not array or array member not matched")]
    NoArrayContains,
}

#[derive(Debug, IsVariant)]
pub enum ValueValidator {
    AnyValue,
    Type(HashSet<JsonType>),
    Bool(bool),
    ExactNumber(serde_json::Number),
    NumericRange(serde_json::Number, serde_json::Number),
    ExactString(String),
    RegexString(Regex),
    ExactArray(Vec<serde_json::Value>),
    ArrayContains(Box<Self>),
    ObjectContains(String, Box<Self>),
    ObjectNotContains(String),
    ExactObject(serde_json::Map<String, serde_json::Value>),
}

impl ValueValidator {
    pub fn allows(&self, value: &serde_json::Value) -> Result<(), ValueProblem> {
        use ValueProblem::*;
        use ValueValidator::*;
        match self {
            AnyValue => Ok(()),
            Type(allowed_types) => {
                let jsontype = JsonType::of(value);
                match allowed_types.contains(&jsontype) {
                    true => Ok(()),
                    false => Err(DisallowedType {
                        allowed_types: allowed_types.clone(),
                        actual_type: jsontype,
                    }),
                }
            }
            Bool(expected) => match value {
                serde_json::Value::Bool(actual) if actual == expected => Ok(()),
                _ => Err(WrongValue {
                    expected: serde_json::Value::Bool(expected.clone()),
                    actual: value.clone(),
                }),
            },
            ExactNumber(expected) => match value {
                serde_json::Value::Number(actual) if actual == expected => Ok(()),
                _ => Err(WrongValue {
                    expected: serde_json::Value::Number(expected.clone()),
                    actual: value.clone(),
                }),
            },
            NumericRange(_, _) => todo!(),
            ExactString(expected) => match value {
                serde_json::Value::String(actual) if actual == expected => Ok(()),
                _ => Err(WrongValue {
                    expected: serde_json::Value::String(expected.clone()),
                    actual: value.clone(),
                }),
            },
            RegexString(must_match) => match value {
                serde_json::Value::String(actual) if must_match.is_match(actual) => Ok(()),
                _ => Err(NoRegexMatch {
                    regex: must_match.clone(),
                    actual: value.clone(),
                }),
            },
            ExactArray(expected) => match value {
                serde_json::Value::Array(actual) if actual == expected => Ok(()),
                _ => Err(WrongValue {
                    expected: serde_json::Value::Array(expected.clone()),
                    actual: value.clone(),
                }),
            },
            ArrayContains(expected) => match value {
                serde_json::Value::Array(actual)
                    if actual.iter().any(|a| expected.allows(a).is_ok()) =>
                {
                    Ok(())
                }

                _ => Err(NoArrayContains),
            },
            ObjectContains(expected_key, expected_value) => todo!(),
            ObjectNotContains(_) => todo!(),
            ExactObject(_) => todo!(),
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum JsonType {
    Null,
    Bool,
    Number,
    String,
    Array,
    Object,
}

impl JsonType {
    pub fn of(value: &serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(_) => Self::Bool,
            serde_json::Value::Number(_) => Self::Number,
            serde_json::Value::String(_) => Self::String,
            serde_json::Value::Array(_) => Self::Array,
            serde_json::Value::Object(_) => Self::Object,
        }
    }
}

impl From<ValueProblem> for Result<(), ValueProblem> {
    fn from(value_problem: ValueProblem) -> Self {
        Err(value_problem)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    #[test]
    fn test() -> anyhow::Result<()> {
        let json = json!(null);

        Ok(())
    }
}
