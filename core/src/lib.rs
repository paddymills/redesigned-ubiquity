
#![feature(cfg_version)]
#![cfg_attr(not(version("1.80")), feature(lazy_cell))]

#![warn(missing_docs)]

//! High Steel system integration components

pub mod api;
pub mod config;
