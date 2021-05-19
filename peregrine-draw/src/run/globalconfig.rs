use peregrine_data::{ DataMessage, PgdPeregrineConfig };
use super::config::PgPeregrineConfig;
use crate::util::Message;

pub(crate) struct CreatedPeregrineConfigs<'a> {
    pub data: PgdPeregrineConfig<'a>,
    pub(crate) draw: PgPeregrineConfig<'a>
}

pub struct PeregrineConfig<'a>(CreatedPeregrineConfigs<'a>);

impl<'a> PeregrineConfig<'a> {
    pub fn new() -> PeregrineConfig<'a> {
        let pg_config = PgPeregrineConfig::new();
        let pgd_config = PgdPeregrineConfig::new();
        PeregrineConfig(CreatedPeregrineConfigs {
            draw: pg_config,
            data: pgd_config
        })
    }

    pub fn set(&mut self, key_str: &str, value: &str) -> Result<(),Message> {
        self.0.data.set(key_str,value).map_err(|e| Message::DataError(e))?;
        self.0.draw.set(key_str,value)?;
        Ok(())
    }

    pub(crate) fn build(self) -> CreatedPeregrineConfigs<'a> { self.0 }
}
