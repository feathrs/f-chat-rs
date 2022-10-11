#![warn(missing_debug_implementations)]

mod util; // Import first because it has macros

pub mod data;
pub mod protocol;
pub mod client;
pub mod http_endpoints;

#[cfg(test)]
mod tests;