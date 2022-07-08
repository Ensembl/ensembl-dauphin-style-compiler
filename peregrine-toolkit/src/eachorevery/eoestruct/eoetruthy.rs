use crate::log;

use super::{eoestructdata::{DataVisitor, eoestack_run}, eoestruct::{StructConst, StructResult, struct_error}, StructBuilt};

/* Falsy values are:
 * false, 0, "", [], {}, null
 * Everything else is truthy.
 * 
 * Implementation is to assume false and then mark as thruthy if true thing found:
 * 1. Constant other than the given values is truthy
 * 2. Nested [] or {} is truthy.
 * 3. separator is truthy
 */

struct ProveFalsy {
    once: bool
}

impl ProveFalsy {
    fn once(&mut self) -> StructResult {
        if self.once { return Err(struct_error("")) }
        self.once = true;
        Ok(())
    }
}

impl DataVisitor for ProveFalsy {
    fn visit_const(&mut self, input: &StructConst) -> StructResult { 
        self.once()?;
        let truthy = match input {
            StructConst::Number(n) => *n != 0.,
            StructConst::String(s) => s != "",
            StructConst::Boolean(b) => *b,
            StructConst::Null => false,
        };
        if truthy { Err(struct_error("")) } else { Ok(()) }
    }
    fn visit_array_start(&mut self) -> StructResult { self.once() }
    fn visit_object_start(&mut self) -> StructResult { self.once() }
    fn visit_pair_start(&mut self, _key: &str) -> StructResult { Err(struct_error("")) }
}

pub(super) fn truthy(input: &StructBuilt) -> bool {
    let mut falsy = ProveFalsy { once: false };
    input.expand(None,&mut falsy).is_err()
}

impl StructBuilt {
    pub fn truthy(&self) -> bool { truthy(self) }
}
