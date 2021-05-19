use std::collections::HashMap;
use std::hash::Hash;

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub enum ConfigError {
    UnknownConfigKey(String),
    BadConfigValue(String,String),
}

pub trait ConfigValue : Sized {
    fn parse(&self, value_str: &str) -> Result<Self,String>;
}

pub struct ConfigKeyInfo<'a,K,V> {
    pub key: K,
    pub name: &'a str,
    pub default: &'a V
}

pub struct Config<'a,K,V> where K: PartialEq+Eq+Hash, V: ConfigValue + Clone {
    str_to_key: HashMap<String,K>,
    defaults: HashMap<K,&'a V>,
    values: HashMap<K,V>
}

impl<'a,K: Clone+PartialEq+Eq+Hash, V: ConfigValue+Clone> Config<'a,K,V> {
    pub fn new(info: &[ConfigKeyInfo<'a,K,V>]) -> Config<'a,K,V> {
        let mut str_to_key = HashMap::new();
        let mut defaults = HashMap::new();
        for info in info.iter() {
            str_to_key.insert(info.name.to_string(),info.key.clone());
            defaults.insert(info.key.clone(),info.default);
        }
        Config {
            str_to_key,
            defaults,
            values: HashMap::new()
        }
    }

    pub fn set(&mut self, key_str: &str, value_str: &str) -> Result<(),ConfigError> {
        if let Some(key) = self.str_to_key.get(key_str){
            let value = self.defaults.get(key).unwrap().parse(value_str).map_err(|e| {
                ConfigError::BadConfigValue(key_str.to_string(),e)
            })?;
            self.values.insert(key.clone(),value);
        }
        Ok(())
    }

    pub fn get(&self, key: &K) -> &V {
        self.values.get(key).unwrap_or_else(|| {
            self.defaults.get(key).unwrap()
        })
    }
}
