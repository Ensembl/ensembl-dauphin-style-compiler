use crate::log;

use super::{eoestructdata::{DataVisitor, eoestack_run}, eoestruct::{StructConst, StructResult}, StructBuilt};

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
    in_struct: bool
}

impl DataVisitor for ProveFalsy {
    fn visit_const(&mut self, input: &StructConst) -> StructResult { 
        let truthy = match input {
            StructConst::Number(n) => *n != 0.,
            StructConst::String(s) => s != "",
            StructConst::Boolean(b) => *b,
            StructConst::Null => false,
        };
        if truthy {
            return Err(String::new());
        }
        Ok(())
    }
    fn visit_separator(&mut self) -> StructResult { Err(String::new()) }
    fn visit_array_start(&mut self) -> StructResult { 
        if self.in_struct {
            return Err(String::new());
        }
        self.in_struct = true;
        Ok(())
    }
    fn visit_array_end(&mut self) -> StructResult { Ok(()) }
    fn visit_object_start(&mut self) -> StructResult {
        if self.in_struct {
            return Err(String::new());
        }
        self.in_struct = true;
        Ok(())
    }
    fn visit_object_end(&mut self) -> StructResult { Ok(()) }
    fn visit_pair_start(&mut self, _key: &str) -> StructResult { Ok(()) }
    fn visit_pair_end(&mut self, _key: &str) -> StructResult { Ok(()) }
}

pub(super) fn truthy(input: &StructBuilt) -> bool {
    let mut falsy = ProveFalsy {
        in_struct: false
    };
    input.expand(None,&mut falsy).is_err()
}

impl StructBuilt {
    pub fn truthy(&self) -> bool { truthy(self) }
}
