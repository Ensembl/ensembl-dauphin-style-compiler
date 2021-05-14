use dauphin_compile::command::{ 
    Command, CommandSchema, CommandType, CommandTrigger, Instruction,
};
use dauphin_interp::command::{ Identifier };
use dauphin_interp::runtime::{ Register };
use serde_cbor::Value as CborValue;
use crate::simple_command;

simple_command!(Rectangle2Command,Rectangle2CommandType,"peregrine","rectangle2",8,(0,1,2,3,4,5,6,7));
simple_command!(Rectangle1Command,Rectangle1CommandType,"peregrine","rectangle1",8,(0,1,2,3,4,5,6,7));
simple_command!(TextCommand,TextCommandType,"peregrine","text",7,(0,1,2,3,4,5,6));
simple_command!(WiggleCommand,WiggleCommandType,"peregrine","wiggle",6,(0,1,2,3,4,5));
simple_command!(RectangleCommand,RectangleCommandType,"peregrine","rectangle",4,(0,1,2,3));
simple_command!(Text2Command,Text2CommandType,"peregrine","text2",4,(0,1,2,3));
