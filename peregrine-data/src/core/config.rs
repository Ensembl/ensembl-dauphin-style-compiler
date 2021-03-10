use std::collections::HashMap;

pub enum ConfigValue {
    Float(f64)
}

impl ConfigValue {
    fn get_f64(&self) -> Option<f64> {
        match self {
            ConfigValue::Float(x) => Some(*x)
        }
    }
}

pub struct PeregrineConfig {
    values: HashMap<String,ConfigValue>
}

impl PeregrineConfig {
    pub fn new() -> PeregrineConfig {
        PeregrineConfig {
            values: HashMap::new()
        }
    }


    pub fn set(&mut self, key: &str, value: ConfigValue) {
        self.values.insert(key.to_string(),value);
    }

    pub fn set_f64(&mut self, key: &str, value: f64) {
        self.values.insert(key.to_string(),ConfigValue::Float(value));
    }

    pub fn get(&self, key: &str) -> Option<&ConfigValue> {
        self.values.get(key)
    }

    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.values.get(key).and_then(|x| x.get_f64())
    }
}
