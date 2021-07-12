/* 
 *  See the NOTICE file distributed with this work for additional information
 *  regarding copyright ownership.
 *  
 *  Licensed under the Apache License, Version 2.0 (the "License"); you may 
 *  not use this file except in compliance with the License. You may obtain a
 *  copy of the License at http://www.apache.org/licenses/LICENSE-2.0
 *  
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS, WITHOUT 
 *  WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 */

use anyhow;
use std::fmt::{ self, Display, Formatter, Debug };
use thiserror::Error;

#[derive(Debug,Error)]
pub enum DauphinError {
    OSError(std::io::Error),
    LogicError(String,u32),
    MalformedError(String),
    ConfigError(String),
    IntegrationError(String),
    FloatingRuntimeError(String),
    RuntimeError(String,String,u32,usize),
    SourceError(String,String,u32,usize),
    FloatingSourceError(String)
}

impl DauphinError {
    pub fn source(msg: &str) -> anyhow::Error {
        anyhow::Error::new(DauphinError::FloatingSourceError(msg.to_string()))
    }

    pub fn runtime(msg: &str) -> anyhow::Error {
        anyhow::Error::new(DauphinError::FloatingRuntimeError(msg.to_string()))
    }

    pub fn internal(file: &str, line: u32) -> anyhow::Error {
        anyhow::Error::new(DauphinError::LogicError(file.to_string(),line))
    }

    pub fn config(msg: &str) -> anyhow::Error {
        anyhow::Error::new(DauphinError::ConfigError(msg.to_string()))
    }

    pub fn integration(msg: &str) -> anyhow::Error {
        anyhow::Error::new(DauphinError::IntegrationError(msg.to_string()))
    }

    pub fn malformed(msg: &str) -> anyhow::Error {
        anyhow::Error::new(DauphinError::MalformedError(msg.to_string()))
    }
}

pub fn error_locate_cb<F,T>(cb: F, error: anyhow::Result<T>) -> anyhow::Result<T> where F: FnOnce() -> (String,u32,usize) {
    match error {
        Ok(t) => Ok(t),
        Err(e) => {
            let (filename,line,instr) = cb();
            if line != 0 {
                Err(error_locate(e,&filename,line,instr))
            } else {
                Err(e)
            }
        }
    }
}

pub fn error_locate(error: anyhow::Error, filename: &str, line: u32,instr: usize) -> anyhow::Error {
    match error.downcast::<DauphinError>() {
        Ok(de) => {
            anyhow::Error::new(match de {
                /* Note we deliberately blast context here are source errors are soft errors whose context is the filename/line */
                DauphinError::FloatingSourceError(msg) => DauphinError::SourceError(msg.to_string(),filename.to_string(),line,instr),
                DauphinError::FloatingRuntimeError(msg) => DauphinError::RuntimeError(msg.to_string(),filename.to_string(),line,instr),
                err => err
            })
        },
        Err(e) => e
    }
}

pub fn extend_error(error: anyhow::Error, more: &str) -> anyhow::Error {
    match error.downcast::<DauphinError>() {
        Ok(de) => {
            anyhow::Error::new(match de {
                DauphinError::SourceError(msg,file,line,instr) => DauphinError::SourceError(format!("{} ({})",msg,more),file,line,instr),
                x => x
            })
        },
        Err(e) => e
    }
}

pub fn result_locate<T>(result: anyhow::Result<T>, filename: &str, line: u32, instr: usize) -> anyhow::Result<T> {
    match result {
        Ok(r) => Ok(r),
        Err(e) => Err(error_locate(e,filename,line,instr))
    }
}

pub fn result_runtime<T>(result: anyhow::Result<T>) -> anyhow::Result<T> {
    match result {
        Ok(r) => Ok(r),
        Err(e) => {
            Err(match e.downcast::<DauphinError>() {
                Ok(de) => {
                    anyhow::Error::new(match de {
                        /* Note we deliberately blast context here are source errors are soft errors whose context is the filename/line */
                        DauphinError::FloatingSourceError(msg) => DauphinError::FloatingRuntimeError(msg),
                        DauphinError::SourceError(msg,filename,line,instr) => DauphinError::RuntimeError(msg.to_string(),filename.to_string(),line,instr),
                        err => err
                    })
                },
                Err(e) => e
            })
        }
    }
}

impl Display for DauphinError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut instr = None;
        match self {
            DauphinError::OSError(err) => write!(f,"Underlying OS Error: {}\n",err)?,
            DauphinError::LogicError(filename,linenum) => write!(f,"Unexpected internal error in {} at line {}\n",filename,linenum)?,
            DauphinError::IntegrationError(msg) => write!(f,"Intergation error: {}\n",msg)?,
            DauphinError::MalformedError(msg) => write!(f,"Malformed data: {}\n",msg)?,
            DauphinError::FloatingRuntimeError(msg) => write!(f,"Runtime error: {}\n",msg)?,
            DauphinError::ConfigError(msg) => write!(f,"Config/Command-line error: {}\n",msg)?,
            DauphinError::RuntimeError(msg,filename,linenum,instr_idx) => 
            {
                write!(f,"{}:{} Runtime error: {}",filename,linenum,msg)?;
                instr = Some(instr_idx);
            },
            DauphinError::SourceError(msg,filename,linenum,instr_idx) => {
                write!(f,"{} in {} at {}",msg,filename,linenum)?;
                instr = Some(instr_idx);
            },
            DauphinError::FloatingSourceError(msg) => write!(f,"{}",msg)?
        }
        if let Some(instr) = instr {
            write!(f," (instr. {})",instr)?;
        }
        Ok(())
    }
}

pub fn triage_source_errors(errors: &mut Vec<anyhow::Error>) -> anyhow::Result<Vec<String>> {
    let mut source_errors = vec![];
    for e in errors.drain(..) {
        match e.downcast_ref::<DauphinError>() {
            Some(DauphinError::SourceError(msg,filename,line,instr)) => {
                source_errors.push(format!("{}:{}({}) {}",filename,line,instr,msg));
            },
            Some(DauphinError::FloatingSourceError(msg)) => {
                source_errors.push(format!("--:-- {}",msg));
            },
            _ => { return Err(e); },
        }
    }
    Ok(source_errors)
}
