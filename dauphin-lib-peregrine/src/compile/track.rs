use dauphin_compile::command::{ 
    Command, CommandSchema, CommandType, CommandTrigger, Instruction,
};
use dauphin_interp::command::{ Identifier };
use dauphin_interp::runtime::{ Register };
use serde_cbor::Value as CborValue;
use crate::simple_command;

simple_command!(NewLaneCommand,NewLaneCommandType,"peregrine","track_new",6,(0,1,2,3,4,5));
simple_command!(AddTagCommand,AddTagCommandType,"peregrine","track_add_tag",2,(0,1));
simple_command!(AddTrackCommand,AddTrackCommandType,"peregrine","track_add_trigger",4,(0,1,2,3));
simple_command!(AddSwitchCommand,AddSwitchCommandType,"peregrine","track_add_switch",4,(0,1,2,3));
simple_command!(AddAllotmentCommand,AddAllotmentCommandType,"peregrine","track_add_allotment",3,(0,1,2));
simple_command!(DataSourceCommand,DataSourceCommandType,"peregrine","track_apply",1,(0));
