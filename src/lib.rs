#![cfg_attr(feature = "cargo-clippy", allow(deref_addrof))]

extern crate failure;
extern crate failure_derive;
#[macro_use]
extern crate ndarray;
#[macro_use]
extern crate serde_derive;
extern crate sdl2;
extern crate toml;
#[macro_use]
extern crate enum_iterator;
extern crate noise;
extern crate rand;

pub mod map;
pub mod renderer;
