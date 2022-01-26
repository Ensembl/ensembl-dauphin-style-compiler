use std::{collections::{HashMap}, mem::replace};
use crate::{ command::Operand, earpfile::EarpFileWriter, error::AssemblerError, parser::{AssemblyLocation, ParseStatement, load_source_file, ParseOperand}, rellabels::RelativeLabelContext, suite::Suite, lookup::Lookup, instructionset::{InstructionSetId, ArgSpec, ArgType}, assets::{AssetLoad}};

#[derive(Debug)]
pub(crate) struct AssembleFile {
    statements: Vec<ParseStatement>,
    rel_label_maker: RelativeLabelContext,
    rel_labels: HashMap<String,i64>,
    lookup: Lookup,
    file_path: Option<String>
}

impl AssembleFile {
    fn new(contents: &str, file_path: Option<&str>) -> Result<AssembleFile,AssemblerError> {
        let statements = load_source_file(contents)?;
        Ok(AssembleFile {
            statements,
            rel_label_maker: RelativeLabelContext::new(),
            rel_labels: HashMap::new(),
            lookup: Lookup::new(),
            file_path: file_path.map(|x| x.to_string())
        })
    }

    pub(crate) fn resolve_label(&self, assemble:&Assemble, location: &AssemblyLocation) -> Result<i64,AssemblerError> {
        let (source,label) = match location {
            AssemblyLocation::Here(delta) => {
                let target = assemble.pc+delta;
                if target < 0 || target >= assemble.max_pc {
                    return Err(AssemblerError::BadHereLabel("Here label out of range".to_string()));
                }
                return Ok(target);
            },
            AssemblyLocation::Label(label) => {
                (&assemble.labels,label.to_string())
            },
            AssemblyLocation::RelativeLabel(label,fwd) => {
                let suffix = if *fwd { "f" } else { "r" };
                (&self.rel_labels,format!("{}{}",label,suffix))
            }
        };
        if let Some(location) = source.get(&label) {
            Ok(*location)
        } else {
            Err(AssemblerError::UnknownLabel(label))
        }
    }

    fn find_includes(&mut self, assemble: &mut Assemble) -> Result<(),AssemblerError> {
        let self_path = self.file_path.clone();
        for statement in &mut self.statements {
            match statement {
                ParseStatement::Include(path,file) => {
                    let source_file = assemble.suite.source_loader().make_load_file(path, &self_path,false)?;
                    let assemble_file= AssembleFile::new(&source_file.load_string()?,Some(source_file.file_path()))?;
                    file.replace(assemble_file);
                },
                _ => {}
            }
        }
        Ok(())
    }

    fn set_labels(&mut self, assemble: &mut Assemble) -> Result<(),AssemblerError> {
        for statement in &mut self.statements {
            match statement {
                ParseStatement::Instruction(_,_,_) => {
                    assemble.pc += 1;
                    assemble.max_pc += 1;
                },
                ParseStatement::Program(program) => {
                    if assemble.labels.contains_key(program) {
                        return Err(AssemblerError::DuplicateLabel(format!("program:{}",program)));
                    }
                    assemble.earp_file.add_entry_point(program,assemble.pc);
                },
                ParseStatement::Label(label) => {
                    if assemble.labels.contains_key(label) {
                        return Err(AssemblerError::DuplicateLabel(label.to_string()))
                    }
                    assemble.labels.insert(label.to_string(),assemble.pc);
                },
                ParseStatement::RelativeLabel(label) => {
                    self.rel_label_maker.add_label(assemble.pc,label);
                },
                ParseStatement::Include(_,Some(include)) => {
                    include.set_labels(assemble)?;
                },
                _ => {}
            }
        }
        Ok(())
    }

    fn try_arg(&self, argument: &ParseOperand, spec: &ArgType) -> bool {
        let allow_jumps = match spec {
            ArgType::Any => { return true; },
            ArgType::Jump => { true },
            ArgType::Register => { false }
        };
        match argument {
            ParseOperand::Register(_) => true,
            ParseOperand::UpRegister(_) => true,
            ParseOperand::Location(_) => allow_jumps,
            _ => false
        }
    }

    fn try_argspec(&self, arguments: &[ParseOperand], specs: &[ArgType]) -> bool {
        if arguments.len() != specs.len() { return false; }
        for (arg,spec) in arguments.iter().zip(specs.iter()) {
            if !self.try_arg(arg,spec) { return false; }
        }
        true
    }

