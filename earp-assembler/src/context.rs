use crate::instructionsuite::EarpInstructionSuite;

pub(crate) struct EarpAssemblerContext {
    suite: EarpInstructionSuite
}

impl EarpAssemblerContext {
    fn new() -> EarpAssemblerContext {
        EarpAssemblerContext {
            suite: EarpInstructionSuite::new()
        }
    }

    pub(crate) fn instruction_suite(&self) -> &EarpInstructionSuite { &self.suite }
    fn instruction_suite_mut(&mut self) -> &mut EarpInstructionSuite { &mut self.suite }
}

#[cfg(test)]
mod test {
    use crate::opcodemap::load_opcode_map;

    #[test]
    fn test_map() {
        let suite = load_opcode_map(include_str!("./maps/std-0.map"));
        print!("{:?}",suite);
    }
}
