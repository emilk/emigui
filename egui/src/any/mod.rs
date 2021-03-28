//! Any type storages for [`crate::Memory`].
//!
//! This module contains structs to store arbitrary type using [`std::any::Any`] trait. Also, they can be cloned, and structs in [`serializable`] can be de/serialized.
//!
//! Basically, all this is just `HashMap<TypeId, Box<dyn Any + static>>` and `HashMap<Id, Box<dyn Any + static>>`, but with helper functions and hacks for cloning and de/serialization.
//!
//! # Trait requirements
//!
//! If you want to store your type here, it must implement `Clone` and `Any` and be `'static`, which means it must not contain references. If you want to store your data in serializable storage, it must implement `serde::Serialize` and `serde::Deserialize` under `persistent` feature.
//!
//! # `AnyMap`
//!
//! It stores everything by just type. You should use this map for your widget, when all instances of your widgets can have only one state. E.g. for popup windows, for color picker.
//!
//! In order to not have intersections, you should create newtype for anything you try to store here, like:
//! ```rust
//! struct MyEditBool(pub bool);
//! ```
//!
//! # `AnyMapId`
//!
//! [`AnyMap`] and [`AnyMapId`] has quite similar interface, except for [`AnyMapId`] you should pass [`crate::Id`] to get and insert things.
//!
//! It stores everything by [`crate::Id`], this should be used when your widget can have different data for different instances of the widget.
//!
//! # `serializable`
//!
//! [`AnyMap`] and [`serializable::AnyMap`] has exactly the same interface, but [`serializable::AnyMap`] only requires serde traits for stored object under `persistent` feature. Same thing for [`AnyMapId`] and [`serializable::AnyMapId`].
//!
//! # What could broke
//!
//! Things here could broke only when you trying to load this from file.
//!
//! First, serialized `TypeId` in [`serializable::AnyMap`] could broke, if you updated version of Rust compiler between runs.
//!
//! Second, count and reset all instances of type in [`serializable::AnyMapId`] could return incorrect value for the same reason.
//!
//! Deserialization errors of loaded elements of these storages can be determined only when you call `get_...` functions, they not logged and not provided to user, on this errors value is just replaced with `or_insert()`/default value.
//!
//! # When not to use this
//!
//! Basically, this is not for important widget data. Some errors just ignored and correct value of type is inserted when you call. This is done in order to simple interface.
//!
//! You shouldn't use any map here when you need very reliable state storage with rich error-handling. For this purposes you should create your own `Memory` struct and pass it everywhere you need it. Then, you should de/serialize it by yourself, handling all serialization or other errors as you wish.

mod any_map;
mod element;
mod id_map;

/// Same structs and traits, but also can be de/serialized under `persistence` feature.
pub mod serializable;

pub use self::{any_map::AnyMap, element::AnyMapTrait, id_map::AnyMapId};