    fn check_argspec(&self, arguments: &[ParseOperand], argspec: &ArgSpec) -> Result<(),AssemblerError> {
        match argspec {
            ArgSpec::Any => { Ok(()) },
            ArgSpec::Specific(specific) => {
                for bundle in specific.iter() {
                    if self.try_argspec(arguments,bundle) {
                        return Ok(());
                    }
                }
                let got = arguments.iter().map(|x| x.type_display()).collect::<Vec<_>>();
                let expected = format!("{}",argspec);
                Err(AssemblerError::BadOperandTypes(format!("got [{}], expected {}",got.join(", "),expected)))
            }
        }
    }

    fn add_instructions(&mut self, assemble: &mut Assemble) -> Result<(),AssemblerError> {
        let mut statements = replace(&mut self.statements,vec![]);
        for statement in &mut statements {
            match statement {
                ParseStatement::Instruction(prefix,identifier,arguments) => {
                    self.rel_label_maker.fix_labels(assemble.pc, &mut self.rel_labels);
                    let prefix = prefix.as_ref().map(|x| x.as_str());
                    let (opcode,argspec) = self.lookup.lookup(assemble.earp_file.set_mapper_mut(),&prefix,&identifier)?;
                    self.check_argspec(arguments,&argspec)?;
                    let operands = arguments.iter()
                        .map(|x| Operand::new(x,assemble,self))
                        .collect::<Result<Vec<_>,_>>()?;
                    assemble.earp_file.add_instruction(opcode,&operands);
                    assemble.pc += 1;
                },
                ParseStatement::InstructionsDecl(prefix,name,version) => {
                    self.lookup.add_mapping(prefix, &InstructionSetId(name.to_string(),*version));
                },
                ParseStatement::AssetDecl(name,format,source,path) => {
                    assemble.earp_file.assets_mut().add(name,format,source,path,&self.file_path)?;
                },
                ParseStatement::Include(_,Some(include)) => {
                    include.add_instructions(assemble)?;
                },
                _ => {}
            }
        }
        self.statements = statements;
        Ok(())
    }

    fn step_resolve(&mut self, assemble: &mut Assemble) -> Result<(),AssemblerError> {
        self.set_labels(assemble)?;
        Ok(())
    }

    fn step_generate(&mut self, assemble: &mut Assemble) -> Result<(),AssemblerError> {
        self.add_instructions(assemble)?;
        Ok(())
    }
}

pub struct Assemble<'t> {
    pc: i64,
    max_pc: i64,
    labels: HashMap<String,i64>,
    source: Vec<AssembleFile>,
    earp_file: EarpFileWriter<'t>,
    suite: &'t Suite
}

impl<'t> Assemble<'t> {
    pub fn new(suite: &'t Suite) -> Assemble<'t> {
        Assemble {
            pc: 0,
            max_pc: 0,
            labels: HashMap::new(),
            source: vec![],
            earp_file: EarpFileWriter::new(suite),
            suite
        }
    }

    fn reset(&mut self) { self.pc = 0; }

    pub fn into_earpfile(self) -> EarpFileWriter<'t> { self.earp_file }

    pub(crate) fn add_source(&mut self, contents: &str, file_path: Option<&str>) -> Result<(),AssemblerError> {
        self.source.push(AssembleFile::new(&contents,file_path)?);
        Ok(())
    }

    pub fn add_file(&mut self, file_path: &str) -> Result<(),AssemblerError> {
        let loader = self.suite.source_loader().make_load_file(file_path, &None, true)?;
        let contents = loader.load_string()?;
        self.add_source(&contents,Some(loader.file_path()))
    }

    pub fn assemble(&mut self) -> Result<(),AssemblerError> {
        let mut sources = replace(&mut self.source,vec![]);
        for source in &mut sources {
            source.find_includes(self)?;
        }
        self.reset();
        for source in &mut sources {
            source.step_resolve(self)?;
        }
        self.reset();
        for source in &mut sources {
            source.step_generate(self)?;
        }
        self.source = sources;
        Ok(())
    }
}

// XXX rename

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use minicbor::Encoder;
    use peregrine_cli_toolkit::hexdump;

