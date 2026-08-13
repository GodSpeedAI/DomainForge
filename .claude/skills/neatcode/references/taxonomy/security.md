# Security failures

Failures of trust. Load this family whenever input crosses a trust boundary, an authorization
decision is made, a secret is handled, or a query, command, path, or template is constructed
from data.

Two rules govern findings here:

- **Name the reachable path.** A vulnerability with no traced route from untrusted input to
  the unsafe use is a *possible* finding at best. Trace it or say you could not.
- **Do not deflate.** When the path is real, it is S1 regardless of how small the diff is.

---

### Missing authorization check

**Is** — An operation that verifies who the caller is but not whether they may do this.

**Signals** — A handler that authenticates and then acts on an id from the request without
checking ownership. A resource fetched by id with no tenant scoping. An admin action behind
authentication only. A GraphQL field resolver with no field-level check. A "internal" endpoint
reachable from outside.

**Cause** — Conflating authentication with authorization.

**Why agents** — Authentication middleware is visible in the framework and looks like the
security layer. Per-resource authorization is domain logic that has to be decided.

**Risk** — Insecure direct object reference. Any authenticated user reads or modifies any
other user's data. This is the most common serious vulnerability in generated web code.

**Trajectory** — The pattern is copied to every new endpoint.

**Exception** — Authorization enforced in a layer you can point at — a policy middleware, a
row-level security rule, a scoped query builder that cannot produce an unscoped query.

**False positive** — The scoping happens in the repository via an ambient tenant context.
Verify it rather than assuming it.

**Fix** — Authorize the *resource*, not just the session. Prefer a query that cannot return
another tenant's rows over a check that can be forgotten.

**Proof** — A test where user A requests user B's resource and receives 403 or 404.

---

### Authentication and authorization confused

**Is** — Treating identity as permission.

**Signals** — `if (user) { allow }`. A role checked in the UI only. A JWT accepted without
verifying signature, issuer, audience, or expiry. `alg: none` accepted. A permission derived
from a client-supplied claim. A role read from a request header.

**Cause** — Not modelling permission as a separate decision with its own inputs.

**Why agents** — "Logged in" is the visible gate in most example code.

**Risk** — Privilege escalation, often trivially.

**Trajectory** — A permission model accretes as scattered conditionals with no single truth.

**Exception** — A system with genuinely one role, stated as such.

**False positive** — Claims are verified centrally in middleware you did not read.

**Fix** — Verify the token properly. Derive permissions server-side from server-side state.
Check the specific permission at the point of the operation.

**Proof** — A test with a valid token and insufficient permission; a test with a forged token.

---

### Insecure default

**Is** — A default that is convenient rather than safe.

**Signals** — TLS verification off. Permissive CORS (`*` with credentials). Debug mode on.
A default password or key. Permissive file modes. A public-by-default resource. Verbose errors
returned to clients. An open-by-default feature flag. Long-lived tokens.

**Cause** — Defaulting to what makes local development work.

**Why agents** — Development-friendly configuration is heavily represented in examples and
starter templates.

**Risk** — Ships to production unnoticed, because it works.

**Trajectory** — Becomes the template for every subsequent service.

**Exception** — Local-only configuration that cannot reach production, provably.

**False positive** — Overridden in the production configuration. Check the whole config chain.

**Fix** — Default closed. Require explicit opt-in for the permissive setting, and fail startup
if a required secret is absent rather than falling back.

**Proof** — Configuration test asserting the secure default; a startup check.

---

### Injection

**Is** — Building an interpreted string from untrusted data.

**Signals** — String-concatenated or template-interpolated SQL. A shell command built from
input, or `shell: true` with interpolation. A path joined from a user-supplied segment with no
containment check. HTML built by concatenation. A regex built from input. LDAP, XPath, or
NoSQL query operators taken from a request body. A prompt built from untrusted text and used
to drive tool calls.

**Cause** — Treating a structured language as text.

**Why agents** — String building is the most natural way to produce any of these, and the safe
API is library-specific.

**Risk** — Data disclosure, data destruction, remote code execution, path traversal.

**Trajectory** — One instance teaches the pattern to the rest of the codebase.

