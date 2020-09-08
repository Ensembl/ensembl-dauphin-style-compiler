use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use dauphin_compile::cli::Config;
use dauphin_compile::command::{ 
    Command, CommandSchema, CommandType, CommandTrigger, CompLibRegister, Instruction, PreImagePrepare, PreImageOutcome, InstructionType,
    TimeTrialCommandType, CompilerLink, trial_write, trial_signature, TimeTrial, trial_write_str
};
use dauphin_interp::command::{ Identifier, InterpCommand };
use dauphin_interp::runtime::{ InterpContext, Register };
use dauphin_interp::types::{ RegisterSignature, to_xstructure, XStructure, VectorRegisters, FullType, MemberMode, BaseType, ComplexPath, MemberDataFlow };
use dauphin_interp::util::DauphinError;
use dauphin_interp::util::cbor::{ cbor_make_map, cbor_map };
use serde_cbor::Value as CborValue;
use dauphin_compile::model::PreImageContext;

struct LookupTimeTrial();

impl TimeTrialCommandType for LookupTimeTrial {
    fn timetrial_make_trials(&self) -> (i64,i64) { (1,10) }

    fn global_prepare(&self, context: &mut InterpContext, t: i64) {
        let t = t as usize;
        trial_write_str(context,1,t*50,|x| x*2); // needles
        trial_write_str(context,2,t*100,|x| x); // data
        trial_write(context,3,1,|_| 0); // offset
        trial_write(context,4,1,|_| t*100); // len
        trial_write(context,5,1,|_| 1); // default
        context.registers_mut().commit();
    }

    fn timetrial_make_command(&self, _: i64, _linker: &CompilerLink, _config: &Config) -> anyhow::Result<Instruction> {
        let regs = (0..6).map(|i| Register(i)).collect();
        let sig = trial_signature(&vec![(MemberMode::Out,0,BaseType::NumberType),(MemberMode::In,0,BaseType::NumberType),
                                        (MemberMode::In,0,BaseType::NumberType),(MemberMode::In,0,BaseType::NumberType),(MemberMode::In,0,BaseType::NumberType),
                                        (MemberMode::In,0,BaseType::NumberType)]);
        Ok(Instruction::new(InstructionType::Call(Identifier::new("std","lookup"),true,sig,vec![
            MemberDataFlow::Out,MemberDataFlow::In,
            MemberDataFlow::In,MemberDataFlow::In,MemberDataFlow::In,
            MemberDataFlow::In
        ]),regs))
    }
}

pub struct LookupCommand(Register,Register,Register,Register,Register,Register,Option<TimeTrial>);

impl Command for LookupCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![self.0.serialize(),self.1.serialize(),self.2.serialize(),self.3.serialize(),self.4.serialize(),self.5.serialize()]))
    }

    fn simple_preimage(&self, context: &mut PreImageContext) -> anyhow::Result<PreImagePrepare> { 
        Ok(if context.is_reg_valid(&self.1) && context.is_reg_valid(&self.2) && 
                context.is_reg_valid(&self.3) && context.is_reg_valid(&self.4) && 
                context.is_reg_valid(&self.5) && !context.is_last() {
            PreImagePrepare::Replace
        } else if let Some(a) = context.get_reg_size(&self.1) {
            PreImagePrepare::Keep(vec![(self.0.clone(),a)])
        } else {
            PreImagePrepare::Keep(vec![])
        })
    }

    fn preimage_post(&self, _context: &mut PreImageContext) -> anyhow::Result<PreImageOutcome> {
        Ok(PreImageOutcome::Constant(vec![self.0]))
    }

    fn execution_time(&self, context: &PreImageContext) -> f64 {
        if let Some(haystack) = context.get_reg_size(&self.2) {
            if let Some(needles) = context.get_reg_size(&self.1) {
                let t = (haystack as f64/100.).max(needles as f64/50.);
                return self.6.as_ref().map(|x| x.evaluate(t)).unwrap_or(1.);
            }   
        }
        1.
    }
}

struct InTimeTrial();

