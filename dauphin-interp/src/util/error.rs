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
    RuntimeError(String),
    SourceError(String,String,u32),
    FloatingSourceError(String)
}

impl DauphinError {
    pub fn floating(msg: &str) -> anyhow::Error {
        anyhow::Error::new(DauphinError::FloatingSourceError(msg.to_string()))
    }
}

pub fn error_locate<T>(result: anyhow::Result<T>, filename: &str, line: u32) -> anyhow::Result<T> {
    match result {
        Ok(r) => Ok(r),
        Err(e) => {
            Err(match e.downcast::<DauphinError>() {
                Ok(de) => {
                    anyhow::Error::new(match de {
                        /* Note we deliberately blast context here are source errors are soft errors whose context is the filename/line */
                        DauphinError::FloatingSourceError(msg) => DauphinError::SourceError(msg.to_string(),filename.to_string(),line),
                        err => err
                    })
                },
                Err(e) => e
            })
        }
    }
}

pub fn error_runtime<T>(result: anyhow::Result<T>) -> anyhow::Result<T> {
    match result {
        Ok(r) => Ok(r),
        Err(e) => {
            Err(match e.downcast::<DauphinError>() {
                Ok(de) => {
                    anyhow::Error::new(match de {
                        /* Note we deliberately blast context here are source errors are soft errors whose context is the filename/line */
                        DauphinError::FloatingSourceError(msg) => DauphinError::RuntimeError(msg),
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
        match self {
            DauphinError::OSError(err) => write!(f,"Underlying OS Error: {}\n",err)?,
            DauphinError::LogicError(filename,linenum) => write!(f,"Unexpected internal error in {} at line {}\n",filename,linenum)?,
            DauphinError::RuntimeError(msg) => write!(f,"Runtime error: {}\n",msg)?,
            DauphinError::SourceError(msg,filename,linenum) => write!(f,"{} in {} at {}",msg,filename,linenum)?,
            DauphinError::FloatingSourceError(msg) => write!(f,"{}",msg)?
        }
        Ok(())
    }
}

pub fn xxx_error<T,E>(a: anyhow::Result<T,E>) -> Result<T,String> where E: Debug {
    a.map_err(|x| format!("{:?}",x))
}