**Exception** — Values provably from a fixed allowlist — identifiers checked against a literal
set, not merely validated by shape.

**False positive** — The library parameterizes automatically, or the "interpolation" is of a
constant.

**Fix** — Parameterized queries. `execFile` with an argument array. Path resolution plus a
containment check against a base directory. Contextual output encoding. For identifiers that
cannot be parameterized, map through an allowlist.

**Proof** — A test with a classic injection payload asserting it is treated as data.

---

### Unsafe deserialization

**Is** — Constructing objects from untrusted bytes with a format that can express behaviour.

**Signals** — `pickle.loads`, `yaml.load` without `SafeLoader`, Java native deserialization,
`eval`/`Function` on input, prototype-polluting merges of parsed JSON, XML parsing with
external entities enabled, unbounded decompression.

**Cause** — Choosing the most convenient parser.

**Why agents** — The convenient call is the common one in examples.

**Risk** — Remote code execution, SSRF and file disclosure via XXE, memory exhaustion.

**Trajectory** — Deserialization boundaries spread and become impossible to enumerate.

**Exception** — Provably trusted, integrity-checked input from inside the trust boundary.

**False positive** — The library is safe by default in the installed version. Verify the
version.

**Fix** — Safe loaders. Schema-validated parsing into known types. Disable external entities.
Bound input size and expansion.

**Proof** — A test with a hostile document asserting rejection.

---

### Secret leakage

**Is** — A credential reaching somewhere it should not.

**Signals** — A key in source or in a committed config. A token in a log line, an error
message, or a URL query string. Credentials in a stack trace returned to a client. A secret in
a client bundle or a build artifact. A `.env` file committed. A secret passed as a command-line
argument, visible in the process table.

**Cause** — Secrets treated as configuration values rather than as a distinct category.

**Why agents** — Inlining a value makes the code work immediately.

**Risk** — Credential compromise, and a rotation cost far exceeding the original work.

**Trajectory** — Once in git history, it is compromised permanently regardless of later
removal.

**Exception** — Deliberately public keys, or test fixtures that are obviously not real.

**False positive** — A placeholder. Confirm it is not a real value before raising an alarm —
and confirm it *is* a placeholder before dismissing one.

**Fix** — Load from the environment or a secret manager. Redact in logs and error paths. If it
was committed, it must be rotated, not merely deleted.

**Proof** — A secret scan; a log assertion that the value is redacted.

---

### Trust-boundary confusion

**Is** — Treating data as trusted after it has crossed from an untrusted source.

**Signals** — Validating at one entry point and not another. Trusting a value because it
"came from our own service." Client-supplied prices, roles, ids, or totals used directly. A
signed value used without verifying the signature. Data trusted after a round trip through a
queue or a cache. Internal APIs with no authentication because they are "internal."

**Cause** — No explicit map of where trust begins.

**Why agents** — The boundary is architectural knowledge, invisible in any one function.

**Risk** — Validation bypassed via a secondary path, which is exactly how these are exploited.

**Trajectory** — As entry points multiply, coverage becomes probabilistic.

**Exception** — A genuinely closed network with authenticated service-to-service calls, stated
as such.

**False positive** — Validation happens at a gateway you did not inspect.

**Fix** — Name the trust boundary. Validate on entry, once, and carry the validated form in a
distinct type so the compiler tracks it.

**Proof** — Every entry point routes through the same validation; a test per entry point.

---

### Validation after unsafe use

**Is** — Checking the input after acting on it.

**Signals** — A file read, then a path check. A query executed, then a permission check. A
resource created, then a quota check. Logging a raw value before redaction. A redirect
followed before the URL is validated.

**Cause** — Ordering by narrative rather than by safety.

**Why agents** — The main action is the point of the function; the check is added around it
afterwards.

**Risk** — The check is decorative. The damage is already done — including via timing and
error-message oracles.

**Trajectory** — The pattern is copied; the checks look present in review.

**Exception** — None. Order matters.

**False positive** — An earlier check exists further up the call stack.

**Fix** — Validate and authorize before the effect. Make the unsafe operation take an
already-validated type.

**Proof** — A test asserting no side effect occurs when validation fails.
