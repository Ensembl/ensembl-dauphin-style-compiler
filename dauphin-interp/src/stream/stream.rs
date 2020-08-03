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

use std::any::Any;
use std::mem::replace;
use std::collections::HashMap;
use crate::runtime::{ Payload, PayloadFactory };

pub struct Stream {
    contents: HashMap<u8,Vec<String>>,
    to_stdout: bool
}

impl Stream {
    pub fn new(to_stdout: bool) -> Stream {
        Stream {
            contents: HashMap::new(),
            to_stdout
        }
    }

    fn entries(&mut self, level: u8) -> &mut Vec<String> {
        self.contents.entry(level).or_insert_with(|| vec![])
    }

    pub fn to_stdout(&mut self, yn: bool) {
        self.to_stdout = yn;
    } 

    pub fn add(&mut self, level: u8, more: &str) {
        self.entries(level).push(more.to_string());
        if self.to_stdout {
            let lev_str = ["","WARNING: ","ERROR: "].get(level as usize).unwrap_or(&"");
            print!("{}{}\n",lev_str,more);
        }
    }

    pub fn take(&mut self, level: u8) -> Vec<String> {
        replace(self.entries(level),vec![])
    }
}

impl Payload for Stream {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn finish(&mut self) {}
}

pub struct StreamFactory {
    to_stdout: bool
}

impl StreamFactory {
    pub fn new() -> StreamFactory {
        StreamFactory{
            to_stdout: false
        }
    }

    pub fn to_stdout(&mut self, yn: bool) {
        self.to_stdout = yn;
    }
}

impl PayloadFactory for StreamFactory {
    fn make_payload(&self) -> Box<dyn Payload> {
        Box::new(Stream::new(self.to_stdout))
    }
}