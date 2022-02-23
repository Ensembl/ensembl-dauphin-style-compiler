use std::sync::Arc;
use peregrine_data::PgdPeregrineConfig;
use peregrine_toolkit::console::{set_verbosity, Verbosity};
use super::{config::PgPeregrineConfig, PgConfigKey};
use crate::util::Message;

pub(crate) struct CreatingPeregrineConfigs<'a> {
    pub data: PgdPeregrineConfig<'a>,
    pub(crate) draw: PgPeregrineConfig
}

#[derive(Clone)]
pub(crate) struct CreatedPeregrineConfigs<'a> {
    pub data: Arc<PgdPeregrineConfig<'a>>,
    pub(crate) draw: Arc<PgPeregrineConfig>
}

pub struct PeregrineConfig<'a>(CreatingPeregrineConfigs<'a>);

impl<'a> PeregrineConfig<'a> {
    pub fn new() -> Result<PeregrineConfig<'a>,Message> {
        let pg_config = PgPeregrineConfig::new();
        let pgd_config = PgdPeregrineConfig::new();
        set_verbosity(&Verbosity::from_string(pg_config.get_str(&PgConfigKey::Verbosity)?));
        Ok(PeregrineConfig(CreatingPeregrineConfigs {
            draw: pg_config,
            data: pgd_config
        }))
    }

    pub fn set(&mut self, key_str: &str, value: &str) -> Result<(),Message> {
        self.0.data.set(key_str,value).map_err(|e| Message::DataError(e))?;
        self.0.draw.set(key_str,value)?;
        Ok(())
    }

    pub(crate) fn build(self) -> CreatedPeregrineConfigs<'a> {
        CreatedPeregrineConfigs {
            data: Arc::new(self.0.data),
            draw: Arc::new(self.0.draw)
        }
    }
}
