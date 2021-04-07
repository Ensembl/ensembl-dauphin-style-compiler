use dauphin_compile::command::{ 
    Command, CommandSchema, CommandType, CommandTrigger, Instruction,
};
use dauphin_interp::command::{ Identifier };
use dauphin_interp::runtime::{ Register };
use serde_cbor::Value as CborValue;
use crate::simple_command;

simple_command!(NewLaneCommand,NewLaneCommandType,"peregrine","lane_new",1,(0));
simple_command!(AddTagCommand,AddTagCommandType,"peregrine","lane_add_tag",2,(0,1));
simple_command!(AddTrackCommand,AddTrackCommandType,"peregrine","lane_add_track",2,(0,1));
simple_command!(SetScaleCommand,SetScaleCommandType,"peregrine","lane_set_scale",3,(0,1,2));
simple_command!(DataSourceCommand,DataSourceCommandType,"peregrine","lane_apply",3,(0,1,2));
simple_command!(SetMaxScaleJumpCommand,SetMaxScaleJumpCommandType,"peregrine","lane_set_max_scale_jump",2,(0,1));
