# One Canonical Semantic World

## Architectural position

A DomainForge domain model should describe one coherent semantic world.

The canonical entities, identities, relationships, constraints, states, and interactions that define that world belong in a single authoritative `.sea` model. This model is expected to be comprehensive and broad, but relatively stable. It represents the shared meaning upon which more variable system behavior depends.

The governing principle is:

> The semantic world is globally coherent; specialization occurs only at its concrete boundaries.

## Current DomainForge behavior

DomainForge 0.15.0 does not allow an imported entity type to back instances declared in another module, even when both modules use the same namespace.

For example, one `.sea` file cannot define the canonical `Journey` entity type while another imports `Journey` and declares concrete journey instances as members of the same validated world.

Consequently:

> One validated semantic world currently requires one canonical `.sea` file.

Smaller `.sea` files may remain useful as focused explanations, examples, or views, but DomainForge does not compose them into the canonical world. They are explanatory companions rather than authoritative semantic modules.

## Why this constraint is acceptable

Although this behavior is technically a limitation of DomainForge’s current module system, it reinforces a desirable modeling property.

The canonical domain model should not change frequently. It defines the stable language and structure through which the system interprets reality. Fragmenting this foundation across independently evolving modules would make it easier for meanings, identities, and assumptions to diverge.

Keeping the world together provides several benefits:

* Every canonical concept has one authoritative definition.
* Relationships and invariants can be validated against the complete world.
* Shared terms cannot silently acquire different meanings in different modules.
* Agents and humans can reason from the same closed semantic context.
* Changes to foundational meaning are explicit and reviewable.
* Declared reality remains coherent enough to compare with observed reality.
* Versioning, auditing, and semantic change control remain straightforward.

This architecture deliberately favors global coherence over local autonomy within the canonical model.

## Where specialization belongs

The canonical model defines what concepts mean. It should not absorb every concrete representation, provider behavior, API shape, or deployment-specific concern.

Specialization belongs at the distal end of the architecture, where stable semantic meaning meets a particular technical or operational environment.

The expected pattern is:

1. The canonical `.sea` model defines the semantic world.
2. Projection or port contracts expose the capabilities required at a boundary.
3. Provider configuration describes how a particular environment satisfies those contracts.
4. A concrete adapter translates between the canonical semantics and an external system.
5. Observations and evidence report what actually occurred without redefining canonical meaning.

For example, `WorkItem` should have one canonical meaning. Jira, Linear, and GitHub adapters may represent, observe, or manipulate it differently, but those adapters should not create competing definitions of what a `WorkItem` is.

## Expected variation by layer

| Layer                       | Responsibility                                   | Expected rate of change |
| --------------------------- | ------------------------------------------------ | ----------------------: |
| Canonical `.sea` world      | Meaning, identity, relationships, and invariants |                     Low |
| Projection or port contract | Required boundary capabilities                   |                Moderate |
| Provider configuration      | Environment-specific bindings and policies       |                  Higher |
| Concrete adapter            | API, SDK, protocol, and platform integration     |                 Highest |

This distribution keeps stable meaning near the center and volatile implementation details near the edges.

## Relationship to declared and observed reality

A coherent canonical world is particularly important when the system distinguishes declared reality from observed reality.

Declared reality establishes what the organization says should exist, what concepts mean, which relationships are valid, and which outcomes are intended. Observed reality records what the organization and its systems actually do.

The difference between the two is meaningful only when declared reality is sufficiently coherent. If separate modules can independently redefine parts of the declared world, an apparent difference may represent semantic disagreement between modules rather than evidence of a real operational deviation.

One canonical model therefore provides the stable reference against which observations, exceptions, experiments, and adaptations can be evaluated.

## Role of companion `.sea` files

Smaller `.sea` files may still serve useful explanatory purposes. They can:

* Present a focused slice of the model.
* Teach a particular concept or interaction.
* Demonstrate valid `.sea` syntax.
* Support discussion of a bounded concern.
* Preserve the intended boundaries of possible future projections or extensions.

They must not become competing sources of truth.

Companion files should avoid duplicating authoritative inventories or maintaining parallel definitions that can drift from the canonical model. Where possible, they should eventually be generated as projections or views of the canonical source.

The authority rule is:

> `interaction-model.sea` defines the world. Companion files explain or project it.

## Limitation versus policy

The current technical limitation and the desired architectural policy should be distinguished.

### Current technical limitation

DomainForge cannot compose a type defined in one `.sea` module with instances declared in another module as members of one validated semantic world.

### Desired architectural policy

Canonical domain truth should ordinarily remain under one coherent semantic authority rather than being fragmented across independently evolving modules.

The policy does not imply that DomainForge should never support stronger module capabilities. Future modularity remains useful for:

* Projections
* Observations
* Provider bindings
* Concrete adapters
* Bounded extensions
* Generated views
* Explanatory slices
* External-domain mappings

However, those capabilities should be asymmetric. Edge modules may import, map, observe, or specialize canonical concepts without silently redefining their identities or meanings.

## Desired future module semantics

A future DomainForge module system should preserve the following authority structure:

* The canonical world owns entity identities, types, invariants, and canonical relationships.
* Boundary modules may reference canonical concepts.
* Provider modules may map those concepts to concrete systems.
* Observation modules may report evidence about canonical entities.
* Extensions may add bounded information without changing established meaning.
* No module may silently override a canonical definition.
* Moving an edge-derived concept into the canonical world requires explicit review and settlement.

This would provide modularity where variation is valuable while preserving coherence where shared meaning is essential.

## Decision

For the current interaction model:

* `interaction-model.sea` will be the single authoritative and validated semantic source.
* The complete model will remain together as one coherent world.
* Smaller `.sea` files will remain valid explanatory companions.
* Companion files will not be treated as composable fragments of the canonical model.
* Concrete specialization will occur through projections, provider configuration, and adapters at the distal boundaries.
* Cross-module entity instantiation will remain documented as a DomainForge capability gap, but resolving it is not required to fragment the canonical model.

This is an intentional trade:

> We accept reduced modularity inside the canonical model in exchange for stronger semantic coherence, authority, and interpretability across the system.