    use crate::{testutil::{no_error, yes_error, test_suite, build}, suite::Suite, opcodemap::load_opcode_map, hexfile::load_hexfile, command::{Command, Operand}, assemble::{Assemble}, instructionset::{ArgSpec, ArgType}};

    #[test]
    fn assemble_smoke() {
        let suite = test_suite();
        let mut assembler = Assemble::new(&suite);
        no_error(assembler.add_source(&include_str!("test/test.earp"),None));
        no_error(assembler.assemble());
        let mut out = vec![];
        let mut encoder = Encoder::new(&mut out);
        no_error(encoder.encode(&assembler.into_earpfile()));
        let cmp = no_error(load_hexfile(include_str!("test/assembler/smoke-earp.hex")));
        print!("{}",hexdump(&out));
        assert_eq!(cmp,out);
    }

    #[test]
    fn test_labels() {
        let suite = test_suite();
        let source = no_error(build(&suite,include_str!("test/assembler-labels/labels.earp")));
        assert_eq!(source,vec![
            Command(5, vec![]),
            Command(5, vec![]),
            Command(1, vec![Operand::Integer(1)])]);
    }

    #[test]
    fn test_labels_dup() {
        let suite = test_suite();
        let err = yes_error(build(&suite,include_str!("test/assembler-labels/dup.earp"))).to_string();
        assert!(err.to_lowercase().contains("duplicate label"));
    }

    #[test]
    fn test_labels_missing() {
        let suite = test_suite();
        let err = yes_error(build(&suite,include_str!("test/assembler-labels/missing.earp"))).to_string();
        assert!(err.to_lowercase().contains("unknown label"));
    }

    fn get_gotos(contents: &str) -> Vec<i64> {
        let suite = test_suite();
        let mut gotos = vec![];
        for cmd in no_error(build(&suite,contents)) {
            if cmd.0 == 1 {
                assert_eq!(cmd.1.len(),1);
                match cmd.1[0] {
                    Operand::Integer(v) => { gotos.push(v); },
                    _ => {}
                }
            }
        }
        gotos
    }

    #[test]
    fn test_rel_labels() {
        assert_eq!(vec![1,1,5,5,5,5,9,9],get_gotos(include_str!("test/assembler-labels/rel.earp")));
    }

    #[test]
    fn test_rel_labels_too_soon() {
        let suite = test_suite();
        yes_error(build(&suite,include_str!("test/assembler-labels/rel-too-soon.earp")));
    }

    #[test]
    fn test_rel_labels_too_late() {
        let suite = test_suite();
        yes_error(build(&suite,include_str!("test/assembler-labels/rel-too-late.earp")));
    }

    #[test]
    fn test_rel_labels_multi() {
        assert_eq!(vec![5,1,1,5,5,5,5,9,9,5],get_gotos(include_str!("test/assembler-labels/rel-multi.earp")));
    }

    #[test]
    fn test_here_labels() {
        assert_eq!(vec![0,2,0,3,5,3],get_gotos(include_str!("test/assembler-labels/here.earp")));
    }

    #[test]
    fn test_here_labels_start() {
        let suite = test_suite();
        let err = yes_error(build(&suite,include_str!("test/assembler-labels/here-start.earp"))).to_string();
        assert!(err.to_lowercase().contains("bad here"));
    }

    #[test]
    fn test_here_labels_end() {
        let suite = test_suite();
        let err = yes_error(build(&suite,include_str!("test/assembler-labels/here-end.earp"))).to_string();
        assert!(err.to_lowercase().contains("bad here"));
    }

    #[test]
    fn test_opcode_mapping() {
        let mut suite = Suite::new();
        for set in no_error(load_opcode_map(include_str!("test/assembler/full-test.map"))) {
            suite.add_instruction_set(set);
        }
        let commands = no_error(build(&suite,include_str!("test/assembler/opcode-mapping.earp")));
        let mut opcodes= vec![];
        for Command(opcode,_) in &commands {
            opcodes.push(*opcode);
        }
        assert_eq!(vec![
                0,1,2,3,5,
                6,7,8,9,11,12,
                13,14,15],
            opcodes);
    }

