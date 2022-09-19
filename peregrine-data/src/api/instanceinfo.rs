use std::fmt::{self, Display, Formatter};

use crate::{request::messages::bootstrapres::BootRes, core::version::VersionMetadata, Channel};

pub struct InstanceInformation {
    pub channel: Channel,
    pub frontend_version: u32,
    pub backend_version: Vec<u32>
}

impl InstanceInformation {
    pub(crate) fn new(channel: &Channel, boot_res: &BootRes, frontend_version: &VersionMetadata) -> InstanceInformation {
        InstanceInformation { 
            channel: channel.clone(),
            frontend_version: frontend_version.backend_version(),
            backend_version: boot_res.supports().map(|x| x.to_vec()).unwrap_or(vec![0])
        }
    }
}

impl Display for InstanceInformation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f,"{} ->  frontend: api-version={}. backend: supports-api-versions={}",
            self.channel,
            self.frontend_version,
            self.backend_version.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")
        )
    }
}
