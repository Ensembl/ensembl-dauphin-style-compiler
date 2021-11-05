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

simple_command!(AddAuthorityCommand,AddAuthorityCommandType,"peregrine","add_stick_authority",1,(0));
simple_command!(GetStickIdCommand,GetStickIdCommandType,"peregrine","get_stick_id",1,(0));
simple_command!(GetJumpLocationCommand,GetJumpLocationCommandType,"peregrine","get_jump_location",1,(0));

/*                             0: name  1: size  2: topo  3:tags/D 4: tags/A0 5: tags/B0 6: channel 7: id */
pub struct GetStickDataCommand(Register,Register,Register,Register,Register,Register,Register,Register);

impl Command for GetStickDataCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![
            self.0.serialize(),self.1.serialize(),self.2.serialize(),self.3.serialize(),
            self.4.serialize(),self.5.serialize(),self.6.serialize(),self.7.serialize()
        ]))
    }
}

pub struct GetStickDataCommandType();

impl CommandType for GetStickDataCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 8,
            trigger: CommandTrigger::Command(Identifier::new("peregrine","get_stick_data"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,sig,_) = &it.itype {
            let mut xs_kv : HashMap<String,Rc<XStructure<Vec<usize>>>> = HashMap::new();
            xs_kv.insert("name".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![0])))));
            xs_kv.insert("size".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![1])))));
            xs_kv.insert("topology".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![2])))));
            xs_kv.insert("tags".to_string(),Rc::new(XStructure::Vector(Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![3,4,5])))))));
            let xs = XStructure::Struct(Identifier::new("peregrine","stick"),xs_kv);
            let mut pos = [0,0,0,0,0,0,0,0];
            map_xstructure(&mut pos,&to_xstructure(&sig[0])?,&xs)?;
            pos[6] = sig[1].iter().next().unwrap().1.data_pos();
            pos[7] = sig[2].iter().next().unwrap().1.data_pos();
            let regs : Vec<_> = pos.iter().map(|x| it.regs[*x]).collect();
            Ok(Box::new(GetStickDataCommand(regs[0],regs[1],regs[2],regs[3],regs[4],regs[5],regs[6],regs[7])))
        } else {
            Err(DauphinError::internal(file!(),line!()))
        }
    }
}

/*                            0: stick 1: start 2: end  3: channel, 4: location */
pub struct GetJumpDataCommand(Register,Register,Register,Register,Register);

impl Command for GetJumpDataCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![
            self.0.serialize(),self.1.serialize(),self.2.serialize(),self.3.serialize(),
            self.4.serialize()
        ]))
    }
}

pub struct GetJumpDataCommandType();

impl CommandType for GetJumpDataCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 5,
            trigger: CommandTrigger::Command(Identifier::new("peregrine","get_jump_data"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,sig,_) = &it.itype {
            let mut xs_kv : HashMap<String,Rc<XStructure<Vec<usize>>>> = HashMap::new();
            xs_kv.insert("stick".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![0])))));
            xs_kv.insert("start".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![1])))));
            xs_kv.insert("end".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![2])))));
            let xs = XStructure::Struct(Identifier::new("peregrine","jump"),xs_kv);
            let mut pos = [0,0,0,0,0];
            map_xstructure(&mut pos,&to_xstructure(&sig[0])?,&xs)?;
            pos[3] = sig[1].iter().next().unwrap().1.data_pos();
            pos[4] = sig[2].iter().next().unwrap().1.data_pos();
            let regs : Vec<_> = pos.iter().map(|x| it.regs[*x]).collect();
            Ok(Box::new(GetJumpDataCommand(regs[0],regs[1],regs[2],regs[3],regs[4])))
        } else {
            Err(DauphinError::internal(file!(),line!()))
        }
    }
}

/*                         0: name  1: size  2: topo  3:tags/D 4: tags/A0 5: tags/B0  */
pub struct AddStickCommand(Register,Register,Register,Register,Register,Register);

impl Command for AddStickCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![
            self.0.serialize(),self.1.serialize(),self.2.serialize(),self.3.serialize(),
            self.4.serialize(),self.5.serialize()
        ]))
    }
}

pub struct AddStickCommandType();

impl CommandType for AddStickCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 6,
            trigger: CommandTrigger::Command(Identifier::new("peregrine","add_stick"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,sig,_) = &it.itype {
            let mut xs_kv : HashMap<String,Rc<XStructure<Vec<usize>>>> = HashMap::new();
            xs_kv.insert("name".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![0])))));
            xs_kv.insert("size".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![1])))));
            xs_kv.insert("topology".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![2])))));
            xs_kv.insert("tags".to_string(),Rc::new(XStructure::Vector(Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![3,4,5])))))));
            let xs = XStructure::Struct(Identifier::new("peregrine","stick"),xs_kv);
            let mut pos = [0,0,0,0,0,0];
            map_xstructure(&mut pos,&to_xstructure(&sig[0])?,&xs)?;
            let regs : Vec<_> = pos.iter().map(|x| it.regs[*x]).collect();
            Ok(Box::new(AddStickCommand(regs[0],regs[1],regs[2],regs[3],regs[4],regs[5])))
        } else {
            Err(DauphinError::internal(file!(),line!()))
        }
    }
}

pub struct AddJumpCommand(Register,Register,Register,Register);

impl Command for AddJumpCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![
            self.0.serialize(),self.1.serialize(),self.2.serialize(),self.3.serialize(),
        ]))
    }
}

pub struct AddJumpCommandType();

impl CommandType for AddJumpCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 4,
            trigger: CommandTrigger::Command(Identifier::new("peregrine","add_jump"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,sig,_) = &it.itype {
            let mut xs_kv : HashMap<String,Rc<XStructure<Vec<usize>>>> = HashMap::new();
            xs_kv.insert("stick".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![0])))));
            xs_kv.insert("start".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![1])))));
            xs_kv.insert("end".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![2])))));
            let xs = XStructure::Struct(Identifier::new("peregrine","jump"),xs_kv);
            let mut pos = [0,0,0];
            map_xstructure(&mut pos,&to_xstructure(&sig[1])?,&xs)?;
            let regs : Vec<_> = pos.iter().map(|x| it.regs[*x]).collect();
            Ok(Box::new(AddJumpCommand(it.regs[0],regs[0],regs[1],regs[2])))
        } else {
            Err(DauphinError::internal(file!(),line!()))
        }
    }
}
