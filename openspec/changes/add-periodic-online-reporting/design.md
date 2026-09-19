# Design

## Context

Workers currently call deterministic advancement and persist the resulting
state, while simulation already captures level-up and act-completion report
events. Reporting can construct and send a browser-compatible request from a
managed character's private credential, but only foreground CLI submission uses
that path. The dashboard has confirmed lifecycle controls and intentionally
does not currently access reporting.

## Goals / Non-Goals

**Goals:**

- Deliver each persisted level-up and act-completion event once from an active
  online worker.
- Preserve local progression independently of automatic delivery results.
- Expose immediate dashboard manual brag using the existing safe reporting
  outcome model.
- Keep credentials, raw responses, and signed URLs outside UI and diagnostics.

**Non-Goals:**

- Durable delivery queues, retries, replay after restart, or exactly-once
  delivery.
- Reporting offline characters, changing simulation transitions, or adding a
  new endpoint configuration.

## Decisions

### Persist progression before sending its report

The worker will use the simulation trace, atomically persist the resulting
canonical state, and only then submit the event reports. This prevents a
leaderboard update for progress that local storage failed to retain. A crash
after persistence but before delivery may lose a report; that is accepted by
the chosen best-effort policy.

Sending before persistence was rejected because it can make remote state lead
the durable local state. A transactional outbox was rejected because it adds
delivery state and retry/duplicate semantics outside the agreed scope.

### Send each emitted event once without retry

Each persisted level-up or act-completion event causes one official-endpoint
delivery attempt. Failed or rejected outcomes do not block the worker and are
not retained for later delivery. The worker will make the outcome observable
through its established runtime error/status surface without including
credentials or request data.

### Reuse manual reporting for the dashboard action

The dashboard's Brag key will invoke the existing report construction and
delivery path with the selected managed character. It sends immediately,
without the lifecycle confirmation overlay, then refreshes the displayed
state/status and shows the safe outcome. Eligibility and endpoint validation
remain centralized in reporting rather than duplicated by the dashboard.

## Risks / Trade-offs

- [Delivery failure or process exit loses a report] -> Accept the loss under
  the explicit best-effort policy; retain progression and do not retry.
- [A worker encounters multiple report events in one advancement] -> Deliver
  each trace event in browser order after persistence.
- [Automatic traffic surprises an operator] -> Limit it to online worker
  events and document it in CLI help and the roadmap.
- [Dashboard report fails while rendering] -> Preserve the last safe snapshot
  and display only the safe error/outcome.

## Migration Plan

1. Extend reporting to deliver worker trace events from an active online
   character without exposing its credential.
2. Replace worker advancement with trace-aware persistence followed by
   best-effort event delivery.
3. Add the dashboard Brag action and its safe outcome presentation.
4. Add transport-boundary, worker, dashboard, and CLI tests.
5. Roll back by disabling worker event delivery and removing the dashboard
   action; persisted character state remains compatible.