impl TimeTrialCommandType for InTimeTrial {
    fn timetrial_make_trials(&self) -> (i64,i64) { (1,10) }

    fn global_prepare(&self, context: &mut InterpContext, t: i64) {
        let t = t as usize;
        trial_write_str(context,1,t*50,|x| x*4); // needles
        trial_write_str(context,2,t*100,|x| x); // data
        trial_write(context,3,1,|_| 0); // offset
        trial_write(context,4,1,|_| t*100); // len
        context.registers_mut().commit();
    }

    fn timetrial_make_command(&self, _: i64, _linker: &CompilerLink, _config: &Config) -> anyhow::Result<Instruction> {
        let regs = (0..5).map(|i| Register(i)).collect();
        let sig = trial_signature(&vec![(MemberMode::Out,0,BaseType::NumberType),(MemberMode::In,0,BaseType::NumberType),
                                        (MemberMode::In,0,BaseType::NumberType),(MemberMode::In,0,BaseType::NumberType),(MemberMode::In,0,BaseType::NumberType)]);
        Ok(Instruction::new(InstructionType::Call(Identifier::new("std","in"),true,sig,vec![
            MemberDataFlow::Out,MemberDataFlow::In,
            MemberDataFlow::In,MemberDataFlow::In,MemberDataFlow::In
        ]),regs))
    }
}

pub struct InCommand(Register,Register,Register,Register,Register,Option<TimeTrial>);

impl Command for InCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![self.0.serialize(),self.1.serialize(),self.2.serialize(),self.3.serialize(),self.4.serialize()]))
    }

    fn simple_preimage(&self, context: &mut PreImageContext) -> anyhow::Result<PreImagePrepare> { 
        Ok(if context.is_reg_valid(&self.1) && context.is_reg_valid(&self.2) && 
                context.is_reg_valid(&self.3) && context.is_reg_valid(&self.4) && 
                !context.is_last() {
            PreImagePrepare::Replace
        } else if let Some(a) = context.get_reg_size(&self.1) {
            PreImagePrepare::Keep(vec![(self.0.clone(),a)])
        } else {
            PreImagePrepare::Keep(vec![])
        })
    }

    fn preimage_post(&self, _context: &mut PreImageContext) -> anyhow::Result<PreImageOutcome> {
        Ok(PreImageOutcome::Constant(vec![self.0]))
    }

    fn execution_time(&self, context: &PreImageContext) -> f64 {
        if let Some(haystack) = context.get_reg_size(&self.2) {
            if let Some(needles) = context.get_reg_size(&self.1) {
                let t = (haystack as f64/100.).max(needles as f64/50.);
                return self.5.as_ref().map(|x| x.evaluate(t)).unwrap_or(1.);
            }   
        }
        1.
    }
}

struct IndexTimeTrial();

impl TimeTrialCommandType for IndexTimeTrial {
    fn timetrial_make_trials(&self) -> (i64,i64) { (1,10) }

    fn global_prepare(&self, context: &mut InterpContext, t: i64) {
        let t = t as usize;
        trial_write(context,1,1,|_| 0); // offset
        trial_write(context,2,1,|_| t*100); // len
        trial_write_str(context,3,t*100,|x| x); // data
        trial_write_str(context,4,t*50,|x| x*2); // needles
        context.registers_mut().commit();
    }

    fn timetrial_make_command(&self, _: i64, _linker: &CompilerLink, _config: &Config) -> anyhow::Result<Instruction> {
        let regs = (0..5).map(|i| Register(i)).collect();
        let sig = trial_signature(&vec![(MemberMode::Out,0,BaseType::NumberType),(MemberMode::In,0,BaseType::NumberType),
                                        (MemberMode::In,0,BaseType::NumberType),(MemberMode::In,0,BaseType::NumberType),(MemberMode::In,0,BaseType::NumberType)]);
        Ok(Instruction::new(InstructionType::Call(Identifier::new("std","_index"),true,sig,vec![
            MemberDataFlow::Out,MemberDataFlow::In,
            MemberDataFlow::In,MemberDataFlow::In,MemberDataFlow::In
        ]),regs))
    }
}

