//! # opensubsonic
//! This crate provides types to work with the [OpenSubsonic API](https://opensubsonic.netlify.app/).
//! Right now it is mainly focused on implementing server-side functionality.
//!
//! ## Server
//!
//! A server can be created by implementing the [`service::OpenSubsonicServer`] trait.
//! Then a [`tower::Service`] can be created from it using [`service::OpenSubsonicService::new`].
//! An example can be found in the [`service`] module.
pub mod common;
pub mod query;
pub mod request;
pub mod response;
pub mod service;

pub use async_trait::async_trait;
