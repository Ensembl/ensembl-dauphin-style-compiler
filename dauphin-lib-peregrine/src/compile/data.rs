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
            trigger: CommandTrigger::Command(Identifier::new("peregrine","get_lane"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,sig,_) = &it.itype {
            let mut xs_kv : HashMap<String,Rc<XStructure<Vec<usize>>>> = HashMap::new();
            xs_kv.insert("stick".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![0])))));
            xs_kv.insert("index".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![1])))));
            xs_kv.insert("scale".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![2])))));
            let xs = XStructure::Struct(Identifier::new("peregrine","lane"),xs_kv);
            let mut pos = [0,0,0];
            map_xstructure(&mut pos,&to_xstructure(&sig[0])?,&xs)?;
            let regs : Vec<_> = pos.iter().map(|x| it.regs[*x]).collect();
            Ok(Box::new(GetLaneCommand(regs[0],regs[1],regs[2])))
        } else {
            Err(DauphinError::internal(file!(),line!()))
        }
    }
}

// func get_data(string,string,lane) becomes datasource;

                     /* 0:out     1:channel 2:name  3:p/stick 4:p/index 5:p/scale */
pub struct GetDataCommand(Register,Register,Register,Register,Register,Register);

impl Command for GetDataCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![
            self.0.serialize(),self.1.serialize(),self.2.serialize(),self.3.serialize(),
            self.4.serialize(),self.5.serialize()
        ]))
    }
}

pub struct GetDataCommandType();

impl CommandType for GetDataCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 6,
            trigger: CommandTrigger::Command(Identifier::new("peregrine","get_data"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,sig,_) = &it.itype {
            let mut xs_kv : HashMap<String,Rc<XStructure<Vec<usize>>>> = HashMap::new();
            xs_kv.insert("stick".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![3])))));
            xs_kv.insert("index".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![4])))));
            xs_kv.insert("scale".to_string(),Rc::new(XStructure::Simple(Rc::new(RefCell::new(vec![5])))));
            let xs = XStructure::Struct(Identifier::new("peregrine","lane"),xs_kv);
            let mut pos = [0,0,0,0,0,0];
            map_xstructure(&mut pos,&to_xstructure(&sig[3])?,&xs)?;
            for i in 0..3 {
                pos[i] = sig[i].iter().next().unwrap().1.data_pos();
            }
            let regs : Vec<_> = pos.iter().map(|x| it.regs[*x]).collect();
            Ok(Box::new(GetDataCommand(regs[0],regs[1],regs[2],regs[3],regs[4],regs[5])))
        } else {
            Err(DauphinError::internal(file!(),line!()))
        }
    }
}

simple_command!(DataStreamCommand,DataStreamCommandType,"peregrine","data_stream",3,(0,1,2));
