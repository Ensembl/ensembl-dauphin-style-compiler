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

pub struct AddStickAuthorityCommand(Register);

impl Command for AddStickAuthorityCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![self.0.serialize()]))
    }
}

pub struct AddStickAuthorityCommandType();

impl CommandType for AddStickAuthorityCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 1,
            trigger: CommandTrigger::Command(Identifier::new("peregrine","add_stick_authority"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        Ok(Box::new(AddStickAuthorityCommand(it.regs[0])))
    }
}

pub struct GetStickIdCommand(Register);

impl Command for GetStickIdCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![self.0.serialize()]))
    }
}

pub struct GetStickIdCommandType();

impl CommandType for GetStickIdCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 1,
            trigger: CommandTrigger::Command(Identifier::new("peregrine","get_stick_id"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        Ok(Box::new(GetStickIdCommand(it.regs[0])))
    }
}

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