pub struct RealIndexCommand(Register,Register,Register,Register,Register,Option<TimeTrial>);

impl Command for RealIndexCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![self.0.serialize(),self.1.serialize(),self.2.serialize(),
                     self.3.serialize(),self.4.serialize()]))
    }

    fn simple_preimage(&self, context: &mut PreImageContext) -> anyhow::Result<PreImagePrepare> { 
        Ok(if context.is_reg_valid(&self.1) && context.is_reg_valid(&self.2) && 
                context.is_reg_valid(&self.3) && context.is_reg_valid(&self.4) && 
                !context.is_last() {
            PreImagePrepare::Replace
        } else {
            let mut sizes = vec![];
            if let Some(a) = context.get_reg_size(&self.3) {
                sizes.push((self.0.clone(),a));
            }
            PreImagePrepare::Keep(sizes)
        })
    }

    fn preimage_post(&self, _context: &mut PreImageContext) -> anyhow::Result<PreImageOutcome> {
        Ok(PreImageOutcome::Constant(vec![self.0]))
    }

    fn execution_time(&self, context: &PreImageContext) -> f64 {
        if let Some(haystack) = context.get_reg_size(&self.3) {
            if let Some(needles) = context.get_reg_size(&self.4) {
                let t = (haystack as f64/100.).max(needles as f64/50.);
                return self.5.as_ref().map(|x| x.evaluate(t)).unwrap_or(1.);
            }   
        }
        1.
    }
}

fn match_xs_map(out: &mut Vec<(Rc<RefCell<VectorRegisters>>,Rc<RefCell<VectorRegisters>>)>,
                a: &HashMap<String,Rc<XStructure<VectorRegisters>>>, b: &HashMap<String,Rc<XStructure<VectorRegisters>>>) -> anyhow::Result<()> {
    let mut a_keys : Vec<&String> = a.keys().collect();
    let mut b_keys : Vec<&String> = b.keys().collect();
    a_keys.sort();
    b_keys.sort();
    if a_keys != b_keys {
        return Err(DauphinError::source("mismatched types in index"));
    }
    for key in &a_keys {
        match_xs(out,a.get(*key).unwrap(),b.get(*key).unwrap())?;
    }
    Ok(())
}

fn match_xs(out: &mut Vec<(Rc<RefCell<VectorRegisters>>,Rc<RefCell<VectorRegisters>>)>, 
            xs_out: &XStructure<VectorRegisters>, xs_in: &XStructure<VectorRegisters>) -> anyhow::Result<()> {
    match (xs_out,xs_in) {
        (XStructure::Simple(a),XStructure::Simple(b)) => {
            out.push((a.clone(),b.clone()));
        },
        (XStructure::Vector(a),XStructure::Vector(b)) => { return match_xs(out,a,b); },
        (XStructure::Struct(a_id,a_map),XStructure::Struct(b_id,b_map)) => {
            if a_id != b_id {
                return Err(DauphinError::source("mismatched types in index"));
            }
            match_xs_map(out,a_map,b_map)?;
        }
        (XStructure::Enum(a_id,a_order,a_map,a_disc),XStructure::Enum(b_id,b_order,b_map,b_disc)) => {
            if a_id != b_id || a_order != b_order {
                return Err(DauphinError::source("mismatched types in index"));
            }
            match_xs_map(out,a_map,b_map)?;
            out.push((a_disc.clone(),b_disc.clone()));
        }
        _ => { return Err(DauphinError::source("mismatched types in index")); }
    }
    Ok(())
}

fn add_to_sig(sigs: &mut RegisterSignature, mode: &MemberMode, base: &BaseType) {
    let mut cr = FullType::new_empty(*mode);
    cr.add(ComplexPath::new_empty().add_levels(0),VectorRegisters::new(0,base.clone()));
    sigs.add(cr);
}

pub struct IndexCommand(RegisterSignature,Vec<Register>);

