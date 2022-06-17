use super::{eoestruct::{StructConst, Struct}, buildstack::{BuildStack, BuildStackTransformer}, buildertree::BuiltVars};
use serde_json::{Value as JsonValue, Number};

struct JsonTransformer;

impl BuildStackTransformer<StructConst,JsonValue> for JsonTransformer {
    fn make_singleton(&mut self, value: StructConst) -> JsonValue {
        match value {
            StructConst::Number(input) => JsonValue::Number(Number::from_f64(input).unwrap()),
            StructConst::String(input) => JsonValue::String(input),
            StructConst::Boolean(input) => JsonValue::Bool(input),
            StructConst::Null => JsonValue::Null
        }
    }

    fn make_array(&mut self, value: Vec<JsonValue>) -> JsonValue {
        JsonValue::Array(value)
    }

    fn make_object(&mut self, value: Vec<(String,JsonValue)>) -> JsonValue {
        JsonValue::Object(value.iter().map(|x| x.clone()).collect())
    }
}

pub(super) fn expand_to_json(input: Struct<BuiltVars>) -> JsonValue {
    let mut stack = BuildStack::new(JsonTransformer);
    input.expand(&mut stack);
    stack.get()
}
