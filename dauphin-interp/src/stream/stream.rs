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

pub trait StreamConnector {
    fn notice(&self, msg: &str) -> anyhow::Result<()>;
    fn warn(&self, msg: &str) -> anyhow::Result<()>;
    fn error(&self, msg: &str) -> anyhow::Result<()>;
}

pub struct ConsoleStreamConnector();

impl ConsoleStreamConnector {
    pub fn new() -> ConsoleStreamConnector {
        ConsoleStreamConnector()
    }
}

impl StreamConnector for ConsoleStreamConnector {
    fn notice(&self, msg: &str) -> anyhow::Result<()> {
        print!("{}\n",msg);
        Ok(())
    }

    fn warn(&self, msg: &str) -> anyhow::Result<()> {
        print!("WARNING! {}\n",msg);
        Ok(())
    }

    fn error(&self, msg: &str) -> anyhow::Result<()> {
        print!("ERROR! {}\n",msg);
        Ok(())
    }
}

pub struct Stream {
    connector: Box<dyn StreamConnector>,
    contents: HashMap<u8,Vec<String>>,
    send: bool,
    keep: bool
}

impl Stream {
    pub fn new(connector: Box<dyn StreamConnector>, keep: bool, send: bool) -> Stream {
        Stream {
            connector,
            contents: HashMap::new(),
            keep, send
        }
    }

    fn entries(&mut self, level: u8) -> &mut Vec<String> {
        self.contents.entry(level).or_insert_with(|| vec![])
    }

    pub fn set_send(&mut self, yn: bool) {
        self.send = yn;
    }

    pub fn set_keep(&mut self, yn: bool) {
        self.keep = yn;
    }

    pub fn take(&mut self, level: u8) -> Vec<String> {
        replace(self.entries(level),vec![])
    }

    pub fn add(&mut self, level: u8, more: &str) {
        if self.keep {
            self.entries(level).push(more.to_string());
        }
        if self.send {
            if level == 0 {
                self.connector.notice(more);
            } else if level == 1 {
                self.connector.warn(more);
            } else {
                self.connector.error(more);
            }
        }
    }
}

impl Payload for Stream {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn finish(&mut self) {}
}

pub struct ConsoleStreamFactory {
    to_stdout: bool
}

impl ConsoleStreamFactory {
    pub fn new() -> ConsoleStreamFactory {
        ConsoleStreamFactory{
            to_stdout: false
        }
    }

    pub fn to_stdout(&mut self, yn: bool) {
        self.to_stdout = yn;
    }
}

impl PayloadFactory for ConsoleStreamFactory {
    fn make_payload(&self) -> Box<dyn Payload> {
        Box::new(Stream::new(Box::new(ConsoleStreamConnector::new()),true,self.to_stdout))
    }
}