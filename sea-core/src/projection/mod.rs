pub mod contracts;
pub mod engine;
pub mod protobuf;
pub mod registry;

pub use contracts::{find_mapping_rule, find_projection_override};
pub use engine::ProjectionExporter;
pub use protobuf::{
    CompatibilityChecker, CompatibilityMode, CompatibilityResult, CompatibilityViolation,
    ProtobufEngine, ProtoFile, ProtoMessage, ProtoField, ProtoType, ScalarType,
    SchemaHistory, ViolationType, WellKnownType,
};
pub use registry::ProjectionRegistry;
