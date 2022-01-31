pub trait EarpValue {
    fn coerce_string(&self) -> Option<String> { None }
    fn type_name(&self) -> String { "*unnamed-type*".to_string() }
}

impl EarpValue for () {
    
}

impl EarpValue for String {
    fn coerce_string(&self) -> Option<String> { Some(self.clone()) }
    fn type_name(&self) -> String { "string".to_string() }
}

impl EarpValue for bool {
    fn coerce_string(&self) -> Option<String> { Some(self.to_string()) }
    fn type_name(&self) -> String { "boolean".to_string() }
}

impl EarpValue for i64 {
    fn coerce_string(&self) -> Option<String> { Some(self.to_string()) }    
    fn type_name(&self) -> String { "integer".to_string() }
}

impl EarpValue for f64 {
    fn coerce_string(&self) -> Option<String> { Some(self.to_string()) }   
    fn type_name(&self) -> String { "float".to_string() } 
}
