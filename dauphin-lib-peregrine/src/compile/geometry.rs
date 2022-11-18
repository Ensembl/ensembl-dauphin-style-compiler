use dauphin_compile::command::{ 
    Command, CommandSchema, CommandType, CommandTrigger, Instruction, InstructionType
};
use dauphin_interp::command::{ Identifier };
use dauphin_interp::runtime::{ Register };
use dauphin_interp::util::DauphinError;
use serde_cbor::Value as CborValue;
use crate::simple_command;

simple_command!(ScreenStartCommand,ScreenStartCommandType,"peregrine","screen_start",2,(0,1));
simple_command!(ScreenEndCommand,ScreenEndCommandType,"peregrine","screen_end",2,(0,1));
simple_command!(PinStartCommand,PinStartCommandType,"peregrine","pin_start",2,(0,1));
simple_command!(PinCentreCommand,PinCentreCommandType,"peregrine","pin_centre",2,(0,1));
simple_command!(PinEndCommand,PinEndCommandType,"peregrine","pin_end",2,(0,1));
simple_command!(ZMenuCommand,ZMenuCommandType,"peregrine","zmenu",2,(0,1));
simple_command!(PatinaFilledCommand,PatinaFilledCommandType,"peregrine","patina_filled",2,(0,1));
simple_command!(PatinaHollowCommand,PatinaHollowCommandType,"peregrine","patina_hollow",3,(0,1,2));
simple_command!(DirectColourCommand,DirectColourCommandType,"peregrine","direct_colour",5,(0,1,2,3,4));
simple_command!(UseAllotmentCommand,UseAllotmentCommandType,"peregrine","use_allotment",2,(0,1));
simple_command!(PenCommand,PenCommandType,"peregrine","pen",5,(0,1,2,3,4));
simple_command!(PlotterCommand,PlotterCommandType,"peregrine","plotter",3,(0,1,2));
simple_command!(SpaceBaseCommand,SpaceBaseCommandType,"peregrine","spacebase",4,(0,1,2,3));
simple_command!(SimpleColourCommand,SimpleColourCommandType,"peregrine","simple_colour",2,(0,1));
simple_command!(StripedCommand,StripedCommandType,"peregrine","striped",6,(0,1,2,3,4,5));
simple_command!(BarCommand,BarCommandType,"peregrine","barred",6,(0,1,2,3,4,5));
simple_command!(BpRangeCommand,BpRangeCommandType,"peregrine","bp_range",1,(0));
simple_command!(SpotColourCommand,SpotColourCommandType,"peregrine","spot_colour",2,(0,1));
simple_command!(PpcCommand,PpcCommandType,"peregrine","px_per_carriage",1,(0));
simple_command!(StyleCommand,StyleCommandType,"peregrine","style",3,(0,1,2));
simple_command!(PatinaMMetadataName,PatinaMetadataCommandType,"peregrine","patina_metadata",4,(0,1,2,3));
simple_command!(BackgroundCommand,BackgroundCommandType,"peregrine","background",3,(0,1,2));
simple_command!(PatinaSettingSetCommand,PatinaSettingSetCommandType,"peregrine","patina_setting_set",3,(0,1,2));
simple_command!(PatinaSettingMemberCommand,PatinaSettingMemberCommandType,"peregrine","patina_setting_member",4,(0,1,2,3));
simple_command!(PatinaSpecialZone,PatinaSpecialZoneCommandType,"peregrine","patina_special_zone",2,(0,1));

/* 0: out/patina  1: zmenu  2: key/D  3: key/A  4: key/B  5: value/D  6: value/A  7: value/B */
pub struct PatinaZMenuCommand(Register,Register,Register,Register,Register,Register,Register,Register);

impl Command for PatinaZMenuCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![
            self.0.serialize(),self.1.serialize(),self.2.serialize(),self.3.serialize(),
            self.4.serialize(),self.5.serialize(),self.6.serialize(),self.7.serialize()
        ]))
    }
}

pub struct PatinaZMenuCommandType();

impl CommandType for PatinaZMenuCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 8,
            trigger: CommandTrigger::Command(Identifier::new("peregrine","patina_zmenu"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,sig,_) = &it.itype {
            let key_vec = sig[2].iter().next().unwrap().1;
            let value_vec = sig[3].iter().next().unwrap().1;
            let pos = vec![
                sig[0].iter().next().unwrap().1.data_pos(),
                sig[1].iter().next().unwrap().1.data_pos(),
                key_vec.data_pos(),
                key_vec.offset_pos(0)?,
                key_vec.length_pos(0)?,
                value_vec.data_pos(),
                value_vec.offset_pos(0)?,
                value_vec.length_pos(0)?                
            ];
            let regs : Vec<_> = pos.iter().map(|x| it.regs[*x]).collect();
            Ok(Box::new(PatinaZMenuCommand(regs[0],regs[1],regs[2],regs[3],regs[4],regs[5],regs[6],regs[7])))
        } else {
            Err(DauphinError::internal(file!(),line!()))
        }
    }
}
