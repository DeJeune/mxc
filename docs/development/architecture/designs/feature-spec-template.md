# Feature design: feature name

> **Audience:** MXC developers

Use this template when maintainers request a spec through the
[feature contribution process](../../guides/authoring-a-new-feature.md).
Copy it to `<feature-name>.md` in this directory and replace the title and
prompts with the proposed design. Keep it short; mark irrelevant sections
as not applicable and explain why.

Keep the status **Proposed** until a maintainer approves the design. The
spec must be approved and merged before implementation starts. Record that
decision in the spec PR. Link implementation PRs as they are opened.

- **Status:** Proposed
- **Issue:** Add the agreed GitHub issue.
- **Implementation PRs:** Not started.

## Problem and scope

- Who needs this change, and what problem does it solve?
- What should users be able to do? Include a concrete scenario.
- What is outside the scope?

## Proposed behavior

- Identify supported backends, host requirements, and affected operations.
  Include SDK and executor access, discovery, and validation where applicable.
- Define defaults and the meaning of omitted and explicitly empty values.
- Describe unsupported requests, errors, cancellation, and resource cleanup.

## SDK and contract compatibility

- List affected policy fields, request contracts, and public Rust, .NET, and
  Node APIs. State when an SDK or contract does not need to change.
- Identify any source, binary, or behavioral break for existing callers.
  Make breaking changes explicit; do not hide them in implementation detail.
- Explain how published APIs and contracts remain supported. Describe any
  new versioned API and migration path.

## Security and platform constraints

- Explain changes to isolation or trust boundaries and how restrictions are
  enforced. Unsupported policy must remain rejected.
- Identify authorization, privileges, and runtime capability checks.
- Describe intentional platform differences and unsupported combinations.
  Do not promise backend parity that MXC cannot enforce.

Report vulnerabilities through [SECURITY.md](../../../../SECURITY.md), not
in a public spec.

## Design approach and alternatives

- Summarize the existing implementation path this change extends and where
  responsibilities belong. Include only the internals needed to assess the
  design, not a walkthrough of every function.
- Describe alternatives considered and the reason for the proposed approach.

## Acceptance criteria and validation plan

- List observable pass/fail conditions for the proposed behavior.
- Identify functional and regression scenarios, relevant backends and hosts,
  and the CI suites that will run them. Include affected SDK operations.
- Cover unsupported inputs, defaults, omitted and empty values, interactions
  with existing policy, and failure cleanup where applicable.
- Explain any validation gaps. Distinguish planned tests from actual results;
  a design proposal does not need completed implementation tests.
- Identify any CI checks being replaced or retired and how coverage remains
  effective. Do not disable a failing check to make the feature pass.

## Delivery and documentation

- Identify separate policy/schema, backend, and SDK PRs where applicable.
  Follow the feature guide's order; do not expose unenforced policy or
  unsupported public APIs.
- Describe contract publication and runtime authorization changes separately.
- List the consumer documentation and examples that need updates. Keep
  contributor and implementation detail in developer documentation.

## Open questions

List unresolved decisions and who needs to resolve them. Resolve material
questions before accepting the spec.

## Review checklist

- [ ] The linked issue and scope have maintainer agreement.
- [ ] Scenarios, non-goals, defaults, and failure behavior are clear.
- [ ] Supported hosts, backends, enforcement, and security constraints are explicit.
- [ ] SDK and contract compatibility are addressed; any breaking changes are
  prominently identified for a maintainer decision.
- [ ] Acceptance criteria are testable and the CI plan preserves meaningful coverage.
- [ ] Delivery stages and consumer documentation updates are appropriate.
- [ ] Material open questions are resolved and maintainer approval is recorded
  in the spec PR before implementation.
