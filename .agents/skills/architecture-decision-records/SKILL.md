---
name: architecture-decision-records
description: Use when documenting, drafting, reviewing, or updating architecture decisions, ADRs, decision logs, tradeoffs, rationale, consequences, alternatives, or architecture decision history.
---

# Architecture Decision Records

## Overview

An Architecture Decision Record captures one architecturally significant decision, its rationale, tradeoffs, and consequences. Optimize for future readers reconstructing decision history.

## When to Use

Use when creating, updating, reviewing, or explaining an ADR, architecture decision, decision log, tradeoff analysis, rationale, consequences, alternatives, or status.

Do not use ADRs for transient implementation details, meeting notes, or insignificant decisions. If significance is unclear, ask what future maintainers will need to know.

## Workflow

1. Identify the single decision. Split multiple decisions into multiple ADRs.
1. Capture known facts only: context, requirements, constraints, options, rationale, decision makers, consequences.
1. If facts are missing, mark them as `Unknown` or ask a focused question; do not invent context, options, or quality attributes.
1. Write honest consequences: benefits, downsides, follow-up.
1. Preserve history: supersede old accepted ADRs; do not rewrite them away.

## Review Checklist

- One decision, not a bundle
- Significant: affects structure, quality attributes, constraints, or evolution
- Context explains why it existed
- Rejected options/tradeoffs are explicit
- Rationale is tied to requirements, not preference
- Downsides and follow-up work are recorded
- Status is clear
- Unknowns are marked, not invented

## Example

<!-- taken from https://joshrotenberg.com/adrs/formats.html#structure-1 -->
```markdown
---
status: accepted
date: 2024-01-15
decision-makers:
  - Alice
  - Bob
consulted:
  - Carol
informed:
  - Dave
---

# Use PostgreSQL for Persistence

## Context and Problem Statement

We need a database for storing user data.

## Decision Drivers

* Need ACID compliance
* Team has PostgreSQL experience
* Open source preferred

## Considered Options

* PostgreSQL
* MySQL
* MongoDB

## Decision Outcome

Chosen option: "PostgreSQL", because it meets all requirements and the team is familiar with it.

### Consequences

* Good, because we have team expertise
* Bad, because it requires more infrastructure than SQLite

### Confirmation

We will confirm this decision after the first production deployment.

## Pros and Cons of the Options

### PostgreSQL

* Good, because ACID compliant
* Good, because team experience
* Neutral, because requires server setup

### MySQL

* Good, because widely used
* Bad, because different SQL dialect

### MongoDB

* Good, because flexible schema
* Bad, because not ACID compliant by default
```

## Common Mistakes

| Mistake | Fix |
| --- | --- |
| Several decisions in one ADR | Split them. |
| Sales pitch | Include rejected options and negative consequences. |
| Invented context | Mark unknowns or ask. |
| Treating status as decoration | Use status to show lifecycle and link superseding ADRs. |
| Rewriting history | Keep old ADR; create/link superseding ADR. |
| Omitting alternatives under pressure | Include at least the rejected option and why it lost. |

## Sources

adr.github.io: home, ADR templates, AD practices.
