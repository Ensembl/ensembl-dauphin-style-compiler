use dauphin_compile::command::{ 
    Command, CommandSchema, CommandType, CommandTrigger, Instruction,
};
use dauphin_interp::command::{ Identifier };
use dauphin_interp::runtime::{ Register };
use serde_cbor::Value as CborValue;
use crate::simple_command;

simple_command!(GetSwitchCommand,GetSwitchCommandType,"peregrine","get_switch",4,(0,1,2,3));