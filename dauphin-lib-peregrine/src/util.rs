use anyhow::{ anyhow as err };
use dauphin_interp::runtime::{ InterpContext };
use peregrine_data::{EachOrEvery, InstancePayload};
use crate::payloads::PeregrinePayload;

#[macro_export]
macro_rules! fixed_ty {
    (($what:ty),$value:expr) => {
        $what
    };
}

#[macro_export]
macro_rules! simple_command {
    ($name:ident, $type_name: ident, $cmd_lib: expr, $cmd_name: expr, $total:expr, ($($reg:tt),*)) => {
        pub struct $name($(crate::fixed_ty!((Register),$reg)),*);

        impl Command for $name {
            fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
                Ok(Some(vec![$(self.$reg.serialize()),*]))
            }
        }        

        pub struct $type_name();

        impl CommandType for $type_name {
            fn get_schema(&self) -> CommandSchema {
                CommandSchema {
                    values: $total,
                    trigger: CommandTrigger::Command(Identifier::new($cmd_lib,$cmd_name))
                }
            }
        
            fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
                Ok(Box::new($name($(it.regs[$reg]),*)))
            }
        }
    
    };
}

#[macro_export]
macro_rules! simple_interp_command {
    ($name:ident, $ds_name: ident, $opcode:expr, $total:expr, ($($reg:tt),*)) => {
        #[derive(Clone)]
        pub struct $name($(crate::fixed_ty!((Register),$reg)),*);

        pub struct $ds_name();

        impl CommandDeserializer for $ds_name {
            fn get_opcode_len(&self) -> anyhow::Result<Option<(u32,usize)>> { Ok(Some(($opcode,$total))) }
            fn deserialize(&self, _opcode: u32, value: &[&CborValue]) -> anyhow::Result<Box<dyn InterpCommand>> {
                Ok(Box::new($name(
                    $(Register::deserialize(&value[$reg])?),*
                )))
            }
        }
    };
}

pub(crate) fn get_instance<T>(context: &mut InterpContext, name: &str) -> anyhow::Result<T> where T: Clone + 'static {
    let mut out = None;
    if let Some(instance) = context.payload("peregrine","instance")?.as_any_mut().downcast_mut::<InstancePayload>() {
        if let Some(value) = instance.get(name) {
            if let Some(value) = value.downcast_ref::<T>() {
                out = Some(value.clone());
            }
        }
    }
    out.ok_or_else(|| err!("missing instance field {}",name))
}

pub(crate) fn get_peregrine(context: &mut InterpContext) -> anyhow::Result<&mut PeregrinePayload> {
    context.payload("peregrine","core")?.as_any_mut().downcast_mut::<PeregrinePayload>().ok_or_else(|| err!("missing peregrine data"))
}

pub(crate) fn vec_to_eoe<X>(mut input: Vec<X>) -> EachOrEvery<X> {
    if input.len() == 1 { EachOrEvery::Every(input.remove(0)) }
    else { EachOrEvery::Each(input) } 
}