impl IndexCommand {
    fn add_internal(&self, instrs: &mut Vec<Instruction>, out_pos: usize, in_pos: (usize,usize,usize), needle: &Register, base: Option<&BaseType>) {
        let out_reg = self.1[out_pos];
        let in_regs = (self.1[in_pos.0],self.1[in_pos.1],self.1[in_pos.2]);
        let mut sigs = RegisterSignature::new();
        add_to_sig(&mut sigs,&MemberMode::Out,base.unwrap_or(&BaseType::NumberType));
        add_to_sig(&mut sigs,&MemberMode::In ,&BaseType::NumberType);
        add_to_sig(&mut sigs,&MemberMode::In ,&BaseType::NumberType);
        add_to_sig(&mut sigs,&MemberMode::In ,base.unwrap_or(&BaseType::NumberType));
        add_to_sig(&mut sigs,&MemberMode::In ,&BaseType::NumberType);
        let flows = vec![MemberDataFlow::Out,
                         MemberDataFlow::In,MemberDataFlow::In,MemberDataFlow::In,MemberDataFlow::In];
        instrs.push(Instruction::new(InstructionType::Call(Identifier::new("std","_index"),false,sigs,flows),vec![
            out_reg,in_regs.0,in_regs.1,in_regs.2,needle.clone()
        ]));
    }

    fn build_one_instr(&self, instrs: &mut Vec<Instruction>, out_vr: &VectorRegisters, index: &Register, in_vr: &VectorRegisters) -> anyhow::Result<()> {
        if in_vr.depth() > 2 {
            for depth in 0..(in_vr.depth()-2) {
                instrs.push(Instruction::new(InstructionType::Copy,vec![
                    self.1[out_vr.offset_pos(depth)?],
                    self.1[in_vr.offset_pos(depth)?]
                ]));
                instrs.push(Instruction::new(InstructionType::Copy,vec![
                    self.1[out_vr.length_pos(depth)?],
                    self.1[in_vr.length_pos(depth)?]
                ]));
            }
        }
        if in_vr.depth() > 1 {
            instrs.push(Instruction::new(InstructionType::Copy,vec![
                self.1[out_vr.data_pos()],
                self.1[in_vr.data_pos()]
            ]));
            self.add_internal(instrs,out_vr.offset_pos(out_vr.depth()-1)?,
                                     (in_vr.offset_pos( in_vr.depth()-1)?,in_vr.length_pos( in_vr.depth()-1)?,in_vr.offset_pos( in_vr.depth()-2)?),
                              index,None);
            self.add_internal(instrs,out_vr.length_pos(out_vr.depth()-1)?,
                                     (in_vr.offset_pos( in_vr.depth()-1)?,in_vr.length_pos( in_vr.depth()-1)?,in_vr.length_pos( in_vr.depth()-2)?),
                              index,None);
        } else {
            self.add_internal(instrs,out_vr.data_pos(),
                                     (in_vr.offset_pos( in_vr.depth()-1)?, in_vr.length_pos( in_vr.depth()-1)?, in_vr.data_pos()),
                              index,Some(in_vr.get_base()));
        }
        Ok(())
    }

    fn build_instrs(&self, context: &mut PreImageContext) -> anyhow::Result<Vec<Instruction>> {
        let mut instrs = vec![];
        let xs_out = to_xstructure(&self.0[0])?;
        let xs_in = to_xstructure(&self.0[2])?;
        let xs_index = to_xstructure(&self.0[1])?;
        let index_pos = if let XStructure::Simple(vr) = xs_index { 
            vr.borrow().data_pos()
        } else { 
            return Err(DauphinError::internal(file!(),line!()));
        };
        let mut out = vec![];
        let extended_xs_out = XStructure::Vector(Rc::new(xs_out));
        match_xs(&mut out,&extended_xs_out,&xs_in)?;
        for (out_vr,in_vr) in out {
            self.build_one_instr(&mut instrs,&out_vr.borrow(),&self.1[index_pos],&in_vr.borrow())?;
        }
        Ok(instrs)
    }
}

