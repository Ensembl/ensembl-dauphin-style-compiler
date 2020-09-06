use crate::simple_command;
use dauphin_compile::command::{ 
    Command, CommandSchema, CommandType, CommandTrigger, Instruction, InstructionType
};
use dauphin_interp::command::{ Identifier };
use dauphin_interp::runtime::{ Register };
use dauphin_interp::util::DauphinError;
use serde_cbor::Value as CborValue;

simple_command!(InflateBytesCommand,InflateBytesCommandType,"peregrine","inflate_bytes",2,(0,1));
simple_command!(InflateStringCommand,InflateStringCommandType,"peregrine","inflate_string",2,(0,1));
simple_command!(Lesqlite2Command,Lesqlite2CommandType,"peregrine","lesqlite2",2,(0,1));
simple_command!(ZigzagCommand,ZigzagCommandType,"peregrine","zigzag",2,(0,1));
simple_command!(DeltaCommand,DeltaCommandType,"peregrine","delta",2,(0,1));
simple_command!(ClassifyCommand,ClassifyCommandType,"peregrine","classify",3,(0,1,2));

/*                             0: out/D  1: out/A  2: out/B  3:in  */
pub struct SplitStringCommand(Register,Register,Register,Register);

impl Command for SplitStringCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![
            self.0.serialize(),self.1.serialize(),self.2.serialize(),self.3.serialize()
        ]))
    }
}

pub struct SplitStringCommandType();

impl CommandType for SplitStringCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 4,
            trigger: CommandTrigger::Command(Identifier::new("peregrine","split_string"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,sig,_) = &it.itype {
            let out_vec = sig[0].iter().next().unwrap().1;
            let pos = vec![
                out_vec.data_pos(),
                out_vec.offset_pos(0)?,
                out_vec.length_pos(0)?,
                sig[1].iter().next().unwrap().1.data_pos()
            ];
            let regs : Vec<_> = pos.iter().map(|x| it.regs[*x]).collect();
            Ok(Box::new(SplitStringCommand(regs[0],regs[1],regs[2],regs[3])))
        } else {
            Err(DauphinError::internal(file!(),line!()))
        }
    }
}
