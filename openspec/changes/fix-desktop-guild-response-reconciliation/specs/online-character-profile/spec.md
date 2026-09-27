## MODIFIED Requirements

### Requirement: Explicit guild membership management

CLI and dashboard SHALL let an operator submit a guild designation for an
eligible online character while active or inactive. Non-empty input SHALL
request joining/changing guild; empty input SHALL request leaving. Each
confirmed submission SHALL attempt exactly one profile-compatible mutation.

Persisted guild SHALL change when the response matches an accepted fingerprint
produced by the same versioned normalization contract used for live evidence.
If the response remains indeterminate, the same explicit action MAY perform
one credential-free read of the verified realm page. The observed guild SHALL
be persisted only when the exact character is present and the observed guild
matches the submitted ASCII designation case-insensitively; the server's
canonical capitalization SHALL be retained. An observed empty membership
SHALL similarly confirm an explicit leave request.

Rejected, unverifiable, oversized, unsuccessful, or undeliverable outcomes
SHALL preserve the prior designation. Reconciliation SHALL NOT repeat the guild
mutation.

Designations SHALL accept non-control Unicode text including empty text. The
initial desktop ASCII contract SHALL reject non-ASCII input before state
change/transport; browser Unicode behavior SHALL remain unchanged. Cancelling
SHALL preserve state and send nothing.

#### Scenario: Joining a guild

- **WHEN** a valid designation receives an acceptance response recognized by
  the shared evidence fingerprint contract
- **THEN** it is persisted and a safe accepted outcome is reported

#### Scenario: Submitting an empty guild name

- **WHEN** empty input receives a recognized acceptance response or an
  indeterminate response followed by a public observation of the exact
  character without a guild
- **THEN** the persisted designation is cleared, a safe accepted outcome is
  reported, and the mutation is not repeated

#### Scenario: Guild request is rejected or indeterminate

- **WHEN** the endpoint rejects a request, delivery fails, or neither the
  response fingerprint nor the bounded public observation verifies the
  submitted membership
- **THEN** a safe failure category preserves the prior guild without retrying
  the mutation

#### Scenario: Guild response is indeterminate but membership changed

- **WHEN** an explicit guild mutation has an indeterminate response and one
  credential-free read of the verified realm page shows the exact character in
  a guild matching the submitted ASCII designation case-insensitively
- **THEN** the observed canonical guild designation is persisted and the
  mutation is not repeated

#### Scenario: Cancelling dashboard guild input

- **WHEN** an operator cancels the editor
- **THEN** guild state is unchanged and no request occurs

#### Scenario: Wrong-profile guild evidence

- **WHEN** a desktop guild operation has only browser response fingerprints
- **THEN** it is refused before delivery rather than using those fingerprints
