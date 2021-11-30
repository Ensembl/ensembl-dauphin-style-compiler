use std::{any::Any, borrow::Cow, sync::{Arc, Mutex}};

/* TODO:
 * safe               4
 * async blocks       3
 * integrations       2
 * return values enum 1
 * resource checks    5
 */

struct EarpFrame {
    registers: Vec<Arc<Box<dyn Any>>>
}

impl EarpFrame {
    fn new() -> EarpFrame {
        EarpFrame {
            registers: vec![]
        }
    }
 
    fn get(&self, index: usize) -> &Arc<Box<dyn Any>> {
        &self.registers[index]
    }

    fn set(&mut self, index: usize, value: Arc<Box<dyn Any>>) {
        while self.registers.len() <= index {
            self.registers.push(Arc::new(Box::new(())));
        }
        self.registers[index] = value;
    }
}

#[derive(Clone)]
pub struct EarpFunction {
    closure: Arc<Box<dyn Fn(Vec<Cow<Arc<Box<dyn Any>>>>) -> EarpReturn + 'static>>
}

impl EarpFunction {
    pub fn new<F>(closure: F) -> EarpFunction where F: Fn(Vec<Cow<Arc<Box<dyn Any>>>>) -> EarpReturn + 'static {
        EarpFunction {
            closure: Arc::new(Box::new(closure))
        }
    }

    fn call(&self, values: Vec<Cow<Arc<Box<dyn Any>>>>) -> EarpReturn {
        (self.closure)(values)
    }
}

#[derive(Clone)]
pub enum EarpArgument {
    Var(usize),
    UpVar(usize),
    Literal(Arc<Box<dyn Any>>),
    Integration(usize),
    Here
}

pub enum EarpReturn {
    Value(Arc<Box<dyn Any>>),
    GotoRel(usize),
    GotoAbs(usize),
    Push,
    Pop,
    None
}

#[derive(Clone)]
pub enum EarpOutcome {
    Var(usize),
    UpVar(usize),
    None
}

impl EarpOutcome {
    fn set(&self, frames: &mut Vec<EarpFrame>, value: Arc<Box<dyn Any>>) {
        let len = frames.len();
        match self {
            EarpOutcome::Var(index) => {
                frames[len-1].set(*index,value);
            },
            EarpOutcome::UpVar(index) => {
                frames[len-2].set(*index,value);
            },
            EarpOutcome::None => {}
        }
    }
}

impl EarpArgument {
    fn value<'a>(&'a self, frames: &'a Vec<EarpFrame>, integrations: &'a Vec<Arc<Box<dyn Any>>>, pc: usize) -> Cow<'a,Arc<Box<dyn Any>>> {
        match self {
            EarpArgument::Var(index) => {
                Cow::Borrowed(frames[frames.len()-1].get(*index))
            },
            EarpArgument::UpVar(index) => {
                Cow::Borrowed(frames[frames.len()-2].get(*index))
            },
            EarpArgument::Literal(value) => {
                Cow::Borrowed(value)
            },
            EarpArgument::Integration(index) => {
                Cow::Borrowed(&integrations[*index])
            },
            EarpArgument::Here => {
                Cow::Owned(Arc::new(Box::new(pc) as Box<dyn Any>))
            }
        }
    }
}

#[derive(Clone)]
pub struct EarpStatement {
    function: EarpFunction,
    arguments: Vec<EarpArgument>,
    outcome: EarpOutcome
}

impl EarpStatement {
    pub fn new(function: &EarpFunction, arguments: Vec<EarpArgument>, outcome: EarpOutcome) -> EarpStatement {
        EarpStatement {
            function: function.clone(),
            arguments,
            outcome
        }
    }

    fn values<'a>(&'a self, frames: &'a Vec<EarpFrame>, integrations: &'a Vec<Arc<Box<dyn Any>>>, pc: usize) -> Vec<Cow<'a,Arc<Box<dyn Any>>>> {
        let mut out = vec![];
        for arg in &self.arguments {
            out.push(arg.value(frames,integrations,pc));
        }
        out
    }

    async fn step(&self, frames: &mut Vec<EarpFrame>, integrations: &mut Vec<Arc<Box<dyn Any>>>, pc: usize) -> EarpReturn {
        let values = self.values(frames,integrations,pc);
        let ret = self.function.call(values);
        match ret {
            EarpReturn::Value(v) => {
                self.outcome.set(frames,v);
                EarpReturn::None
            },
            ret => ret
        }
    }
}

