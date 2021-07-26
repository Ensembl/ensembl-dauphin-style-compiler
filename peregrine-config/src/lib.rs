use std::collections::HashMap;
use std::hash::Hash;
use std::fmt::Debug;

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub enum ConfigError {
    UnknownConfigKey(String),
    UninitialisedKey(String),
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
    key_to_str: HashMap<K,String>,
    defaults: HashMap<K,&'a V>,
    values: HashMap<K,V>
}

impl<'a,K: Debug+Clone+PartialEq+Eq+Hash, V: ConfigValue+Clone> Config<'a,K,V> {
    pub fn new(info: &[ConfigKeyInfo<'a,K,V>]) -> Config<'a,K,V> {
        let mut str_to_key = HashMap::new();
        let mut key_to_str = HashMap::new();
        let mut defaults = HashMap::new();
        for info in info.iter() {
            str_to_key.insert(info.name.to_string(),info.key.clone());
            key_to_str.insert(info.key.clone(),info.name.to_string());
            defaults.insert(info.key.clone(),info.default);
        }
        Config {
            str_to_key,
            key_to_str,
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

    pub fn try_get(&self, key: &K) -> Option<&V> {
        if let Some(v) = self.values.get(key) { return Some(v); }
        if let Some(v) = self.defaults.get(key) { return Some(v); }
        None
    }

    pub fn get(&self, key: &K) -> Result<&V,ConfigError> {
        if let Some(v) = self.try_get(key) { return Ok(v); }
        Err(ConfigError::UninitialisedKey(format!("{:?}",key)))
    }
}
