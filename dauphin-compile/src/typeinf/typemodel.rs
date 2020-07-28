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

use std::collections::{ BTreeMap, HashMap };
use std::fmt;
use std::iter::FromIterator;

use super::types::MemberType;
use super::typing::Typing;
use dauphin_interp::runtime::Register;
use dauphin_interp::types::{ BaseType };

pub struct TypeModel {
    values: BTreeMap<Register,MemberType>
}

impl fmt::Debug for TypeModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut keys : Vec<Register> = self.values.keys().cloned().collect();
        keys.sort();
        for reg in &keys {
            write!(f,"{:?} : {:?}\n",reg,self.values[reg])?;
        }
        Ok(())
    }
}

impl TypeModel {
    pub fn new() -> TypeModel {
        TypeModel {
            values: BTreeMap::new()
        }
    }

    pub fn set(&mut self, reg: &Register, type_: &MemberType) {
        self.values.insert(reg.clone(),type_.clone());
    }

    pub fn get(&self, reg: &Register) -> Option<&MemberType> {
        self.values.get(reg)
    }

    pub fn each_register(&self) -> impl Iterator<Item=(&Register,&MemberType)> {
        self.values.iter()
    }

    pub fn from_constraints(&mut self, types: &Typing) {
        for (reg,expression_type) in types.all_external() {
            self.set(&reg,&expression_type.to_membertype(&BaseType::BooleanType));
        }
    }
}