use std::num::ParseFloatError;

use peregrine_config::ConfigError;
use peregrine_data::DataMessage;

use crate::{Message, input::low::modifiers::{KeyboardModifiers, Modifiers}};

use super::modifiers::ModifiersPattern;

fn handle_err(v: &str,e: Result<f64,ParseFloatError>) -> Result<f64,Message> {
    e.map_err(|e| Message::DataError(DataMessage::ConfigError(
        ConfigError::BadConfigValue(format!("mapping parsing: {}",v),e.to_string())
    )))
}

fn parse_args(args: &str) -> Result<Vec<f64>,Message> {
    args.split(",").map(|x| handle_err(x,x.trim().parse::<f64>())).collect()
}

fn parse_final_part(text: &str) -> Result<(Vec<f64>,String),Message> {
    let mut args = vec![];
    let mut text = text.to_string();
    if text.ends_with("]") {
        if let Some(start) = text.rfind("[") {
            if start > 0 {
                args = parse_args(&text[(start+1)..(text.len()-1)])?;
                text = text[0..start].to_string();
            }
        }
    }
    Ok((args,text))
}

fn parse_one_split(spec: &str) -> Vec<String> {
    let mut sections = vec![String::new()];
    let mut in_bracket = 0;
    for (i,c) in spec.chars().enumerate() {
        let last_char = i == spec.len()-1;
        if c == '-' && in_bracket == 0 && !last_char {
            sections.push(String::new());
        } else {
            sections.last_mut().as_mut().unwrap().push(c);
            if c == '(' || c == '[' { in_bracket += 1; }
            if c == ')' { in_bracket -= 1; }
        }
    }
    sections
}

const REQUIRE_START : &str = "require(";
const PROHIBIT_START : &str = "prohibit(";

fn parse_one(spec: &str) -> Result<Option<(String,ModifiersPattern,Vec<f64>)>,Message> {
    let mut sections = parse_one_split(spec);
    let last = sections.pop().unwrap();
    let mut shift = false;
    let mut control = false;
    let mut alt = false;
    let mut require = vec![];
    let mut prohibit = vec![];
    for modifier in sections {
        let modifier = modifier.to_lowercase();
        if modifier == "shift" { shift = true; }
        if modifier == "ctrl" || modifier == "control" { control = true; }
        if modifier == "alt" { alt = true; }
        if modifier.starts_with(&REQUIRE_START.to_lowercase()) && modifier.ends_with(")") {
           require.push(modifier[REQUIRE_START.len()..modifier.len()-1].to_lowercase());
        }
        if modifier.starts_with(&PROHIBIT_START.to_lowercase()) && modifier.ends_with(")") {
            prohibit.push(modifier[PROHIBIT_START.len()..modifier.len()-1].to_lowercase());
         } 
    }
    let required = Modifiers::new(KeyboardModifiers::new(shift,control,alt),&require);
    let prohibited = Modifiers::new(KeyboardModifiers::new(false,false,false),&prohibit);
    let (args,text) = parse_final_part(&last)?;
    Ok(Some((text.to_string(),ModifiersPattern::new(required,prohibited),args)))
}

pub fn parse_keyspec(spec: &str) -> Result<Vec<(String,ModifiersPattern,Vec<f64>)>,Message> {
    spec.split_whitespace().filter_map(|spec| { parse_one(spec).transpose() }).collect::<Result<Vec<_>,_>>()
}
