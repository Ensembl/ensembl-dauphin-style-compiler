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

use anyhow::{ self };
use std::collections::{ HashMap };
use dauphin_interp::command::{ CommandInterpretSuite, CommandSetId, OpcodeMapping, CommandDeserializer };
use crate::command::{ CommandCompileSuite, CompilerLink };
use dauphin_interp::util::DauphinError;
use dauphin_interp::util::cbor::{ cbor_array, cbor_int };
use serde_cbor::Value as CborValue;

pub struct OpcodeRemapper {
    suites: HashMap<(String,u32),CommandSetId>,
    mapping: OpcodeMapping
}

impl OpcodeRemapper {
    pub fn new(mapping: &OpcodeMapping) -> OpcodeRemapper {
        let mapping = mapping.clone();
        let suites = mapping.iter().map(|x| ((x.0.name().to_string(),x.0.version().0),x.0.clone())).collect::<HashMap<_,_>>();
        OpcodeRemapper { suites, mapping }
    }

    fn load(&mut self, mapping: &OpcodeMapping) -> anyhow::Result<()> {
        for (incoming_sid,_) in mapping.iter() {
            if let Some(stored_sid) = self.suites.get(&(incoming_sid.name().to_string(),incoming_sid.version().0)) {
                if stored_sid.version().1 < incoming_sid.version().1 {
                    return Err(DauphinError::integration(&format!("Need {}, got {}\n",incoming_sid,stored_sid)));
                }
            } else {
                return Err(DauphinError::integration(&format!("Missing {}\n",incoming_sid)));
            }
        }
        Ok(())
    }

    fn sid_to_offset(&self, sid: &CommandSetId) -> anyhow::Result<u32> {
        self.mapping.sid_to_offset(sid)
    }

    fn to_canonical(&self, sid: &CommandSetId) -> anyhow::Result<&CommandSetId> {
        Ok(self.suites.get(&(sid.name().to_string(),sid.version().0)).ok_or_else(|| DauphinError::malformed("no such sid"))?)
    }

    pub fn serialize(&self) -> CborValue {
        self.mapping.serialize()
    }
}

pub struct RemapperEndpoint {
    mapping: OpcodeMapping
}

impl RemapperEndpoint {
    pub fn new(remapper: &mut OpcodeRemapper, suite: &CborValue) -> anyhow::Result<RemapperEndpoint> {
        let out = RemapperEndpoint {
            mapping: OpcodeMapping::deserialize(suite)?
        };
        remapper.load(&out.mapping)?;
        Ok(out)
    }

    fn remap_opcode(&self, remapper: &OpcodeRemapper, opcode: u32) -> anyhow::Result<u32> {
        let (sid,offset) = self.mapping.decode_opcode(opcode)?;
        let sid = remapper.to_canonical(&sid)?;
        Ok(remapper.sid_to_offset(&sid)?+offset)
    }

    fn opcode_len(&self, clink: &CompilerLink, opcode: u32) -> anyhow::Result<usize> {
        let ds = clink.get_suite().get_deserializer_by_opcode(opcode)?;
        Ok(ds.get_opcode_len()?.ok_or_else(|| DauphinError::malformed("attempt to deserialize an unserializable"))?.1)
    }

    pub fn remap_program(&self, clink: &CompilerLink, remapper: &OpcodeRemapper, program: &CborValue) -> anyhow::Result<CborValue> {
        let mut out = vec![];
        let mut values = cbor_array(program,0,true)?.iter();
        while let Some(opcode) = values.next() {
            let opcode = cbor_int(opcode,None)? as u32;
            let opcode = self.remap_opcode(remapper,opcode)?;
            out.push(CborValue::Integer(opcode as i128));
            let num_args = self.opcode_len(clink,opcode)?;
            let mut values = (0..num_args).map(|_| values.next().cloned())
                                .collect::<Option<Vec<_>>>().ok_or_else(|| DauphinError::malformed("attempt to deserialize an unserializable"))?;
            out.append(&mut values);
        }
        Ok(CborValue::Array(out))
    }
}

#[cfg(test)]
mod test{
    use std::cell::RefCell;
    use std::rc::Rc;
    use crate::command::{ CommandCompileSuite, CommandSchema, CommandTrigger, CommandType, Instruction, Command, CompLibRegister };
    use crate::test::{ FakeDeserializer, hexdump };
    use dauphin_interp::command::{ Identifier, InterpLibRegister, InterpreterLink };
    use dauphin_interp::util::DauphinError;
    use dauphin_interp::util::cbor::{ cbor_serialize };
    use super::*;

    // XXX dedup fake*
    fn fake_trigger(name: &str) -> CommandTrigger {
        CommandTrigger::Command(Identifier::new("fake",name))
    }

    struct FakeCommandType(String);

    impl CommandType for FakeCommandType {
        fn get_schema(&self) -> CommandSchema {
            CommandSchema {
                trigger: fake_trigger(&self.0),
                values: 0
            }
        }

