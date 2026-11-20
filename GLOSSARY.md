# DomainForge semantic modeling

DomainForge resolves declared business meaning into canonical semantic representations with reproducible identity.

## Language

**Domain**:
A bounded field of meaning with declared concepts and relationships.

**Domain Model**:
A structured declaration of the distinctions a domain needs, including its concepts, entities, relations, and constraints.

**Application Contract**:
A resolved declaration of a domain application’s types, operations, and constraints that its consumers can rely on.

**Entity**:
A declared type whose instances have domain identity and may carry state.

**Record**:
A declared structured value whose identity is determined by its contents rather than an entity identifier.

**Semantic Source Set**:
The collection of identified source texts resolved together to compile a declared domain model.

**Semantic World**:
A named resolved body of domain meaning. Work can pin a particular revision of that meaning.

**World Reference**:
The immutable reference identifying a named Semantic World at a particular compiled revision. A revision can change while the semantic closure remains equivalent.
_Avoid_: Closure Digest, Execution State Digest, world alias

**World Alias**:
A mutable selector that resolves to a World Reference.

**Semantic Closure**:
The normalized, resolved meaning of a domain model, including the declarations, contracts, imports, and semantic dependencies that determine it.

**Closure Digest**:
The fingerprint used to compare resolved semantic closures under the same scheme.

**Semantic Projection**:
A derived representation of a domain model shaped for a consuming system while preserving its declared meaning and provenance.

## Sources and boundary

The [SEA application contract](docs/reference/sea-application-contract.md) governs these terms. Its world identity contract derives `world_ref` from the full compiled model identity and records `semantic_closure_hash` separately. A comment-only source edit can therefore change World Reference while preserving Closure Digest. Horizon's existing closure-based startup pin has a different equality criterion.

Semantic projection preserves the applicable declared distinctions. An Execution World or Workbench Execution State Digest may be related by recorded provenance, but neither names a Semantic World revision. CEP carries the World Reference in its applicable semantic exchange profiles.
