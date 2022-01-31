use std::convert::TryInto;

use earp_runtime::{runtime::command::{Command, OperandSpec, CommandSpec}, commands::baseutils::get_any};

pub fn print_command() -> Command {
    Command::new(|context,operands| {
        let value = get_any(context,&operands[1])?;
        let coerced = value.coerce_string().unwrap_or_else(|| "*unprintable*".to_string());
        println!("{}",coerced);
        Ok(())
    },CommandSpec {
        operand_spec: vec![OperandSpec::Any]
    })
}
