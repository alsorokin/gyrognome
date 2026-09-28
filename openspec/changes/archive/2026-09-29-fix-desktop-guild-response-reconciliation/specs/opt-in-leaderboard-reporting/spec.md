## MODIFIED Requirements

### Requirement: Credential-safe report delivery

The reporting surface SHALL construct profile-compatible manual-brag,
motto-change, and guild requests from coherent persisted state, online profile,
and retained private authentication. Browser delivery SHALL retain its existing
official endpoint behavior. Desktop delivery SHALL use only an explicitly
verified allowlisted official HTTPS endpoint and authentication contract; a
saved host or URL userinfo SHALL NOT authorize a destination. Redirects SHALL
NOT leak authentication or switch to an unapproved endpoint or HTTP.

Output SHALL contain only safe identity, operation, eligibility reason, and
categorized outcome. Logs/errors SHALL NOT contain passkeys, account
logins/passwords, signed or authenticated URLs, raw saves/responses, browser
profiles, or normalized response text. Successful HTTP status SHALL mean
delivered, not proof of normal leaderboard classification.

Guild responses SHALL be bounded and matched only to sanitized
profile-and-realm-specific fingerprints produced by the same versioned
normalization contract as the live evidence. The production classifier SHALL
supply the same ordered categories of dynamic values and use the same
replacement marker as the evidence generator. After an indeterminate desktop
guild response, the same explicit action MAY perform one bounded,
credential-free read of the verified realm page and treat an exact character
and case-insensitive ASCII guild match as acceptance without retrying the guild
mutation.

#### Scenario: Delivering an eligible report

- **WHEN** the approved endpoint returns successful HTTP status for manual brag
  or motto change
- **THEN** only a safe delivered outcome is reported, without a classification
  claim

#### Scenario: Categorizing a guild response

- **WHEN** a guild response matches a validated accepted or rejected
  fingerprint for the character's profile and realm
- **THEN** the matching safe category is exposed and no public fallback is
  required

#### Scenario: Reproducing a live-evidence fingerprint

- **WHEN** production classifies a response represented by a credential-free
  live-evidence vector
- **THEN** it uses the identical normalization version, ordered dynamic-value
  categories, replacement marker, and SHA-256 encoding

#### Scenario: Guild response fingerprint is unknown

- **WHEN** a bounded guild response matches no validated accepted or rejected
  fingerprint
- **THEN** the response outcome remains indeterminate unless the one permitted
  public-profile observation verifies the requested membership, and neither
  raw nor normalized response text is exposed or persisted

#### Scenario: Handling delivery failure

- **WHEN** delivery fails or returns unsuccessful HTTP status
- **THEN** a safe failure preserves state not explicitly locally persistent
  under the requested action and triggers no automatic retry

#### Scenario: Save contains an unapproved endpoint

- **WHEN** a desktop save supplies an arbitrary host, URL userinfo, or an
  endpoint with no verified HTTPS mapping
- **THEN** no credentials are transmitted and online eligibility explains the
  unsupported endpoint without echoing its raw value
