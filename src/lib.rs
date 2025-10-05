#![allow(dead_code)]
#![feature(never_type)]

pub mod agent;
pub mod battle_of_lits;
pub mod ltp_server;

pub mod utils {
    pub mod prelude {
        pub use anyhow::{anyhow, Context, Error};
        pub type Result<T> = anyhow::Result<T, Error>;
        pub use fastset::Set as FastSet;
        pub use primitive_types::U256;

        pub use std::{
            collections::{BTreeSet, HashSet, HashMap},
            ops::{Add, Sub}
        };
    }
}

pub mod prelude {
    pub use super::agent::*;
    pub use super::battle_of_lits::prelude::*;
    pub use super::ltp_server::*;
    pub use super::utils::prelude::*;
}
