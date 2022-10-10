#![warn(missing_debug_implementations)]

mod util; // Import first because it has macros

mod data;
mod protocol;
#[cfg(test)]
mod tests;
mod client;
mod http_endpoints;