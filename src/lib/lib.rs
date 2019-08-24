//!Quick and lean social media library

#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]
#![feature(async_closure)]

pub mod data;
pub mod config;
pub mod api;

pub use api::API;
