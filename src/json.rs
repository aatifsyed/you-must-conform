use std::collections::HashSet;

use maplit::hashset;
use regex::Regex;
use strum::IntoEnumIterator;

#[derive(Debug, thiserror::Error)]
pub enum NodeProblem {
    #[error("Node has restriction {type_restriction:?} but was found to be {actual_type:?}")]
    TypeRestrictionFailed {
        type_restriction: TypeRestriction,
        actual_type: JsonType,
    },
}

pub enum ValueRestriction {
    Bool(bool),
    ExactNumber(serde_json::Number),
    NumericRange(serde_json::Number, serde_json::Number),
    ExactString(String),
    Regex(Regex),
    Key(String, Box<NodeRestriction>),
}

impl ValueRestriction {
    pub fn allows(&self, value: &serde_json::Value) {
        match value {
            serde_json::Value::Null => todo!(),
            serde_json::Value::Bool(_) => todo!(),
            serde_json::Value::Number(_) => todo!(),
            serde_json::Value::String(_) => todo!(),
            serde_json::Value::Array(_) => todo!(),
            serde_json::Value::Object(_) => todo!(),
        }
    }
}

#[derive(Debug)]
pub enum TypeRestriction {
    IsNull,
    NonNull,
    IsBool,
    NotBool,
    IsNumber,
    NotNumber,
    IsString,
    NotString,
    IsArray,
    NotArray,
    IsObject,
    NotObject,
}

impl TypeRestriction {
    pub fn allows(&self, json_type: JsonType) -> bool {
        let mut allowed = HashSet::new();
        allowed.extend(JsonType::iter());

        match self {
            TypeRestriction::IsNull => allowed = hashset!(JsonType::Null),
            TypeRestriction::NonNull => {
                allowed.remove(&JsonType::Null);
            }
            TypeRestriction::IsBool => allowed = hashset!(JsonType::Bool),
            TypeRestriction::NotBool => {
                allowed.remove(&JsonType::Bool);
            }
            TypeRestriction::IsNumber => allowed = hashset!(JsonType::Number),
            TypeRestriction::NotNumber => {
                allowed.remove(&JsonType::Number);
            }
            TypeRestriction::IsString => allowed = hashset!(JsonType::String),
            TypeRestriction::NotString => {
                allowed.remove(&JsonType::String);
            }
            TypeRestriction::IsArray => allowed = hashset!(JsonType::Array),
            TypeRestriction::NotArray => {
                allowed.remove(&JsonType::Array);
            }
            TypeRestriction::IsObject => allowed = hashset!(JsonType::Object),
            TypeRestriction::NotObject => {
                allowed.remove(&JsonType::Object);
            }
        }
        allowed.contains(&json_type)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, strum::EnumIter, Clone, Copy)]
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

pub enum NodeRestriction {
    TypeRestriction(TypeRestriction),
    ValueRestriction(ValueRestriction),
}

pub trait NodeValidate<T> {
    fn validate(self, value: T) -> Vec<NodeProblem>;
}

impl NodeValidate<serde_json::Value> for NodeRestriction {
    fn validate(self, value: serde_json::Value) -> Vec<NodeProblem> {
        use NodeProblem::*;
        let mut problems = Vec::new();

        match self {
            NodeRestriction::TypeRestriction(type_restriction) => {
                let actual_type = JsonType::of(&value);
                if !type_restriction.allows(actual_type) {
                    problems.push(TypeRestrictionFailed {
                        type_restriction,
                        actual_type,
                    })
                }
            }
            NodeRestriction::ValueRestriction(value_restriction) => todo!(),
        }
        problems
    }
}
