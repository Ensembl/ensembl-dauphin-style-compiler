use std::any::Any;

pub struct EarpConsoleIntegration {

}

pub fn console_intergation() -> Box<dyn Any> {
    Box::new(EarpConsoleIntegration {})
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use earp_runtime::EarpArgument;
    use earp_runtime::EarpFunction;
    use earp_runtime::EarpProgram;
    use earp_runtime::EarpResultRegister;
    use earp_runtime::EarpReturn;
    use earp_runtime::EarpRuntime;
    use earp_runtime::EarpStatement;
    use futures::executor::block_on;
    use super::*;

    #[test]
    fn test_smoke() {
        let ign = console_intergation();
        let push_frame = EarpFunction::new(|_| {
            EarpReturn::Push    
        });
        let pop_frame = EarpFunction::new(|_| {
            EarpReturn::Pop    
        });
        let add =  EarpFunction::new(|args| {
            if let (Some(a),Some(b)) = (args[0].downcast_ref::<f64>(),args[1].downcast_ref::<f64>()) {
                EarpReturn::Value(Arc::new(Box::new(a+b)))
            } else {
                EarpReturn::None
            }
        });
        let clone = EarpFunction::new(|args| {
            EarpReturn::Value(args[0].clone().into_owned())
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
            } else {
                EarpReturn::None
            }
        });
        
        
        let univ_stmt = EarpStatement::new(&add,vec![
            EarpArgument::Literal(Arc::new(Box::new(40_f64))),
            EarpArgument::Literal(Arc::new(Box::new(2_f64)))
        ],EarpResultRegister::Var(1));
        let n2s_stmt = EarpStatement::new(&num_to_string,vec![
            EarpArgument::Var(1)
        ],EarpResultRegister::Var(0));
        let print_stmt = EarpStatement::new(&print,vec![EarpArgument::Var(0)],
        EarpResultRegister::None);
        let program = EarpProgram::new(vec![univ_stmt,n2s_stmt,print_stmt]);
        /**/
        let runtime = EarpRuntime::new(program);
        block_on(runtime.run());
    }
}