    #[test]
    fn test_program_label() {
        let suite = test_suite();
        let mut assembler = Assemble::new(&suite);
        no_error(assembler.add_source(&include_str!("test/assembler-labels/labels-program.earp"),None));
        no_error(assembler.assemble());
        let earp_file = assembler.into_earpfile();
        let mut out = vec![];
        for (label,pc) in earp_file.entry_points() {
            out.push((label.to_string(),*pc));
        }
        out.sort();
        assert_eq!(vec![("test1".to_string(),0),("test2".to_string(),0),("test3".to_string(),1)],out);
    }

    fn unshape(mut v: u64) -> String {
        let mut out = String::new();
        loop {
            let c = match v%4 {
                1 => { "r" },
                2 => { "u" },
                3 => { "c" },
                _ => { break; },
            };
            out += c;
            v /= 4;
        }
        out.chars().rev().collect::<String>()
    }

    #[test]
    fn test_type_value() {
        let suite = test_suite();
        let source = no_error(build(&suite,include_str!("test/assembler/shape.earp")));
        let mut shapes = vec![];
        for shape in &source {
            shapes.push(unshape(shape.type_value()));
        }
        assert_eq!(vec![
            "c","c","c","c",

            "","r","u","c",

            "rr","ru","rc",
            "ur","uu","uc",
            "cr","cu","cc",

            "rrr","rru","rrc",
            "rur","ruu","ruc",
            "rcr","rcu","rcc",

            "urr","uru","urc",
            "uur","uuu","uuc",
            "ucr","ucu","ucc",

        ],shapes);        
    }

    #[test]
    fn test_collide_ok() {
        let mut suite = Suite::new();
        for set in no_error(load_opcode_map(include_str!("test/assembler/collide.map"))) {
            suite.add_instruction_set(set);
        }
        no_error(build(&suite,include_str!("test/assembler/collide-ok.earp")));
    }

    #[test]
    fn test_collide_bad() {
        let mut suite = Suite::new();
        for set in no_error(load_opcode_map(include_str!("test/assembler/collide.map"))) {
            suite.add_instruction_set(set);
        }
        let e = yes_error(build(&suite,include_str!("test/assembler/collide-bad.earp")));
        assert!(e.to_string().to_lowercase().contains("duplicate"));
    }

    #[test]
    fn test_include_smoke() {
        let suite = test_suite();
        let mut assembler = Assemble::new(&suite);
        no_error(assembler.add_source(&include_str!("test/includer/include.earp"),Some("src/test/includer/include.earp")));
        no_error(assembler.assemble());
        let mut out = vec![];
        let mut encoder = Encoder::new(&mut out);
        no_error(encoder.encode(&assembler.into_earpfile()));
        let cmp = no_error(load_hexfile(include_str!("test/assembler/smoke-earp.hex")));
        print!("{}",hexdump(&out));
        assert_eq!(cmp,out);
    }

    #[test]
    fn test_include_labels() {
        let suite = test_suite();
        let mut assembler = Assemble::new(&suite);
        no_error(assembler.add_source(&include_str!("test/includer/labels.earp"),Some("src/test/includer/labels.earp")));
        no_error(assembler.assemble());
    }

    #[test]
    fn test_include_labels_dup() {
        let suite = test_suite();
        let mut assembler = Assemble::new(&suite);
        no_error(assembler.add_source(&include_str!("test/includer/labels-dup.earp"),Some("src/test/includer/labels-dup.earp")));
        let e = yes_error(assembler.assemble()).to_string();
        assert!(e.to_lowercase().contains("duplicate label"));
        assert!(e.to_lowercase().contains("test"));
    }

    #[test]
    fn test_include_labels_no_rel() {
        let suite = test_suite();
        let mut assembler = Assemble::new(&suite);
        no_error(assembler.add_source(&include_str!("test/includer/no-rel.earp"),Some("src/test/includer/no-rel.earp")));
        let e = yes_error(assembler.assemble()).to_string();
        assert!(e.to_lowercase().contains("unknown label"));
        assert!(e.to_lowercase().contains("1r"));
    }

    #[test]
    fn test_local_on_include() {
        let mut suite = test_suite();
        suite.source_loader_mut().add_search_path("1");
        suite.source_loader_mut().add_search_path("2");
        let mut assembler = Assemble::new(&suite);
        no_error(assembler.add_source(&include_str!("test/includer/2/local.earp"),Some("src/test/includer/2/local.earp")));
        no_error(assembler.assemble());
    }

