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

simple_command!(RequestCommand,RequestCommandType,"peregrine","make_request",6,(0,1,2,3,4,5));
simple_command!(RequestScopeCommand,RequestScopeCommandType,"peregrine","request_scope",4,(0,1,2,3));
simple_command!(GetDataCommand,GetDataCommandType,"peregrine","get_data",2,(0,1));
simple_command!(OnlyWarmCommand,OnlyWarmCommandType,"peregrine","only_warm",1,(0));
/*                         0: stick 1: index 2: scale */
pub struct GetLaneCommand(Register,Register,Register);

impl Command for GetLaneCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![self.0.serialize(),self.1.serialize(),self.2.serialize()]))
    }
}

pub struct GetLaneCommandType();

impl CommandType for GetLaneCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 3,
            trigger: CommandTrigger::Command(Identifier::new("peregrine","get_region"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,sig,_) = &it.itype {
            let mut xs_kv : HashMap<String,Rc<XStructure<Vec<usize>>>> = HashMap::new();
            xs_kv.insert("stick".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![0])))));
            xs_kv.insert("index".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![1])))));
            xs_kv.insert("scale".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![2])))));
            let xs = XStructure::Struct(Identifier::new("peregrine","region"),xs_kv);
            let mut pos = [0,0,0];
            map_xstructure(&mut pos,&to_xstructure(&sig[0])?,&xs)?;
            let regs : Vec<_> = pos.iter().map(|x| it.regs[*x]).collect();
            Ok(Box::new(GetLaneCommand(regs[0],regs[1],regs[2])))
        } else {
            Err(DauphinError::internal(file!(),line!()))
        }
    }
}

/* out: [0: stick 1: index 2: scale] in: [3: stick 4: start 5: end] */
pub struct MakeRegionCommand(Register,Register,Register,Register,Register,Register);

impl Command for MakeRegionCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![
            self.0.serialize(),self.1.serialize(),self.2.serialize(),
            self.3.serialize(),self.4.serialize(),self.5.serialize(),
        ]))
    }
}

pub struct MakeRegionCommandType();

impl CommandType for MakeRegionCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 6,
            trigger: CommandTrigger::Command(Identifier::new("peregrine","make_region"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,sig,_) = &it.itype {
            let mut xs_kv : HashMap<String,Rc<XStructure<Vec<usize>>>> = HashMap::new();
            xs_kv.insert("stick".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![0])))));
            xs_kv.insert("index".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![1])))));
            xs_kv.insert("scale".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![2])))));
            let xs = XStructure::Struct(Identifier::new("peregrine","region"),xs_kv);
            let mut pos = [0,0,0,0,0,0];
            map_xstructure(&mut pos,&to_xstructure(&sig[0])?,&xs)?;
            pos[3] = sig[1].iter().next().unwrap().1.data_pos();
            pos[4] = sig[2].iter().next().unwrap().1.data_pos();
            pos[5] = sig[3].iter().next().unwrap().1.data_pos();
            let regs : Vec<_> = pos.iter().map(|x| it.regs[*x]).collect();
            Ok(Box::new(MakeRegionCommand(regs[0],regs[1],regs[2],regs[3],regs[4],regs[5])))
        } else {
            Err(DauphinError::internal(file!(),line!()))
        }
    }
}

simple_command!(DataStreamCommand,DataStreamCommandType,"peregrine","data_stream",3,(0,1,2));