pub struct EarpProgram {
    statements: Vec<EarpStatement>
}

impl EarpProgram {
    pub fn new(statements: Vec<EarpStatement>) -> EarpProgram {
        EarpProgram {
            statements
        }
    }

    pub fn statement(&self, index: usize) -> Option<&EarpStatement> {
        self.statements.get(index)
    }
}

struct EarpRuntimeState {
    program: EarpProgram,
    pc: usize,
    frames: Vec<EarpFrame>,
    integrations: Vec<Arc<Box<dyn Any>>>
}

impl EarpRuntimeState {
    fn new(program: EarpProgram) -> EarpRuntimeState {
        let frames = vec![
            EarpFrame::new(),
            EarpFrame::new(),
        ];
        EarpRuntimeState {
            program,
            pc: 0,
            frames,
            integrations: vec![]
        }
    }
}

impl EarpRuntimeState {
    fn register_integration(&mut self, index: usize, integration: Arc<Box<dyn Any>>) {
//        while self.integrations.len() <= index {
//            self.integrations.push(None);
//        }
        self.integrations[index] = integration;
    }

    fn push(&mut self) {
        self.frames.push(EarpFrame::new());
    }

    fn pop(&mut self) {
        // XXX error check
        self.frames.pop();
    }

    async fn step(&mut self) -> bool {
        if let Some(stmt) = self.program.statement(self.pc) {
            match  stmt.step(&mut self.frames,&mut self.integrations,self.pc).await {
                EarpReturn::None | EarpReturn::Value(_) => {},
                EarpReturn::GotoRel(g) => { self.pc += g-1; },
                EarpReturn::GotoAbs(g) => { self.pc = g-1; },
                EarpReturn::Push => self.push(),
                EarpReturn::Pop => self.pop()
            }
            self.pc += 1;
            false
        } else {
            true
        }
    }
}

#[derive(Clone)]
pub struct EarpRuntime {
    runtime: Arc<Mutex<EarpRuntimeState>>
}
//struct LockedEarpRuntimeState<'a>(MutexGuard<'a,EarpRuntimeState>);

impl EarpRuntime {
    pub fn new(program: EarpProgram) -> EarpRuntime {
        EarpRuntime {
            runtime: Arc::new(Mutex::new(EarpRuntimeState::new(program)))
        }
    }

    pub async fn step(&self) {
        loop {
            let mut state = self.runtime.lock().unwrap();
            if state.step().await { break; }
            drop(state);
        }
    }

    pub async fn run(&self) {
//        loop {
            self.step().await;
  //      }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;
    use futures::executor::block_on;
    use super::*;

    #[test]
    fn test_pc() {
        let push_frame = EarpFunction::new(|_| {
            EarpReturn::Push    
        });
        let pop_frame = EarpFunction::new(|_| {
            EarpReturn::Pop    
        });
        let clone = EarpFunction::new(|args| {
            EarpReturn::Value(args[0].clone().into_owned())
        });
        let nop = EarpFunction::new(|_| {
            EarpReturn::None
        });
        let print = EarpFunction::new(|args| {
            if let Some(string) = args[0].downcast_ref::<String>() {
                print!("{}\n",string);
            }
            EarpReturn::None
        });
        let num_to_string = EarpFunction::new(|args| {
            if let Some(num) = args[0].downcast_ref::<f64>() {
                EarpReturn::Value(Arc::new(Box::new(num.to_string())))
            } else if let Some(num) = args[0].downcast_ref::<usize>() {
                EarpReturn::Value(Arc::new(Box::new(num.to_string())))    
            } else {
                EarpReturn::None
            }
        });
        let here_to_r0_stmt = EarpStatement::new(&clone,vec![
            EarpArgument::Here
        ],EarpOutcome::Var(0));
        let nop_stmt = EarpStatement::new(&nop,vec![],EarpOutcome::None);
        let n2s_stmt = EarpStatement::new(&num_to_string,vec![
            EarpArgument::Var(0)
        ],EarpOutcome::Var(0));
        let print_stmt = EarpStatement::new(&print,vec![EarpArgument::Var(0)],
        EarpOutcome::None);
        let program = EarpProgram::new(vec![nop_stmt.clone(),nop_stmt.clone(),nop_stmt,here_to_r0_stmt,n2s_stmt,print_stmt]);
        /**/
        let runtime = EarpRuntime::new(program);
        block_on(runtime.run());
    }
}