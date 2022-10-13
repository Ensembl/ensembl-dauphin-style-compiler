use crate::simple_command;
use dauphin_compile::command::{ 
    Command, CommandSchema, CommandType, CommandTrigger, Instruction
};
use dauphin_interp::command::{ Identifier };
use dauphin_interp::runtime::{ Register };
use serde_cbor::Value as CborValue;

simple_command!(AddAuthorityCommand,AddAuthorityCommandType,"peregrine","add_stick_authority",1,(0));
