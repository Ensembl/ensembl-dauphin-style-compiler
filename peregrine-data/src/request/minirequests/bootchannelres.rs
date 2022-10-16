use std::fmt;
use peregrine_toolkit::{serdetools::st_field};
use serde::{Deserialize, Deserializer, de::{Visitor, MapAccess, IgnoredAny}};
use crate::{Assets, BackendNamespace, request::core::response::MiniResponseVariety};

pub struct BootChannelRes {
    namespace: BackendNamespace,
    channel_assets: Assets,
    chrome_assets: Assets,
    supports: Option<Vec<u32>>
}

impl BootChannelRes {
    pub fn new(namespace: BackendNamespace, channel_assets: Assets, chrome_assets: Assets, supports: Option<Vec<u32>>) -> BootChannelRes {
        BootChannelRes { namespace, channel_assets, chrome_assets, supports }
    }

    pub(crate) fn channel_assets(&self) -> &Assets { &self.channel_assets }
    pub(crate) fn chrome_assets(&self) -> &Assets { &self.chrome_assets }
    pub(crate) fn namespace(&self) -> &BackendNamespace { &self.namespace }
    pub(crate) fn supports(&self) -> Option<&[u32]> { self.supports.as_ref().map(|x| &x[..]).clone() }
}

struct BootChannelVisitor;

impl<'de> Visitor<'de> for BootChannelVisitor {
    type Value = BootChannelRes;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a StickRes")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where M: MapAccess<'de> {
        let mut supports = None;
        let mut namespace : Option<BackendNamespace> = None;
        let mut assets = None;
        let mut chrome_assets = None;
        while let Some(key) = access.next_key()? {
            match key {
                "supports" => { supports = Some(access.next_value()?); },
                "namespace" => { namespace = Some(access.next_value()?); },
                "chrome-assets" => { chrome_assets = Some(access.next_value()?); },
                "assets" => { assets = Some(access.next_value()?); },
                _ => { let _ : IgnoredAny = access.next_value()?; }
            }
        }
        let namespace = st_field("namespace",namespace)?;
        let assets_loader = st_field("assets",assets)?;
        let chrome_assets_loader = st_field("chrome-assets",chrome_assets)?;
        let mut chrome_assets = Assets::empty();
        chrome_assets.load(chrome_assets_loader,Some(namespace.clone()));
        let mut assets = Assets::empty();
        assets.load(assets_loader,Some(namespace.clone()));
        Ok(BootChannelRes {
            namespace,
            channel_assets: assets,
            chrome_assets,
            supports
        })
    }
}

impl<'de> Deserialize<'de> for BootChannelRes {
    fn deserialize<D>(deserializer: D) -> Result<BootChannelRes, D::Error>
            where D: Deserializer<'de> {
        deserializer.deserialize_map(BootChannelVisitor)
    }
}

impl MiniResponseVariety for BootChannelRes {
    fn description(&self) -> &str { "bootstrap" }
}
