use crate::eachorevery::{EachOrEvery, EachOrEveryGroupCompatible};
use identitynumber::{ identitynumber };
use lazy_static::lazy_static;

identitynumber!(IDS);

#[cfg(debug_assertions)]
pub type StructError = String;

#[cfg(debug_assertions)]
pub(super) fn struct_error(msg: &str) -> StructError { msg.to_string() }

#[cfg(not(debug_assertions))]
pub type StructError = ();

#[cfg(not(debug_assertions))]
pub(super) fn struct_error(msg: &str) -> StructError { () }

pub type StructResult = Result<(),StructError>;

#[derive(Copy,Clone,PartialEq,Eq,Hash)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct StructValueId(pub(super) u64);

impl StructValueId {
    pub(super) fn new() -> StructValueId { StructValueId(IDS.next()) }
}

pub struct StructVarGroup(pub(super) Vec<StructValueId>);

impl StructVarGroup {
    pub fn new() -> StructVarGroup { StructVarGroup(vec![]) }
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub enum StructConst {
    Number(f64),
    String(String),
    Boolean(bool),
    Null
}

impl StructConst {
    pub(super) fn truthy(&self) -> bool {
        match self {
            StructConst::Number(_) => true,
            StructConst::String(_) => true,
            StructConst::Boolean(b) => *b,
            StructConst::Null => false
        }
    }
}

#[derive(Clone)]
/* Guarantee: all EachOrEverys in here will be Each after construction */
pub enum StructVarValue {
    Number(EachOrEvery<f64>),
    String(EachOrEvery<String>),
    Boolean(EachOrEvery<bool>),
}

fn to_const<X>(input: &EachOrEvery<X>) -> Option<&X> {
    if input.len().is_none() {
        Some(input.get(0).unwrap()) // EoE every is guaranteed to be Some
    } else {
        None
    }
}
 
fn format<X: std::fmt::Debug>(value: &EachOrEvery<X>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if let Some(len) = value.len() {
        let mut sep = false;
        write!(f,"<")?;
        for value in value.iter(len).unwrap() { // guaranteed by outer conditional
            if sep { write!(f,",")?; }
            write!(f,"{:?}",value)?;
            sep = true;
        }
        write!(f,">")?;
    } else {
        let value = value.iter(1).unwrap().next().unwrap(); // EoE every is guaranteed to be Some
        write!(f,"{:?}",value)?;
    }
    Ok(())
}

impl std::fmt::Debug for StructVarValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StructVarValue::Number(x) => format(x,f),
            StructVarValue::String(x) => format(x,f),
            StructVarValue::Boolean(x) => format(x,f)
        }
    }
}

impl StructVarValue {
    pub(super) fn to_const(&self) -> Option<StructConst> {
        match self {
            StructVarValue::Number(input) => {
                to_const(input).map(|x| StructConst::Number(*x))
            },
            StructVarValue::String(input) => {
                to_const(input).map(|x| StructConst::String(x.clone()))
            },
            StructVarValue::Boolean(input) => {
                to_const(input).map(|x| StructConst::Boolean(*x))
            }
        }
    }

    pub(super) fn is_finite(&self) -> bool {
        match self {
            StructVarValue::Number(x) => x.len().is_some(),
            StructVarValue::String(x) => x.len().is_some(),
            StructVarValue::Boolean(x) => x.len().is_some(),
        }
    }

    pub(super) fn check_compatible(&self, compat: &mut EachOrEveryGroupCompatible) {
        match self {
            StructVarValue::Number(input) => compat.add(input),
            StructVarValue::String(input) => compat.add(input),
            StructVarValue::Boolean(input) => compat.add(input)
        };
    }

