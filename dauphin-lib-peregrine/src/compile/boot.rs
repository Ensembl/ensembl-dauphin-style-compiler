use crate::simple_command;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use dauphin_compile::command::{ 
    Command, CommandSchema, CommandType, CommandTrigger, Instruction, InstructionType
};
use dauphin_interp::command::{ Identifier };
use dauphin_interp::runtime::{ Register };
use dauphin_interp::types::{ to_xstructure, XStructure, map_xstructure };
use dauphin_interp::util::DauphinError;
use serde_cbor::Value as CborValue;

simple_command!(AddStickAuthorityCommand,AddStickAuthorityCommandType,"peregrine","add_stick_authority",1,(0));
simple_command!(GetStickIdCommand,GetStickIdCommandType,"peregrine","get_stick_id",1,(0));

                            /* 0: name  1: size  2: topo  3:tags/D 4: tags/A0 5: tags/B0 */ 
pub struct GetStickDataCommand(Register,Register,Register,Register,Register,Register,
                    /* stick:  6:name/D 7:name/A 8:name/B 9:prio/D 10:prio/A 11:prio/B */
                               Register,Register,Register,Register,Register,Register,
                            /* 12: channel 13: id */
                               Register,Register);

impl Command for GetStickDataCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![
            self.0.serialize(),self.1.serialize(),self.2.serialize(),self.3.serialize(),
            self.4.serialize(),self.5.serialize(),self.6.serialize(),self.7.serialize(),
            self.8.serialize(),self.9.serialize(),self.10.serialize(),self.11.serialize(),
            self.12.serialize(),self.13.serialize()
        ]))
    }
}

pub struct GetStickDataCommandType();

impl CommandType for GetStickDataCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 14,
            trigger: CommandTrigger::Command(Identifier::new("peregrine","get_stick_data"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,sig,_) = &it.itype {
            let mut xs_kv_allot = HashMap::new();
            xs_kv_allot.insert("name".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![6,7,8])))));
            xs_kv_allot.insert("priority".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![9,10,11])))));
            let mut xs_kv : HashMap<String,Rc<XStructure<Vec<usize>>>> = HashMap::new();
            xs_kv.insert("name".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![0])))));
            xs_kv.insert("size".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![1])))));
            xs_kv.insert("topology".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![2])))));
            xs_kv.insert("tags".to_string(),Rc::new(XStructure::Vector(Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![3,4,5])))))));
            xs_kv.insert("allotments".to_string(),Rc::new(XStructure::Vector(Rc::new(XStructure::Struct(Identifier::new("peregrine","allot"),xs_kv_allot)))));
            let xs = XStructure::Struct(Identifier::new("peregrine","stick"),xs_kv);
            let mut pos = [0,0,0,0,0,0,0,0,0,0,0,0,0,0];
            map_xstructure(&mut pos,&to_xstructure(&sig[0])?,&xs)?;
            pos[12] = sig[1].iter().next().unwrap().1.data_pos();
            pos[13] = sig[2].iter().next().unwrap().1.data_pos();
            let regs : Vec<_> = pos.iter().map(|x| it.regs[*x]).collect();
            Ok(Box::new(GetStickDataCommand(regs[0],regs[1],regs[2],regs[3],regs[4],regs[5],regs[6],regs[7],
                                            regs[8],regs[9],regs[10],regs[11],regs[12],regs[13])))
        } else {
            Err(DauphinError::internal(file!(),line!()))
        }
    }
}

/*                         0: name  1: size  2: topo  3:tags/D 4: tags/A0 5: tags/B0  */
pub struct AddStickCommand(Register,Register,Register,Register,Register,Register,
                /* stick:  6:name/D 7:name/A 8:name/B 9:prio/D 10:prio/A 11:prio/B */
                           Register,Register,Register,Register,Register,Register);


impl Command for AddStickCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![
            self.0.serialize(),self.1.serialize(),self.2.serialize(),self.3.serialize(),
            self.4.serialize(),self.5.serialize(),self.6.serialize(),self.7.serialize(),
            self.8.serialize(),self.9.serialize(),self.10.serialize(),self.11.serialize()
        ]))
    }
}

pub struct AddStickCommandType();

impl CommandType for AddStickCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 12,
            trigger: CommandTrigger::Command(Identifier::new("peregrine","add_stick"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,sig,_) = &it.itype {
            let mut xs_kv_allot = HashMap::new();
            xs_kv_allot.insert("name".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![6,7,8])))));
            xs_kv_allot.insert("priority".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![9,10,11])))));
            let mut xs_kv : HashMap<String,Rc<XStructure<Vec<usize>>>> = HashMap::new();
            xs_kv.insert("name".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![0])))));
            xs_kv.insert("size".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![1])))));
            xs_kv.insert("topology".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![2])))));
            xs_kv.insert("tags".to_string(),Rc::new(XStructure::Vector(Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![3,4,5])))))));
            xs_kv.insert("allotments".to_string(),Rc::new(XStructure::Vector(Rc::new(XStructure::Struct(Identifier::new("peregrine","allot"),xs_kv_allot)))));
            let xs = XStructure::Struct(Identifier::new("peregrine","stick"),xs_kv);
            let mut pos = [0,0,0,0,0,0,0,0,0,0,0,0];
            map_xstructure(&mut pos,&to_xstructure(&sig[0])?,&xs)?;
            let regs : Vec<_> = pos.iter().map(|x| it.regs[*x]).collect();
            Ok(Box::new(AddStickCommand(regs[0],regs[1],regs[2],regs[3],regs[4],regs[5],
                                        regs[6],regs[7],regs[8],regs[9],regs[10],regs[11])))
        } else {
            Err(DauphinError::internal(file!(),line!()))
        }
    }
}
