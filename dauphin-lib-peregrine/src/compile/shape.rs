use dauphin_compile::command::{ 
    Command, CommandSchema, CommandType, CommandTrigger, Instruction,
};
use dauphin_interp::command::{ Identifier };
use dauphin_interp::runtime::{ Register };
use serde_cbor::Value as CborValue;
use crate::simple_command;

simple_command!(WiggleCommand,WiggleCommandType,"peregrine","wiggle",6,(0,1,2,3,4,5));
simple_command!(RectangleCommand,RectangleCommandType,"peregrine","rectangle",4,(0,1,2,3));
simple_command!(Text2Command,Text2CommandType,"peregrine","text2",4,(0,1,2,3));
