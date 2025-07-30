//! Common Modul for the laptimer
//!
//! Provides the common data types that are used across every modul.

pub mod lap;
pub mod position;
mod serde;
pub mod session;
pub mod test_helper;
pub mod track;

#[cfg(test)]
mod tests;
