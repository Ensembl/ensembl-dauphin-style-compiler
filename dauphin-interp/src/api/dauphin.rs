use anyhow::{ self, Context };
use std::collections::HashMap;
use std::rc::Rc;
use crate::command::{ CommandInterpretSuite, InterpreterLink };
use crate::runtime::{ InterpContext, InterpretInstance, PayloadFactory, StandardInterpretInstance };
use crate::util::DauphinError;
use serde_cbor::Value as CborValue;

pub struct Dauphin {
    suite: CommandInterpretSuite,
    mapping: HashMap<String,Rc<DauphinInstance>>,
    payloads: HashMap<(String,String),Box<dyn PayloadFactory>>
}

struct DauphinInstance {
    linker: InterpreterLink
}

impl DauphinInstance {
    fn new(suite: &CommandInterpretSuite, cbor: &CborValue) -> anyhow::Result<(DauphinInstance,Vec<String>)> {
        let linker = InterpreterLink::new(suite,&cbor).context("linking binary")?;
        let progs = linker.list_programs();
        Ok((DauphinInstance { linker },progs))
    }

    fn run_stepwise(&self, name: &str, payloads: &HashMap<(String,String),Box<dyn PayloadFactory>>, more_payloads: &HashMap<(String,String),Box<dyn PayloadFactory>>) -> anyhow::Result<impl InterpretInstance> {
        let mut interp = StandardInterpretInstance::new(&self.linker,name)?;
        interp.context_mut().add_payloads(payloads);
        interp.context_mut().add_payloads(more_payloads);
        Ok(interp)
    }
}

impl Dauphin {
    pub fn new(suite: CommandInterpretSuite) -> Dauphin {
        Dauphin {
            suite,
            mapping: HashMap::new(),
            payloads: HashMap::new()
        }
    }

    pub fn add_payload_factory(&mut self, module: &str, name: &str, pf: Box<dyn PayloadFactory>) {
        self.payloads.insert((module.to_string(),name.to_string()),pf);
    }

    pub fn add_binary(&mut self, binary_name: &str, cbor: &CborValue) -> anyhow::Result<()> {
        let (instance,programs) = DauphinInstance::new(&self.suite,cbor)?;
        let instance = Rc::new(instance);
        for program in &programs {
            self.mapping.insert(program.to_string(),instance.clone());
        }
        Ok(())
    }

    pub fn list(&self) -> Vec<String> {
        self.mapping.keys().cloned().collect()
    }

    pub fn run_stepwise(&self, binary_name: &str, name: &str, more_payloads: &HashMap<(String,String),Box<dyn PayloadFactory>>) -> anyhow::Result<impl InterpretInstance> {
        let instance = self.mapping.get(name).ok_or(DauphinError::runtime(&format!("No such program: {}",name)))?;
        instance.run_stepwise(name,&self.payloads,more_payloads)
    }
}
