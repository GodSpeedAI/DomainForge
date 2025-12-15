pub mod buf;
pub mod contracts;
pub mod engine;
pub mod protobuf;
pub mod registry;

pub use contracts::{find_mapping_rule, find_projection_override};
pub use engine::ProjectionExporter;
pub use protobuf::{
    CompatibilityChecker, CompatibilityMode, CompatibilityResult, CompatibilityViolation,
    ProtoCustomOption, ProtoField, ProtoFile, ProtoMessage, ProtoOptionValue, ProtoOptions,
    ProtoRpcMethod, ProtoService, ProtoType, ProtobufEngine, ScalarType, SchemaHistory,
    StreamingMode, ViolationType, WellKnownType,
};
pub use registry::ProjectionRegistry;
