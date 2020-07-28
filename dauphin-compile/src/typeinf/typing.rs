/* 
 *  See the NOTICE file distributed with this work for additional information
 *  regarding copyright ownership.
 *  
 *  Licensed under the Apache License, Version 2.0 (the "License"); you may 
 *  not use this file except in compliance with the License. You may obtain a
 *  copy of the License at http://www.apache.org/licenses/LICENSE-2.0
 *  
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS, WITHOUT 
 *  WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 */

use std::fmt;
use std::iter::FromIterator;
use std::collections::{ HashMap, HashSet };

use super::types::{ InstructionConstraint, ExpressionType };
use dauphin_interp::runtime::{ Register };
use dauphin_interp::types::{ BaseType };
use super::typesinternal::{ Key, TypeConstraint };
use super::typestore::TypeStore;

pub struct Typing {
    next: usize,
    store: TypeStore,
    regmap: HashMap<Register,usize>,
    reg_isref: HashSet<Register>
}

impl fmt::Debug for Typing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut map : Vec<(Register,usize)> = self.regmap.iter().map(|(k,v)| (k.clone(),v.clone())).collect();
        map.sort();
        for (reg,reg_id) in &map {
            write!(f,"{:?} = ",reg)?;
            if self.reg_isref.contains(reg) { write!(f,"ref(")?; }
            let type_ = self.store.get(&Key::External(*reg_id)).unwrap();
            write!(f,"{:?}",type_)?;
            if self.reg_isref.contains(reg) { write!(f,")")?; }
            write!(f,"\n")?;
        }
        Ok(())
    }
}


impl Typing {
    pub fn new() -> Typing {
        Typing {
            next: 0,
            store: TypeStore::new(),
            regmap: HashMap::new(),
            reg_isref: HashSet::new()
        }
    }

    fn extract(&mut self, in_: &InstructionConstraint) -> Vec<(TypeConstraint,Register)> {
        let mut out = Vec::new();
        let mut name = HashMap::new();
        for (argument_constraint,register) in in_.each_member() {
            let type_constraint =
                TypeConstraint::from_argumentconstraint(&argument_constraint,|s| {
                    let next_val = self.next;
                    let val = *name.entry(s.to_string()).or_insert(next_val);
                    if val == next_val { self.next += 1; }
                    val
                });
            out.push((type_constraint,register.clone()));
        }
        out
    }

    pub fn add(&mut self, sig: &InstructionConstraint) -> anyhow::Result<()> {
        for (constraint,register) in self.extract(sig) {
            let is_ref = match constraint {
                TypeConstraint::Reference(_) => true,
                TypeConstraint::NonReference(_) => false
            };
            if is_ref {
                self.reg_isref.insert(register.clone());
            }
            let next_val = self.next;
            let reg_id = *self.regmap.entry(register.clone()).or_insert(next_val);
            if reg_id == next_val { self.next += 1; }
            self.store.add(&Key::External(reg_id),constraint.get_expressionconstraint())?;
        }
        Ok(())
    }

    pub fn get(&self, reg: &Register) -> ExpressionType {
        if let Some(reg_id) = self.regmap.get(reg) {
            if let Some(out) = self.store.get(&Key::External(*reg_id)) {
                return out;
            }
        }
        ExpressionType::Base(BaseType::Invalid)
    }

    pub fn all_external(&self) -> Vec<(Register,ExpressionType)> {
        let revmap : HashMap<usize,Register> = 
            HashMap::from_iter(self.regmap.iter().map(|(k,v)| (v.clone(),k.clone())));
        self.store.get_all().filter_map(|(key,typ)| {
            if let Key::External(id) = key {
                if let Some(reg) = revmap.get(id) {
                    return Some((reg.clone(),typ));
                }
            }
            None
        }).collect()
    }
}
