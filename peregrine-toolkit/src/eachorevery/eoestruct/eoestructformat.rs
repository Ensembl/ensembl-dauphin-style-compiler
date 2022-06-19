use super::{eoestruct::{VariableSystem, Struct, StructConst, StructResult, StructError}};

#[cfg(debug_assertions)]
pub trait VariableSystemFormatter<T: VariableSystem> {
    fn format_declare_start(&mut self, var: &[T::Declare]) -> String;
    fn format_declare_end(&mut self, var: &[T::Declare]) -> String;
    fn format_use(&mut self, var: &T::Use) -> Result<String,StructError>;
}

#[cfg(debug_assertions)]
impl<T: VariableSystem+Clone> std::fmt::Debug for Struct<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",self.format().unwrap_or_else(|x| format!("*error-formatting-struct*({})",x)))
    }
}

// XXX test serial at DataVisitor
#[cfg(debug_assertions)]
fn comma_separate<'a,F,X,Y>(it: X, mut cb: F, output: &mut String) -> StructResult
        where X: Iterator<Item=Y>,
              F: FnMut(Y,&mut String) -> StructResult {
    let mut first = true;
    for item in it {
        if !first { output.push_str(","); }
        cb(item,output)?;
        first = false;
    }
    Ok(())
}

impl<T: VariableSystem+Clone> Struct<T> {
    pub(super) fn format(&self) -> Result<String,StructError> {
        let mut output = String::new();
        let mut formatter = T::build_formatter();
        self.format_level(&mut formatter,&mut output)?;
        Ok(output)
    }

    fn format_level(&self, formatter: &mut Box<dyn VariableSystemFormatter<T>>, output: &mut String) -> StructResult {
        match self {
            Struct::Var(var) => {
                output.push_str(&formatter.format_use(var)?);
            },
            Struct::Const(val) => {
                output.push_str(&match val {
                    StructConst::Number(value) => format!("{:?}",value),
                    StructConst::String(value) => format!("{:?}",value),
                    StructConst::Boolean(value) => format!("{:?}",value),
                    StructConst::Null => format!("null")
                });
            },
            Struct::Array(values) => {
                output.push_str("[");
                comma_separate(values.iter(),|item,output| {
                    item.format_level(formatter,output)
                },output)?;
                output.push_str("]");
            },
            Struct::Object(object) => {
                output.push_str("{");
                comma_separate(object.iter(),|item,output| {
                    output.push_str(&format!("{:?}: ",item.0));
                    item.1.format_level(formatter,output)
                }, output)?;
                output.push_str("}");
            },
            Struct::All(ids, expr) => {
                output.push_str(&formatter.format_declare_start(ids));
                expr.format_level(formatter,output)?;                
                output.push_str(&formatter.format_declare_end(ids));
            },
        }
        Ok(())
    }
}
