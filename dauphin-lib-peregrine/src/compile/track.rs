use dauphin_compile::command::{ 
    Command, CommandSchema, CommandType, CommandTrigger, Instruction,
};
use dauphin_interp::command::{ Identifier };
use dauphin_interp::runtime::{ Register };
use serde_cbor::Value as CborValue;
use crate::simple_command;

simple_command!(AppendGroupCommand,AppendGroupCommandType,"peregrine","append_group",3,(0,1,2));
simple_command!(AppendDepthCommand,AppendDepthCommandType,"peregrine","append_depth",3,(0,1,2));
