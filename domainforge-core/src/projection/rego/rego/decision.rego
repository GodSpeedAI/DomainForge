# Decision contract `domainforge.rego.decision/v1`.
#
# `conditions` (generated, policies.rego) holds each policy's three-valued
# result. This module validates the input contract and aggregates the results
# into allow / deny / escalate. The aggregation is a target adaptation: the
# DomainForge evaluator reports per-policy violations and has no aggregate
# outcome, so the rule below is this projection's own, mirrored by
# `projection::rego::reference` and recorded as `adapter` in the fidelity
# report.
package domainforge.policy

import rego.v1

import data.domainforge.model
import data.domainforge.sea.lib

_facts_ok if is_object(input.facts)

_collection_ok(c) if {
	is_array(input.facts[c])
	every item in input.facts[c] {
		is_object(item)
	}
}

input_errors := ["facts: expected object"] if {
	not _facts_ok
} else := sort([msg |
	some c in model.collections
	not _collection_ok(c)
	msg := sprintf("facts.%s: expected array of objects", [c])
])

_result(key) := "error" if {
	count(input_errors) > 0
} else := c if {
	c := lib.value(conditions[key])
	c in {"true", "false", "unknown", "error"}
} else := "error"

results[key] := _result(key) if some key, _ in model.policies

_code(r) := "malformed_input" if {
	count(input_errors) > 0
} else := "violated" if {
	r == "false"
} else := "unknown" if {
	r == "unknown"
} else := "evaluation_error"

_reason_rows := sort([[0 - p.priority, key, reason] |
	some key, p in model.policies
	r := results[key]
	r != "true"
	reason := {
		"policy": key,
		"code": _code(r),
		"modality": p.modality,
		"severity": p.severity,
		"priority": p.priority,
		"blocking": p.severity == "error",
	}
])

reasons := [row[2] | some row in _reason_rows]

_blocking_codes := {reason.code | some reason in reasons; reason.blocking}

outcome := "deny" if {
	count(input_errors) > 0
} else := "deny" if {
	count(_blocking_codes & {"violated", "evaluation_error", "malformed_input"}) > 0
} else := "escalate" if {
	"unknown" in _blocking_codes
} else := "allow"

decision := {
	"contract": "domainforge.rego.decision/v1",
	"outcome": outcome,
	"results": results,
	"reasons": reasons,
	"input_errors": input_errors,
	"model": model.identity,
}