        fn from_instruction(&self, _it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
            Err(DauphinError::internal(file!(),line!()))
        }
    }

    fn fake_command(name: &str) -> impl CommandType {
        FakeCommandType(name.to_string())
    }

    fn command(v: &Rc<RefCell<u32>>, ccs: &mut CommandCompileSuite, name: &str, opcode: u32, version: &(u32,u32), trace: u64) {
        let csi1 = CommandSetId::new(name,*version,trace);
        let mut is1 = InterpLibRegister::new(&csi1);
        is1.push(FakeDeserializer(v.clone(),opcode));
        let mut cs1 = CompLibRegister::new(&csi1,Some(is1));
        cs1.push(name,Some(opcode),fake_command("c"));
        ccs.register(cs1).expect("a");
    }

    fn interpret(v: &Rc<RefCell<u32>>, cis: &mut CommandInterpretSuite, name: &str, opcode: u32, version: &(u32,u32), trace: u64) {
        let csi1 = CommandSetId::new(name,*version,trace);
        let mut is1 = InterpLibRegister::new(&csi1);
        is1.push(FakeDeserializer(v.clone(),opcode));
        cis.register(is1);        
    }

    #[test]
    fn remapper_smoke() {
        let v : Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        /* first program! */
        let mut ccs = CommandCompileSuite::new();
        command(&v,&mut ccs,"test", 5,&(1,2),0xF32E7C72C8633855);
        command(&v,&mut ccs,"test2",6,&(1,2),0xB03D4E7C72C8628A);
        /* second program! (other way round) */
        let mut ccs2 = CommandCompileSuite::new();
        command(&v,&mut ccs2,"test2",6,&(1,2),0xB03D4E7C72C8628A);
        command(&v,&mut ccs2,"test", 5,&(1,2),0xF32E7C72C8633855);
        /* serialize them */
        let prog1 = ccs.serialize();
        let prog2 = ccs2.serialize();
        print!("PROG1\n{:?}\nPROG2\n{:?}\n",hexdump(&cbor_serialize(&prog1).expect("n")),hexdump(&cbor_serialize(&prog2).expect("n")));
        /* interpreter */
        let mut cis = CommandInterpretSuite::new();
        interpret(&v,&mut cis,"test",5,&(1,2),0xF32E7C72C8633855);
        interpret(&v,&mut cis,"test2",6,&(1,2),0xB03D4E7C72C8628A);
        /* remapper! (at last) */
        let mut remapper = OpcodeRemapper::new(&cis.default_opcode_mapper());
        let rm1 = RemapperEndpoint::new(&mut remapper, &prog1).expect("b");
        let rm2 = RemapperEndpoint::new(&mut remapper, &prog2).expect("b");
        assert_eq!(5,rm1.remap_opcode(&remapper,5).expect("c"));
        assert_eq!(5,rm2.remap_opcode(&remapper,12).expect("d"));
        assert_eq!(12,rm1.remap_opcode(&remapper,12).expect("e"));
        assert_eq!(12,rm2.remap_opcode(&remapper,6).expect("f"));
    }

    #[test]
    fn remapper_version_bump() {
        let v : Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        /* first program! */
        let mut ccs = CommandCompileSuite::new();
        command(&v,&mut ccs,"test", 4,&(1,1),0xF32E7C72C86339E5);
        command(&v,&mut ccs,"test2",6,&(1,2),0xB03D4E7C72C8628A);
        /* second program! (other way round) */
        let mut ccs2 = CommandCompileSuite::new();
        command(&v,&mut ccs2,"test2",6,&(1,2),0xB03D4E7C72C8628A);
        command(&v,&mut ccs2,"test", 5,&(1,2),0xF32E7C72C8633855);
        /* serialize them */
        let prog1 = ccs.serialize();
        let prog2 = ccs2.serialize();
        print!("PROG1\n{:?}\nPROG2\n{:?}\n",hexdump(&cbor_serialize(&prog1).expect("n")),hexdump(&cbor_serialize(&prog2).expect("n")));
        /* interpreter */
        let mut cis = CommandInterpretSuite::new();
        interpret(&v,&mut cis,"test",5,&(1,2),0xF32E7C72C8633855);
        interpret(&v,&mut cis,"test2",6,&(1,2),0xB03D4E7C72C8628A);
        /* remapper! */
        let mut remapper = OpcodeRemapper::new(&cis.default_opcode_mapper());
        let rm1 = RemapperEndpoint::new(&mut remapper, &prog1).expect("b");
        let rm2 = RemapperEndpoint::new(&mut remapper, &prog2).expect("b");
        assert_eq!(6,rm1.remap_opcode(&remapper,5).expect("c"));
        assert_eq!(5,rm2.remap_opcode(&remapper,12).expect("d"));
        assert_eq!(13,rm1.remap_opcode(&remapper,12).expect("e"));
        assert_eq!(12,rm2.remap_opcode(&remapper,6).expect("f"));
    }
}