    #[test]
    fn test_no_search_on_include_1() {
        let mut suite = test_suite();
        suite.source_loader_mut().add_search_path("1");
        suite.source_loader_mut().add_search_path("2");
        let mut assembler = Assemble::new(&suite);
        no_error(assembler.add_source(&include_str!("test/includer/1/no-search.earp"),Some("src/test/includer/1/no-search.earp")));
        let e = yes_error(assembler.assemble()).to_string();
        assert!(e.contains("no such path"))
    }

    #[test]
    fn test_no_search_on_include_2() {
        let mut suite = test_suite();
        suite.source_loader_mut().add_search_path("1");
        suite.source_loader_mut().add_search_path("2");
        let mut assembler = Assemble::new(&suite);
        no_error(assembler.add_source(&include_str!("test/includer/2/no-search.earp"),Some("src/test/includer/2/no-search.earp")));
        no_error(assembler.assemble());

    }

    const ALL_OPERANDS : &[&str] = &["\"hello\"","12","false","r3","u1",".x","@",".1r"];
    const JUMP_OPERANDS :  &[&str] = &["r3","u1",".x","@",".1r"];
    const REG_OPERANDS :  &[&str]  = &["r3","u1"];    

    fn operand_list(arg: &ArgType, valid: bool) -> &[&str] {
        if valid {
            match arg {
                ArgType::Any => ALL_OPERANDS,
                ArgType::Jump => JUMP_OPERANDS,
                ArgType::Register => REG_OPERANDS
            }
        } else {
            ALL_OPERANDS
        }
    }

    fn example_operands(bundle: &[ArgType], valid: bool, mut variety: u64) -> Option<String> {
        let mut out = vec![];
        for arg in bundle {
            let choice = operand_list(arg,valid);
            let len = choice.len() as u64;
            out.push(choice[(variety % len) as usize]);
            variety /= len;
        }
        if variety == 0 {
            Some(out.join(", "))
        } else {
            None
        }
    }

    fn all_example_operands(bundle: &[ArgType], valid: bool) -> Vec<String> {
        let mut out = vec![];
        let mut variety = 0;
        while let Some(result) = example_operands(bundle,valid,variety) {
            out.push(result);
            variety += 1;
        }
        out
    }

    fn prepare_instruction_bundle<'a>(suite: &'a Suite, name: &str, operand: &str) -> Assemble<'a> {
        let mut assembler = Assemble::new(&suite);
        let source = format!("{} {}",name,operand);
        println!("testing \"{}\"",source);
        let source = format!("$instructions std/1\n$instructions console/0\nx:\n 1:\n{}",source);
        no_error(assembler.add_source(&source,None));
        assembler
    }

    fn test_instruction_bundle(name: &str, operands: Vec<String>, valid: bool) {
        println!("{} : {:?} {:?}",name,operands,valid);
        let suite = test_suite();
        for operand in operands {
            let mut assembler = prepare_instruction_bundle(&suite,name,&operand);
            if valid {
                no_error(assembler.assemble());
            } else {
                yes_error(assembler.assemble());
            }
        }
    }

    #[test]
    fn test_invalid_opcode() {
        for instruction_set in no_error(load_opcode_map(include_str!("test/test.map"))) {
            for (name,_,spec) in instruction_set.opcodes() {
                /* check valid combinations */
                match &spec {
                    ArgSpec::Specific(s) => {
                        /* test valid combinations */
                        let mut all_good_examples = HashSet::new();
                        let mut all_examples = HashSet::new();
                        for bundle in s.iter() {
                            let good_examples = all_example_operands(bundle,true);
                            all_good_examples.extend(good_examples.iter().cloned());
                            all_examples.extend(all_example_operands(bundle,false));
                            test_instruction_bundle(name,good_examples,true);
                        }
                        /* test invalid examples */
                        let bundle = all_examples.difference(&all_good_examples).cloned().collect::<Vec<_>>();
                        test_instruction_bundle(name,bundle,false);
                    },
                    ArgSpec::Any => {
                        for len in 0..3 {
                            let bundle = vec![ArgType::Any;len];
                            let operands = all_example_operands(&bundle,true);
                            test_instruction_bundle(name,operands,true);
                        }
                    }
                }
                println!("{} -> {}",name,spec);
            }
        }    
    }
}
