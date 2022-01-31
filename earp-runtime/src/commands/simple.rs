use std::sync::Arc;

use crate::runtime::command::{Command, CommandSpec, OperandSpec};

use super::baseutils::{get_any, set};

pub fn copy_command() -> Command {
    Command::new(|context,operands| {
        set(context,&operands[0],get_any(context,&operands[1])?)
    },CommandSpec {
        operand_spec: vec![OperandSpec::Register,OperandSpec::Any]
    })
}

pub fn halt_command() -> Command {
    Command::new(|context,_operands| {
        context.halt();
        Ok(())
    },CommandSpec {
        operand_spec: vec![]
    })
}

pub fn coerce_string_command() -> Command {
    Command::new(|context,operands| {
        let value = get_any(context,&operands[1])?;
        let coerced = value.coerce_string().unwrap_or_else(|| format!("*{}*",value.type_name()));
        set(context,&operands[0],Arc::new(Box::new(coerced)))
    },CommandSpec {
        operand_spec: vec![OperandSpec::Register,OperandSpec::Any]
    })
}
