use std::sync::Arc;
use super::{eoestruct::{StructPair, Struct, VariableSystem, StructConst}, expand::DataVisitor};

pub trait BuildStackTransformer<T,X> {
    fn make_singleton(&mut self, value: T) -> X;
    fn make_array(&mut self, value: Vec<X>) -> X;
    fn make_object(&mut self, value: Vec<(String,X)>) -> X;
}

pub(super) struct IdentityBuildStackTransformer;

impl<T: VariableSystem+Clone> BuildStackTransformer<Struct<T>,Struct<T>> for IdentityBuildStackTransformer {
    fn make_singleton(&mut self, value: Struct<T>) -> Struct<T> { value }
    
    fn make_array(&mut self, value: Vec<Struct<T>>) -> Struct<T> {
        Struct::Array(Arc::new(value))
    }

    fn make_object(&mut self, mut value: Vec<(String,Struct<T>)>) -> Struct<T> {
        Struct::Object(Arc::new(value.drain(..).map(|(k,v)| StructPair(k,v)).collect()))
    }
}

enum TemplateBuildStackEntry<X> {
    Node(Option<X>),
    Array(Vec<X>),
    Object(Vec<(String,X)>)
}

pub(super) struct BuildStack<T,X> {
    stack: Vec<TemplateBuildStackEntry<X>>,
    keys: Vec<String>,
    transformer: Box<dyn BuildStackTransformer<T,X>>
}

impl<T,X> BuildStack<T,X> {
    pub(super) fn new<F>(transformer: F) -> BuildStack<T,X> where F: BuildStackTransformer<T,X> + 'static {
        BuildStack {
            stack: vec![TemplateBuildStackEntry::Node(None)],
            keys: vec![],
            transformer: Box::new(transformer)
        }
    }

    pub(super) fn get(mut self) -> X {
        if let TemplateBuildStackEntry::Node(Some(n) )= self.stack.pop().unwrap() {
            n
        } else {
            panic!("inocorrect stack size");
        }
    }

    pub(super) fn push_array(&mut self) {
        self.stack.push(TemplateBuildStackEntry::Array(vec![]));
    }

    pub(super) fn push_object(&mut self) {
        self.stack.push(TemplateBuildStackEntry::Object(vec![]));
    }

    pub(super) fn push_singleton(&mut self) {
        self.stack.push(TemplateBuildStackEntry::Node(None));
    }

    pub(super) fn push_key(&mut self, key: &str) {
        self.keys.push(key.to_string());
    }

    fn add(&mut self, item: X) {
        match self.stack.last_mut().unwrap() {
            TemplateBuildStackEntry::Array(entries) => {
                entries.push(item);
            },
            TemplateBuildStackEntry::Object(entries) => {
                let key = self.keys.pop().unwrap();
                entries.push((key,item));
            },
            TemplateBuildStackEntry::Node(value) => {
                *value = Some(item);
            }
        }
    }

    pub(super) fn add_atom(&mut self, item: T) {
        let item = self.transformer.make_singleton(item);
        self.add(item);
    }

    pub(super) fn pop<F>(&mut self, cb: F) where F: FnOnce(X) -> X {
        match self.stack.pop().unwrap() {
            TemplateBuildStackEntry::Array(entries) => {
                let item = cb(self.transformer.make_array(entries));
                self.add(item);
            },
            TemplateBuildStackEntry::Object(entries) => {
                let item = cb(self.transformer.make_object(entries));
                self.add(item);
            },
            TemplateBuildStackEntry::Node(node) => {
                self.add(cb(node.expect("unset")));
            }
        };
    }
}

impl<X> DataVisitor for BuildStack<StructConst,X> {
    fn visit_const(&mut self, input: &StructConst) { self.add_atom(input.clone()); }
    fn visit_separator(&mut self) {}
    fn visit_array_start(&mut self) { self.push_array(); }
    fn visit_array_end(&mut self) { self.pop(|x| x); }
    fn visit_object_start(&mut self) { self.push_object(); }
    fn visit_object_end(&mut self) { self.pop(|x| x); }
    fn visit_pair_start(&mut self, key: &str) { self.push_key(key); }
    fn visit_pair_end(&mut self, _key: &str) {}
}