    pub(super) fn get<'a>(&'a self, index: usize) -> StructConst {
        match self {
            StructVarValue::Number(input) => {
                StructConst::Number(*input.get(index).unwrap())
            },
            StructVarValue::String(input) => {
                StructConst::String(input.get(index).unwrap().clone())
            },
            StructVarValue::Boolean(input) => {
                StructConst::Boolean(*input.get(index).unwrap())
            }
        }
    }

    pub(super) fn exists<'a>(&'a self, index: usize) -> bool {
        match self {
            StructVarValue::Number(input) => {
                input.get(index).is_some()
            },
            StructVarValue::String(input) => {
                input.get(index).is_some()
            },
            StructVarValue::Boolean(input) => {
                input.get(index).is_some()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;
    use crate::{eachorevery::{eoestruct::{eoejson::{struct_to_json, struct_from_json}, structtemplate::{StructVar, StructPair}, StructTemplate, eoestructdata::{DataVisitor}}, EachOrEvery}};
    use serde_json::{Value as JsonValue, Number};

    use super::{StructResult, StructConst, StructVarGroup};

    fn json_fix_numbers(json: &JsonValue) -> JsonValue {
        match json {
            JsonValue::Null => JsonValue::Null,
            JsonValue::Bool(x) => JsonValue::Bool(*x),
            JsonValue::Number(n) => JsonValue::Number(Number::from_f64(n.as_f64().unwrap()).unwrap()),
            JsonValue::String(s) => JsonValue::String(s.to_string()),
            JsonValue::Array(x) => JsonValue::Array(x.iter().map(|x| json_fix_numbers(x)).collect()),
            JsonValue::Object(x) => JsonValue::Object(x.iter().map(|(k,v)| (k.to_string(),json_fix_numbers(v))).collect()),
        }
    }

    macro_rules! json_get {
        ($name:ident,$var:tt,$typ:ty) => {
            fn $name(value: &JsonValue) -> $typ {
                match value {
                    JsonValue::$var(v) => v.clone(),
                    _ => panic!("malformatted test data")
                }
            }
                    
        };
    }

    json_get!(json_array,Array,Vec<JsonValue>);
    json_get!(json_string,String,String);

    fn run_case(value: &JsonValue) {
        let parts = json_array(value);
        println!("ruuning {}\n",json_string(&parts[0]));
        let vars = json_array(&parts[1]).iter().map(|x| json_string(x)).collect::<Vec<_>>();
        let ifs = json_array(&parts[2]).iter().map(|x| json_string(x)).collect::<Vec<_>>();
        let template = struct_from_json(vars,ifs,&parts[3]).ok().unwrap();
        let debug = format!("{:?}",template);
        if !parts[4].is_null() {
            assert_eq!(debug,json_string(&parts[4]));
        }
        println!("{:?}\n",template);
        println!("{:?}\n",template.build());
        let output = struct_to_json(&template.build().ok().expect("unexpected error")).ok().unwrap();
        let output = JsonValue::from_str(&output.to_string()).ok().unwrap();
        assert_eq!(json_fix_numbers(&output),json_fix_numbers(&parts[5]));
        println!("{:?}\n",json_fix_numbers(&output));
    }

    fn run_case_buildfail(value: &JsonValue) {
        let parts = json_array(value);
        println!("ruuning {}\n",json_string(&parts[0]));
        let vars = json_array(&parts[1]).iter().map(|x| json_string(x)).collect::<Vec<_>>();
        let ifs = json_array(&parts[2]).iter().map(|x| json_string(x)).collect::<Vec<_>>();
        let template = struct_from_json(vars,ifs,&parts[3]).ok().unwrap();
        match template.build() {
            Ok(r) => { eprintln!("unexpected success: {:?}",r); assert!(false); },
            Err(e) => assert_eq!(e,json_string(&parts[4]))
        }
    }

    fn run_case_parsefail(value: &JsonValue) {
        let parts = json_array(value);
        println!("ruuning {}\n",json_string(&parts[0]));
        let vars = json_array(&parts[1]).iter().map(|x| json_string(x)).collect::<Vec<_>>();
        let ifs = json_array(&parts[2]).iter().map(|x| json_string(x)).collect::<Vec<_>>();
        match struct_from_json(vars,ifs,&parts[3]) {
            Ok(r) => { eprintln!("unexpected success: {:?}",r); assert!(false); },
            Err(e) => assert_eq!(e,json_string(&parts[4]))
        }
    }

    #[test]
    fn test_eoestruct_smoke() {
        let data = JsonValue::from_str(include_str!("test-eoe-smoke.json")).ok().unwrap();
        for testcase in json_array(&data).iter() {
            run_case(&testcase);
        }
    } 

    #[test]
    fn test_eoestruct_buildfail() {
        let data = JsonValue::from_str(include_str!("test-eoe-buildfail.json")).ok().unwrap();
        for testcase in json_array(&data).iter() {
            run_case_buildfail(&testcase);
        }
    } 

    #[test]
    fn test_eoestruct_parsefail() {
        let data = JsonValue::from_str(include_str!("test-eoe-parsefail.json")).ok().unwrap();
        for testcase in json_array(&data).iter() {
            run_case_parsefail(&testcase);
        }
    }

    #[test]
    fn test_eoestruct_free() {
        /* corner case not testable with the available harnesses */
        let mut group = StructVarGroup::new();
        let template = StructTemplate::new_array(EachOrEvery::each(vec![
            StructTemplate::new_boolean(true),
            StructTemplate::new_var(&StructVar::new_boolean(&mut group,EachOrEvery::each(vec![false,true])))
        ]));
        match template.build() {
            Ok(r) => { eprintln!("unexpected success: {:?}",r); assert!(false); },
            Err(e) => assert_eq!(e,"free variable in template")
        }
    }

    #[test]
    fn test_eoestruct_every() {
        let mut group = StructVarGroup::new();
        let every = StructVar::new_boolean(&mut group,EachOrEvery::every(false));
        let each = StructVar::new_number(&mut group,EachOrEvery::each(vec![1.,2.]));
        let template = StructTemplate::new_all(&mut group,
        StructTemplate::new_array(EachOrEvery::each(vec![
            StructTemplate::new_boolean(true),
            StructTemplate::new_var(&every),
            StructTemplate::new_var(&each)
        ]))
        );
        let debug = format!("{:?}",template);
        assert_eq!("Aab.( [true,false,b=<1.0,2.0>] )",debug);
        let output = struct_to_json(&template.build().ok().expect("unexpected error")).ok().unwrap();
        let wanted = JsonValue::from_str("[[true,false,1],[true,false,2]]").ok().unwrap();
        assert_eq!(&json_fix_numbers(&wanted),&json_fix_numbers(&output));
    }

    #[test]
    fn test_infinite_array() {
        let template = StructTemplate::new_object(EachOrEvery::each(vec![
            StructPair::new("a",StructTemplate::new_number(42.)),
            StructPair::new("b",StructTemplate::new_array(EachOrEvery::every(StructTemplate::new_number(77.))))
        ]));
        match template.build() {
            Ok(r) => { eprintln!("unexpected success: {:?}",r); assert!(false); },
            Err(e) => assert_eq!(e,"no infinite arrays in json")
        }
    }

    #[test]
    fn test_infinite_object() {
        let template = StructTemplate::new_object(EachOrEvery::every(
            StructPair::new("a",StructTemplate::new_number(42.)),
        ));
        match template.build() {
            Ok(r) => { eprintln!("unexpected success: {:?}",r); assert!(false); },
            Err(e) => assert_eq!(e,"no infinite objects in json")
        }
    }

    #[test]
    fn test_infinite_all() {
        // XXX reenable when late-binding implemented
        /*
        let mut group = StructVarGroup::new();
        let var = StructVar::new_boolean(&mut group,EachOrEvery::every(false));
        let template = StructTemplate::new_all(&mut group,
            StructTemplate::new_var(&var)
        );
        let output = struct_to_json(&template.build().ok().expect("unexpected error")).err().unwrap();
        assert_eq!("no infinite recursion allowed",output);
        */
    }

    #[test]
    fn test_eoe_smoke_array() {
        let pattern = vec![0,1,2,3,1,2,3,1,2,1];
        let start = EachOrEvery::each(pattern.clone()).index(|x| *x);
        let options = vec![
            StructTemplate::new_number(0.),
            StructTemplate::new_string("1".to_string()),
            StructTemplate::new_boolean(true),
            StructTemplate::new_null(),
        ];
        let output_options = vec![
            JsonValue::Number(Number::from_f64(0.).unwrap()),
            JsonValue::String("1".to_string()),
            JsonValue::Bool(true),
            JsonValue::Null
        ];
        let cmp = JsonValue::Array(
            pattern.iter().map(|x| output_options[*x].clone()).collect::<Vec<_>>()
        );
        let template = StructTemplate::new_array(start.map(|x| { options[*x].clone() }));
        let output = struct_to_json(&template.build().ok().expect("unexpected error")).ok().unwrap();
        println!("{}",output.to_string());
        assert_eq!(json_fix_numbers(&output),json_fix_numbers(&cmp));
    }

    #[test]
    fn test_eoestruct_notopcond() {
        let mut group = StructVarGroup::new();
        let template = StructTemplate::new_condition(StructVar::new_boolean(&mut group,EachOrEvery::each(vec![true])),
            StructTemplate::new_number(42.)
        );
        match template.build() {
            Ok(r) => { eprintln!("unexpected success: {:?}",r); assert!(false); },
            Err(e) => assert_eq!(e,"conditionals banned at top level")
        }
    }

    struct TestVisitor(String);

    impl DataVisitor for TestVisitor {
        fn visit_const(&mut self, _input: &StructConst) -> StructResult { self.0.push('c'); Ok(()) }
        fn visit_separator(&mut self) -> StructResult { self.0.push(','); Ok(())}
        fn visit_array_start(&mut self) -> StructResult { self.0.push('['); Ok(()) }
        fn visit_array_end(&mut self) -> StructResult { self.0.push(']'); Ok(()) }
        fn visit_object_start(&mut self) -> StructResult { self.0.push('{'); Ok(()) }
        fn visit_object_end(&mut self) -> StructResult { self.0.push('}'); Ok(()) }
        fn visit_pair_start(&mut self, key: &str) -> StructResult { self.0.push_str(&format!("<{}>",key)); Ok(()) }
        fn visit_pair_end(&mut self, key: &str) -> StructResult { self.0.push_str(&format!("</{}>",key)); Ok(()) }
    }

    fn visitor_case(value: &JsonValue) {
        let parts = json_array(value);
        println!("ruuning {}\n",json_string(&parts[0]));
        let vars = json_array(&parts[1]).iter().map(|x| json_string(x)).collect::<Vec<_>>();
        let ifs = json_array(&parts[2]).iter().map(|x| json_string(x)).collect::<Vec<_>>();
        let template = struct_from_json(vars,ifs,&parts[3]).ok().unwrap();
        let debug = format!("{:?}",template);
        if !parts[4].is_null() {
            assert_eq!(debug,json_string(&parts[4]));
        }
        println!("{:?}\n",template);
        println!("{:?}\n",template.build());
        let mut visitor = TestVisitor(String::new());
        template.build().ok().expect("unexpected error").expand(&mut visitor).ok().expect("visitor failed");
        println!("{:?}",visitor.0);
        assert_eq!(&parts[5],&visitor.0)
    }

    #[test]
    fn test_eoestruct_visitor() {
        let data = JsonValue::from_str(include_str!("test-visitor.json")).ok().unwrap();
        for testcase in json_array(&data).iter() {
            visitor_case(&testcase);
        }

    }

}
