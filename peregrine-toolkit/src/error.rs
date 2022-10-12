use crate::error;

#[derive(Clone,Debug)]
pub enum ErrorType {
    FatalError,      // Browser should be considered crashed
    OperationError,  // Operation did not complete but other operations may succeed
    NoSuch           // A parameter passed contained reference to missing data
}

#[derive(Clone,Debug)]
pub struct Error {
    pub error_type: ErrorType,
    pub message: String
}

macro_rules! error_ctor {
    ($name:ident,$rname:ident,$type:tt) => {
        pub fn $name(text: &str) -> Error {
            crate::error::Error {
                error_type: ErrorType::$type,
                message: text.to_string()
            }
        }

        pub fn $rname<T, E: std::fmt::Debug>(data: Result<T,E>, text: &str) -> Result<T,Error> {
            data.map_err(|e| {
                crate::error::Error::$name(&format!("{}: {:?}",text,e))
            })
        }
    }
}

impl Error {
    error_ctor!(fatal,fatal_r,FatalError);
    error_ctor!(operr,oper_r,OperationError);
    error_ctor!(nosuch,nosuch_r,NoSuch);

    pub fn web_deadend(&self) {
        match self.error_type {
            ErrorType::FatalError => { panic!("{}",self.message); },
            _ => { error!("{}",self.message); },
        }
    }
}

pub fn err_web_drop(value: Result<(),Error>) {
    if let Err(e) = value {
        e.web_deadend();
    }
}

#[macro_export]
macro_rules! pg_ok {
    ($data:expr) => {
        $data.map_err(|e| {
            use $crate::error::{ Error };
            Error::fatal(&format!("{:?} ({}:{})",e,file!(),line!())) 
        })            
    };
}

#[macro_export]
macro_rules! pg_unwrap {
    ($data:expr) => {
        $data.ok_or_else(|| {
            use $crate::error::{ Error }; 
            Error::fatal(&format!("unwrap failed ({}:{})",file!(),line!()))
        })
    };
}
