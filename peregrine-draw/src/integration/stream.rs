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

use dauphin_interp::runtime::{ Payload, PayloadFactory };
use dauphin_interp::{ StreamConnector, Stream };
use peregrine_toolkit::console::Severity;
use peregrine_toolkit::{error, warn, log};

pub struct WebStreamConnector();

impl WebStreamConnector {
    pub fn new() -> WebStreamConnector {
        WebStreamConnector()
    }
}

impl StreamConnector for WebStreamConnector {
    fn log(&self, severity: &Severity, msg: &str) -> anyhow::Result<()> {
        match severity {
            Severity::Error => { error!("ERROR! {}",msg); },
            Severity::Warning => { warn!("WARNING! {}",msg); },
            _ => { log!("{}",msg); }
        };
        Ok(())
    }
}

pub struct WebStreamFactory {
}

impl WebStreamFactory {
    pub fn new() -> WebStreamFactory {
        WebStreamFactory{
        }
    }
}

impl PayloadFactory for WebStreamFactory {
    fn make_payload(&self) -> Box<dyn Payload> {
        Box::new(Stream::new(Box::new(WebStreamConnector::new()),false,true))
    }
}