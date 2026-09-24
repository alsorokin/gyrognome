## 1. Canonical Prime Stat derivation

- [x] 1.1 Add a pure browser-compatible Prime Stat refresh that derives `Stats.best` and `beststat` from current prime-stat values in fixed browser order, and verify focused simulation tests cover winner changes, integer formatting, and ties.
- [x] 1.2 Invoke Prime Stat refresh at canonical successor-state and credential-free transition-snapshot boundaries alongside Specialty refresh, and verify deterministic advancement retains final-state and Alea-continuation conformance.

## 2. Reporting integration

- [x] 2.1 Refresh Prime Stat in each explicit reporting preparation path so manual brags and motto changes serialize current field `k` values, and verify report-path tests begin with stale metadata and assert the derived Prime Stat.
- [x] 2.2 Verify worker-generated level-up and act-completion report snapshots serialize the derived Prime Stat while preserving report endpoint validation, field order, and signing behavior.

## 3. Regression validation

- [x] 3.1 Update only affected synthetic report-trace or checkpoint expectations and verify fixture-safety checks still reject sensitive data.
- [x] 3.2 Run the smallest focused Rust test selections covering simulation, report tracing, reporting, and runtime persistence; fix only failures caused by Prime Stat refresh behavior.
