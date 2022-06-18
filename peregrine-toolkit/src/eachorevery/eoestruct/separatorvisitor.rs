use super::eoestruct::{StructVisitor, VariableSystem, StructConst, StructResult};

pub(super) trait SeparatorVisitor<T: VariableSystem+Clone> : StructVisitor<T> {
    fn visit_separator(&mut self) {}
}

pub(super) struct SeparatedStructAdaptor<'a,T: VariableSystem+Clone> {
    first: Vec<bool>,
    inner: &'a mut dyn SeparatorVisitor<T>
}

impl<'a,T: VariableSystem+Clone> SeparatedStructAdaptor<'a,T> {
    pub fn new(inner: &'a mut dyn SeparatorVisitor<T>) -> SeparatedStructAdaptor<'a,T> {
        SeparatedStructAdaptor { first: vec![true], inner }
    }

    fn sep(&mut self) {
        let first = self.first.last_mut().unwrap();
        if *first {
            *first = false;
        } else {
            self.inner.visit_separator();
        }
    }
}

impl<'a,T: VariableSystem+Clone> StructVisitor<T> for SeparatedStructAdaptor<'a,T> {
    fn visit_const(&mut self, input: &StructConst) -> StructResult {
        self.sep();
        self.inner.visit_const(input)
    }

    fn visit_var(&mut self, input: &T::Use) -> StructResult {
        self.sep();
        self.inner.visit_var(input)
    }

    fn visit_array_start(&mut self) -> StructResult {
        self.sep();
        self.first.push(true);
        self.inner.visit_array_start()
    }

    fn visit_array_end(&mut self) -> StructResult {
        self.first.pop();
        self.inner.visit_array_end()
    }

    fn visit_object_start(&mut self) -> StructResult {
        self.sep();
        self.first.push(true);
        self.inner.visit_object_start()
    }

    fn visit_object_end(&mut self) -> StructResult {
        self.first.pop();
        self.inner.visit_object_end()
    }

    fn visit_pair_start(&mut self, key: &str) -> StructResult {
        self.sep();
        self.first.push(true);
        self.inner.visit_pair_start(key)
    }

    fn visit_pair_end(&mut self, key: &str) -> StructResult {
        self.first.pop();
        self.inner.visit_pair_end(key)
    }

    fn visit_all_start(&mut self, id: &[T::Declare]) -> StructResult {
        self.sep();
        self.first.push(true);
        self.inner.visit_all_start(id)
    }

    fn visit_all_end(&mut self, id: &[T::Declare]) -> StructResult {
        self.first.pop();
        self.inner.visit_all_end(id)
    }
}
