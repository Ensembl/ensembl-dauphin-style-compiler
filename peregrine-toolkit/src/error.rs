use crate::{error, log};

#[derive(Clone,Debug)]
pub enum CallToAction {
    BadVersion
}

#[derive(Clone,Debug)]
pub enum ErrorType {
    FatalError,      // Browser should be considered crashed
    OperationError,  // Operation did not complete but other operations may succeed
    Unavailable(CallToAction),     // Browser out-of-date etc; enum-based call to action supplied
    NoSuch,          // A parameter passed contained reference to missing data
    Temporary,       // FYI, will retry
}

#[derive(Clone,Debug)]
pub struct Error {
    pub error_type: ErrorType,
    pub message: String
}

macro_rules! error_ctor {
    ($name:ident,$rname:ident,$type:expr) => {
        pub fn $name(text: &str) -> Error {
            crate::error::Error {
                error_type: $type,
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
    error_ctor!(fatal,fatal_r,ErrorType::FatalError);
    error_ctor!(operr,oper_r,ErrorType::OperationError);
    error_ctor!(nosuch,nosuch_r,ErrorType::NoSuch);
    error_ctor!(tmp,tmp_r,ErrorType::Temporary);
    error_ctor!(bad_version,bad_version_r,ErrorType::Unavailable(CallToAction::BadVersion));

    pub fn web_deadend(&self) {
        match self.error_type {
            ErrorType::FatalError => { panic!("{}",self.message); },
            ErrorType::Temporary => { log!("{}",self.message); },
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
