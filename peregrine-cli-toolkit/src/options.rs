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

use clap::{ App, Arg };

pub struct ConfigOption<C,E> {
    name: String,
    long: String,
    short: Option<String>,
    value: Option<String>,
    multiple: bool,
    cb: Box<dyn Fn(&mut C,&str) -> Result<(),E>>
}

impl<C,E> ConfigOption<C,E> {
    pub fn new<T>(name: &str, long: &str, short: Option<&str>, value: Option<&str>, multiple: bool, cb: T) -> ConfigOption<C,E>
                                                            where T: Fn(&mut C,&str) -> Result<(),E> + 'static {
        ConfigOption {
            name: name.to_string(),
            long: long.to_string(),
            short: short.map(|x| x.to_string()),
            value: value.map(|x| x.to_string()),
            multiple,
            cb: Box::new(cb)
        }
    }
    
    fn to_arg<'a>(&'a self) -> Arg<'a> {
        let mut arg = Arg::new(self.name.as_str()).long(&self.long);
        if let Some(ref short) = self.short { 
            if let Some(first) = short.chars().next() {
                arg = arg.short(first);
            }
        }
        if let Some(ref value) = self.value { arg = arg.takes_value(true).value_name(value); }
        if self.multiple { arg = arg.multiple_occurrences(true); }
        arg
    }
}

pub fn config_from_options<C,E>(config: &mut C, options: Vec<ConfigOption<C,E>>, 
                                    name: &str, version: &str, author: &str, about: &str) ->Result<(),E> {
    let mut args = App::new(name).version(version).author(author).about(about);
    for option in &options {
        args = args.arg(option.to_arg());
    }
    let matches = args.get_matches();
    for option in &options {
        if option.value.is_some() {
            if let Some(values) = matches.values_of(&option.name) {
                for value in values {
                    (option.cb)(config,&value)?;
                }
            }
        } else {
            if matches.is_present(&option.name) {
                (option.cb)(config,&format!("{}",matches.occurrences_of(&option.name)))?;
            }
        }
    }
    Ok(())
}