impl Command for IndexCommand {
    fn serialize(&self) -> anyhow::Result<Option<Vec<CborValue>>> {
        Ok(Some(vec![]))
    }

    fn preimage(&self, context: &mut PreImageContext, _ic: Option<Box<dyn InterpCommand>>) -> anyhow::Result<PreImageOutcome> {
        Ok(PreImageOutcome::Replace(self.build_instrs(context)?))
    }
}

pub struct LookupCommandType(Option<TimeTrial>);

impl CommandType for LookupCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 6,
            trigger: CommandTrigger::Command(Identifier::new("std","lookup"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        Ok(Box::new(LookupCommand(it.regs[0],it.regs[1],it.regs[2],it.regs[3],it.regs[4],it.regs[5],self.0.clone())))
    }
    
    fn generate_dynamic_data(&self, linker: &CompilerLink, config: &Config) -> anyhow::Result<CborValue> {
        let timings = TimeTrial::run(&LookupTimeTrial(),linker,config)?;
        Ok(cbor_make_map(&vec!["t"],vec![timings.serialize()])?)
    }

    fn use_dynamic_data(&mut self, value: &CborValue) -> anyhow::Result<()> {
        let t = cbor_map(value,&vec!["t"])?;
        self.0 = Some(TimeTrial::deserialize(&t[0])?);
        Ok(())
    }
}

pub struct InCommandType(Option<TimeTrial>);

impl CommandType for InCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 5,
            trigger: CommandTrigger::Command(Identifier::new("std","in"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        Ok(Box::new(InCommand(it.regs[0],it.regs[1],it.regs[2],it.regs[3],it.regs[4],self.0.clone())))
    }    

    fn generate_dynamic_data(&self, linker: &CompilerLink, config: &Config) -> anyhow::Result<CborValue> {
        let timings = TimeTrial::run(&InTimeTrial(),linker,config)?;
        Ok(cbor_make_map(&vec!["t"],vec![timings.serialize()])?)
    }

    fn use_dynamic_data(&mut self, value: &CborValue) -> anyhow::Result<()> {
        let t = cbor_map(value,&vec!["t"])?;
        self.0 = Some(TimeTrial::deserialize(&t[0])?);
        Ok(())
    }
}

pub struct IndexCommandType();

impl CommandType for IndexCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 0,
            trigger: CommandTrigger::Command(Identifier::new("std","index"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        if let InstructionType::Call(_,_,sig,_) = &it.itype {
            Ok(Box::new(IndexCommand(sig.clone(),it.regs.to_vec())))
        } else {
            Err(DauphinError::malformed("unexpected instruction"))
        }
    }    
}

pub struct RealIndexCommandType(Option<TimeTrial>);

impl CommandType for RealIndexCommandType {
    fn get_schema(&self) -> CommandSchema {
        CommandSchema {
            values: 5,
            trigger: CommandTrigger::Command(Identifier::new("std","_index"))
        }
    }

    fn from_instruction(&self, it: &Instruction) -> anyhow::Result<Box<dyn Command>> {
        Ok(Box::new(RealIndexCommand(
            it.regs[0],it.regs[1],it.regs[2],it.regs[3],it.regs[4],self.0.clone()
        )))
    }    

    fn generate_dynamic_data(&self, linker: &CompilerLink, config: &Config) -> anyhow::Result<CborValue> {
        let timings = TimeTrial::run(&IndexTimeTrial(),linker,config)?;
        Ok(cbor_make_map(&vec!["t"],vec![timings.serialize()])?)
    }

    fn use_dynamic_data(&mut self, value: &CborValue) -> anyhow::Result<()> {
        let t = cbor_map(value,&vec!["t"])?;
        self.0 = Some(TimeTrial::deserialize(&t[0])?);
        Ok(())
    }
}

pub(super) fn library_map_commands(set: &mut CompLibRegister) {
    set.push("lookup",Some(3),LookupCommandType(None));
    set.push("in",Some(21),InCommandType(None));
    set.push("index",None,IndexCommandType());
    set.push("_index",Some(22),RealIndexCommandType(None));
}
