use super::{eoestruct::{VariableSystem, Struct, StructVisitor, StructConst}, separatorvisitor::{SeparatedStructAdaptor, SeparatorVisitor}};

#[cfg(debug_assertions)]
pub trait VariableSystemFormatter<T: VariableSystem> {
    fn format_declare_start(&mut self, var: &[T::Declare]) -> String;
    fn format_declare_end(&mut self, var: &[T::Declare]) -> String;
    fn format_use(&mut self, var: &T::Use) -> String;
}

#[cfg(debug_assertions)]
impl<T: VariableSystem+Clone> std::fmt::Debug for Struct<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",StructDebug::format(self))
    }
}

#[cfg(debug_assertions)]
pub struct StructDebug<T: VariableSystem> {
    output: String,
    formatter: Box<dyn VariableSystemFormatter<T>>
}

#[cfg(debug_assertions)]
impl<T: VariableSystem+Clone> StructDebug<T> {
    pub(super) fn format(input: &Struct<T>) -> String {
        let mut visitor = StructDebug::new();
        input.visit(&mut visitor.visitor());
        visitor.out()
    }

    pub(super) fn new() -> StructDebug<T> {
        StructDebug {
            output: String::new(),
            formatter: T::build_formatter()
        }
    }

    pub(super) fn visitor(&mut self) -> SeparatedStructAdaptor<T> {
        SeparatedStructAdaptor::new(self)
    }

    fn add(&mut self, value: &str) {
        self.output.push_str(value);
    }

    pub(super) fn out(self) -> String { self.output }
}

#[cfg(debug_assertions)]
impl<T: VariableSystem+Clone> SeparatorVisitor<T> for StructDebug<T> {
    fn visit_separator(&mut self) { self.add(","); }
}

#[cfg(debug_assertions)]
impl<T: VariableSystem+Clone> StructVisitor<T> for StructDebug<T> {
    fn visit_const(&mut self, input: &StructConst) {
        self.add(&match input {
            StructConst::Number(value) => format!("{:?}",value),
            StructConst::String(value) => format!("{:?}",value),
            StructConst::Boolean(value) => format!("{:?}",value),
            StructConst::Null => format!("null")
        });
    }

    fn visit_array_start(&mut self) { self.add("["); }
    fn visit_array_end(&mut self) { self.add("]"); }
    fn visit_object_start(&mut self) { self.add("{"); }
    fn visit_object_end(&mut self) { self.add("}"); }
    fn visit_pair_start(&mut self, key: &str) { self.add(&format!("{:?}: ",key)); }
    fn visit_pair_end(&mut self, _key: &str) {}

    fn visit_var(&mut self, input: &T::Use) {
        let value = self.formatter.format_use(input);
        self.add(&value);
    }

    fn visit_all_start(&mut self, ids: &[T::Declare]) {
        let value = self.formatter.format_declare_start(ids);
        self.add(&value);
    }

    fn visit_all_end(&mut self, ids: &[T::Declare]) {
        let value = self.formatter.format_declare_end(ids);
        self.add(&value);
    }
}
