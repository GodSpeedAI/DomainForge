//! Domain-code projection family: one language-neutral IR (`ir.rs`) consumed
//! by three pure renderers (`python.rs`, `typescript.rs`, `rust.rs`).
//!
//! # CORE PRINCIPLE
//! All DDD/CQRS semantics — what becomes an aggregate, a command's name, an
//! event's name, an error's name, a port's methods — are decided exactly
//! once in [`ir::DomainIr::from_graph`]. Renderers translate IR → language
//! syntax and NOTHING else: a renderer never calls `graph.*`, never
//! re-derives a name, never invents a construct the IR does not carry.

pub mod ir;
pub mod python;
pub mod rust;
pub mod typescript;
