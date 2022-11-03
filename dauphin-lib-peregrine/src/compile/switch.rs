use dauphin_compile::command::{ 
    Command, CommandSchema, CommandType, CommandTrigger, Instruction,
};
use dauphin_interp::command::{ Identifier };
use dauphin_interp::runtime::{ Register };
use serde_cbor::Value as CborValue;
use crate::simple_command;

simple_command!(ListSwitchCommand,ListSwitchCommandType,"peregrine","list_switch",4,(0,1,2,3));

simple_command!(SwitchStringCommand,SwitchStringCommandType,"peregrine","switch_string",3,(0,1,2));
simple_command!(SwitchNumberCommand,SwitchNumberCommandType,"peregrine","switch_number",3,(0,1,2));
simple_command!(SwitchBooleanCommand,SwitchBooleanCommandType,"peregrine","switch_boolean",3,(0,1,2));
simple_command!(SwitchNullCommand,SwitchNullCommandType,"peregrine","switch_null",3,(0,1,2));

simple_command!(SettingStringCommand,SettingStringCommandType,"peregrine","setting_string",3,(0,1,2));
simple_command!(SettingNumberCommand,SettingNumberCommandType,"peregrine","setting_number",3,(0,1,2));
simple_command!(SettingBooleanCommand,SettingBooleanCommandType,"peregrine","setting_boolean",3,(0,1,2));
simple_command!(SettingNullCommand,SettingNullCommandType,"peregrine","setting_null",3,(0,1,2));
