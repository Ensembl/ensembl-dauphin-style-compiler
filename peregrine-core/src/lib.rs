mod core {
    pub mod focus;
    pub mod stick;
    pub mod track;
    pub use stick::{ StickId, Stick };
}

mod index {
    pub(crate) mod stickstore;
    pub(crate) mod stickauthority;
    pub(crate) mod stickauthoritystore;
    pub use self::stickstore::StickStore;
    pub use self::stickauthoritystore::StickAuthorityStore;
}

mod panel {
    mod panel;
    mod scale;
}

mod request {
    pub(crate) mod backoff;
    pub(crate) mod bootstrap;
    pub(crate) mod channel;
    pub(crate) mod failure;
    pub(crate) mod manager;
    pub(crate) mod packet;
    pub(crate) mod queue;
    pub(crate) mod program;
    pub(crate) mod request;
    pub(crate) mod stick;
    pub(crate) mod stickauthority;
    pub use self::channel::{ Channel, ChannelIntegration, ChannelLocation, PacketPriority };
    pub use self::program::{ ProgramLoader };
    pub use self::manager::RequestManager;
}

mod run {
    pub mod console;
    mod core;
    pub mod instancepayload;
    pub mod pgcommander;
    pub mod pgdauphin;
    pub use self::core::PgCore;
    pub use self::console::PgConsole;
    pub use self::pgcommander::Commander;
    pub use self::pgcommander::{ PgCommander, PgCommanderTaskSpec };
    pub use self::pgdauphin::{ PgDauphin, PgDauphinTaskSpec };
    pub use self::instancepayload::InstancePayload;
}

mod util {
    pub mod cbor;
    pub mod singlefile;
    pub mod unlock;
}

#[cfg(test)]
mod test {
    pub(crate) mod integrations {
        mod channel;
        mod commander;
        mod console;
        mod dauphin;
        pub(crate) use self::console::TestConsole;
        pub(crate) use self::channel::{ TestChannelIntegration, cbor_matches, cbor_matches_print };
        pub(crate) use self::commander::{ TestCommander };
        pub(crate) use self::dauphin::FakeDauphinReceiver;
    }
    pub(crate) mod helpers;
}

pub use self::core::{ Stick, StickId };
pub use self::index::{ StickStore, StickAuthorityStore };
pub use self::run::{ PgCommander, PgCommanderTaskSpec, PgConsole, PgDauphin, Commander, InstancePayload };
pub use self::request::{ Channel, ChannelIntegration, ChannelLocation, PacketPriority, ProgramLoader, RequestManager };
pub use self::run::PgCore;
