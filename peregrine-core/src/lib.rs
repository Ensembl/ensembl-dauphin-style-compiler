mod core {
    pub mod focus;
    pub mod stick;
    pub mod track;
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
    pub use self::channel::{ Channel, ChannelIntegration, ChannelLocation, PacketPriority };
    pub use self::program::ProgramLoader;
    pub use self::manager::RequestManager;
}

mod run {
    pub mod console;
    mod core;
    pub mod pgcommander;
    pub mod pgdauphin;
    pub use pgdauphin::PgDauphinIntegration;
    pub use self::core::PgCore;
    pub use self::console::PgConsole;
    pub use self::pgcommander::Commander;
    pub use self::pgcommander::{ PgCommander, PgCommanderTaskSpec };
    pub use self::pgdauphin::{ PgDauphin, PgDauphinTaskSpec };
}

mod util {
    pub mod cbor;
    pub mod singlefile;
}

#[cfg(test)]
mod test {
    pub(crate) mod integrations {
        mod channel;
        mod commander;
        mod console;
        mod dauphin;
        pub(crate) use self::console::TestConsole;
        pub(crate) use self::channel::{ TestChannelIntegration, cbor_matches };
        pub(crate) use self::commander::{ TestCommander };
        pub(crate) use self::dauphin::{ TestDauphinIntegration, test_program };
    }
    pub(crate) mod helpers;
}

pub use self::run::{ PgCommander, PgCommanderTaskSpec, PgConsole, PgDauphin, Commander, PgDauphinIntegration };
pub use self::request::{ Channel, ChannelIntegration, ChannelLocation, PacketPriority, ProgramLoader, RequestManager };
pub use self::run::PgCore;
