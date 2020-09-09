use dauphin_compile::command::{ 
    Command, CommandSchema, CommandType, CommandTrigger, Instruction,
};
use dauphin_interp::command::{ Identifier };
use dauphin_interp::runtime::{ Register };
use serde_cbor::Value as CborValue;
use crate::simple_command;

simple_command!(IntervalCommand,IntervalCommandType,"peregrine","interval",3,(0,1,2));
simple_command!(ScreenStartPairCommand,ScreenStartPairCommandType,"peregrine","screen_start_pair",3,(0,1,2));
simple_command!(ScreenEndPairCommand,ScreenEndPairCommandType,"peregrine","screen_end_pair",3,(0,1,2));
simple_command!(ScreenSpanPairCommand,ScreenSpanPairCommandType,"peregrine","screen_span_pair",3,(0,1,2));

simple_command!(PositionCommand,PositionCommandType,"peregrine","position",2,(0,1));
simple_command!(ScreenStartCommand,ScreenStartCommandType,"peregrine","screen_start",2,(0,1));
simple_command!(ScreenEndCommand,ScreenEndCommandType,"peregrine","screen_end",2,(0,1));

simple_command!(PinStartCommand,PinStartCommandType,"peregrine","pin_start",2,(0,1));
simple_command!(PinCentreCommand,PinCentreCommandType,"peregrine","pin_centre",2,(0,1));
simple_command!(PinEndCommand,PinEndCommandType,"peregrine","pin_end",2,(0,1));

simple_command!(PatinaFilledCommand,PatinaFilledCommandType,"peregrine","patina_filled",2,(0,1));
simple_command!(PatinaHollowCommand,PatinaHollowCommandType,"peregrine","patina_hollow",2,(0,1));
simple_command!(DirectColourCommand,DirectColourCommandType,"peregrine","colour",4,(0,1,2,3));
