# desktop-save-compatibility Specification

## Purpose

Import original desktop Progress Quest saves without executing serialized
components, and continue supported characters under an explicit, evidenced
desktop-6.4.4 compatibility contract rather than browser rules.

## Requirements

### Requirement: Bounded desktop save decoding

The system SHALL identify saves by content (`.pq` or same-format `.bak`) and
parse zlib/Delphi streams as data. It SHALL bound compressed/inflated sizes,
nesting, strings, lists, and rows; reject malformed lengths, truncation,
unsupported layouts, duplicate/missing state, and unexplained trailing bytes
with credential-safe errors. At most one bounded Gyrognome metadata block may
trail. Neither operation SHALL execute components, launch the client, mutate
source files, or contact servers.

#### Scenario: Importing a supported backup

- **WHEN** a valid supported desktop save is supplied with a `.bak` extension
- **THEN** it decodes to the same state as identical content named `.pq`

#### Scenario: Rejecting malformed or oversized content

- **WHEN** a compressed stream or component/list record exceeds a declared
  bound, contradicts its length, or uses an unsupported layout
- **THEN** import fails without partial registration, unbounded allocation,
  executing content, or including raw property values in the error

#### Scenario: Rejecting conflicting state components

- **WHEN** a save duplicates a state-bearing component, lacks required state,
  or has unexplained bytes after the supported component sequence
- **THEN** import fails rather than accepting whichever value was read last

#### Scenario: Rejecting content after the metadata block

- **WHEN** a save has a Gyrognome metadata block followed by more bytes, or
  more than one block
- **THEN** import fails as unexplained trailing content

### Requirement: Evidence-based desktop state interpretation

The system SHALL import desktop traits, attributes, equipment,
ordered inventory/spells, quests/plots, bars, activity, commands,
markers/indexes, prized equipment, and style. It SHALL apply only evidenced
load defaults, prefer numeric over visual state, validate ranks, indexes,
commands, and ranges, and preserve desktop identities without browser mapping.
Text support SHALL initially be ASCII; non-ASCII state/credential text SHALL
fail safely without lossy decoding or code-page guesses.

#### Scenario: Loading omitted default properties

- **WHEN** a supported fresh save omits empty spells, empty quests, a zero bar
  position, or an empty optional profile value
- **THEN** the importer applies the evidenced desktop default without treating
  the entire save as corrupt or inventing other required state

#### Scenario: Ignoring a stale progress hint

- **WHEN** a quest bar's display hint says complete but its numeric position is
  zero
- **THEN** the imported bar is numerically zero and its display is derived from
  that state

#### Scenario: Preserving distinct desktop spells

- **WHEN** a save includes distinct desktop spells such as Gyp and Shoelaces
- **THEN** their names, ranks, and collection positions remain distinct

#### Scenario: Rejecting unsupported text or indexes

- **WHEN** a save contains non-ASCII state-bearing text, an invalid rank, or an
  out-of-range rule index without an evidenced legacy interpretation
- **THEN** import fails safely before simulation and does not substitute a
  browser name, replacement character, or arbitrary rule entry

### Requirement: Explicit desktop continuation provenance

The system SHALL distinguish source format, evidenced legacy adaptation, and
the selected desktop-6.4.4 continuation profile. It SHALL support observed
6.2/6.4.4 layouts only through validated 6.4.4 load/continuation semantics,
apply pinned reference behavior to legacy state, reject ambiguous state, and
report unavailable history. New local counters SHALL be labeled as measured
since import.

#### Scenario: Avoiding unsupported writer-version claims

- **WHEN** a save's writer version is inferred only from its filename,
  executable version resource, realm name, or absence of newer fields
- **THEN** the system does not claim an exact writer version

#### Scenario: Applying legacy load behavior

- **WHEN** a supported save uses a legacy prologue queue, quest placeholder, or
  load-time spelling change
- **THEN** interpretation follows the pinned source-derived reference,
  including collection-order effects

#### Scenario: Rejecting ambiguous legacy state

- **WHEN** a save contains ambiguous state without a supported interpretation
- **THEN** import rejects it with a safe, actionable explanation instead of
  silently converting it to browser rules

#### Scenario: Importing a legacy prologue

- **WHEN** a supported 6.2-shaped prologue is imported
- **THEN** it follows the verified 6.4.4 loading adaptation for that queue and
  records the adaptation without claiming exact 6.2 runtime continuation

#### Scenario: Encountering a carried-forward quest placeholder

- **WHEN** a supported save retains the legacy `fQuest` placeholder
- **THEN** interpretation follows the evidenced load and next-transition
  behavior rather than parsing the placeholder as a normal monster identity

#### Scenario: Displaying unavailable history

- **WHEN** an imported character lacks saved RNG state, birthday, seed history,
  task counts, or elapsed totals
- **THEN** inspection identifies the missing history and distinguishes newly
  measured counters from historical totals

### Requirement: Desktop rules and random continuation

The desktop profile SHALL follow the pinned 6.4.4 rules and deterministic
continuation behavior, including progression and reward semantics.

#### Scenario: Matching the desktop reference rules

- **WHEN** the profile performs ordered rule lookup, random operations,
  weighted-stat selection, numeric rounding, or load behavior
- **THEN** results match the pinned 6.4.4 reference harness

#### Scenario: Initializing a desktop random continuation

- **WHEN** a desktop save is imported
- **THEN** a new desktop random state is initialized and persisted without
  claiming the original client's unsaved next outcome or exact uncorroborated
  Delphi edge behavior

#### Scenario: Repeating deterministic desktop callbacks

- **WHEN** imported state, desktop random state, and callback inputs are
  identical
- **THEN** results and restored-state continuation are identical

#### Scenario: Crediting a fractional-second task duration

- **WHEN** a supported task with maximum duration 5,114 milliseconds completes
- **THEN** each applicable XP, quest, and plot credit uses 5 whole seconds,
  not 5.114 seconds

#### Scenario: Completing a noncombat task

- **WHEN** a non-loading noncombat task completes with applicable plot progress
- **THEN** desktop plot credit is applied even though the task is not combat

#### Scenario: Ordering desktop level-up effects

- **WHEN** a desktop task triggers a level-up
- **THEN** level-up effects occur in the pinned 6.4.4 order

#### Scenario: Entering Act II

- **WHEN** the desktop transition enters Act II
- **THEN** its reward includes an item and excludes the later-act equipment
  award

#### Scenario: Entering Act III or later

- **WHEN** the desktop transition enters Act III or a later act
- **THEN** its reward includes both an item and equipment

#### Scenario: Selecting the desktop specialty

- **WHEN** multiple learned spells contribute to specialty
- **THEN** the selected value is the greatest of (saved learned-list position
  plus one) times rank, with ties retaining the earliest spell

#### Scenario: Resolving tied prime stats

- **WHEN** multiple stats tie for the prime stat
- **THEN** the earliest stat in STR, CON, DEX, INT, WIS, CHA order is selected

#### Scenario: Resuming desktop randomness

- **WHEN** a character is saved and restored after a known desktop random
  sequence
- **THEN** its subsequent outcomes match uninterrupted desktop-profile
  execution and do not use Alea

### Requirement: Offline desktop protocol construction

The system SHALL construct desktop request data without transport and reject
unsupported encoding or authentication explicitly.

#### Scenario: Constructing desktop protocol payloads

- **WHEN** progress, motto, or guild request data is constructed
- **THEN** it uses revision 8, evidenced desktop byte encoding and field order,
  and validates the passkey without sending a request

#### Scenario: Matching exact transition snapshots

- **WHEN** a desktop level-up or act-completion snapshot is supplied
- **THEN** unsigned fields and the synthetic validator match the
  source-derived desktop vector at that exact boundary

#### Scenario: Reporting before any spell is learned

- **WHEN** a synthetic desktop character without learned spells produces a
  progress payload
- **THEN** it contains revision 8 and no `z` field

#### Scenario: Constructing a desktop report boundary

- **WHEN** a synthetic desktop level-up or act-completion snapshot is supplied
- **THEN** the unsigned fields and synthetic validator match the source-derived
  desktop vector at that exact boundary without sending a request

#### Scenario: Retaining account authentication

- **WHEN** a desktop save contains account credentials and a passkey
- **THEN** both remain available privately to the selected transport contract,
  with neither exposed through request previews or diagnostic output

#### Scenario: Rejecting unsupported protocol inputs

- **WHEN** request construction encounters unsupported encoding or
  authentication
- **THEN** it fails explicitly rather than producing a request payload

### Requirement: Desktop save export

The system SHALL export managed desktop characters as compatible `.pq` saves
that round-trip to the same canonical state.

#### Scenario: Exporting without external effects

- **WHEN** a managed desktop character is exported
- **THEN** export includes its passkey, online credentials, and Gyrognome
  metadata without launching the client, contacting a server, or retaining a
  raw binary save

#### Scenario: Round-tripping an exported desktop character

- **WHEN** a registered desktop character is exported and the file is imported
- **THEN** the imported canonical state equals the stored state

#### Scenario: Opening an export in the original client

- **WHEN** an exported `.pq` is opened in pq.exe 6.4.4
- **THEN** the client loads the character and shows the same traits, stats,
  equipment, inventory, spells, plots, quests, bars, and activity, without
  an error caused by the metadata block

### Requirement: Gyrognome metadata in desktop exports

Metadata SHALL preserve local-only character state across desktop export and
import, and invalid metadata SHALL fail import.

#### Scenario: Preserving metadata state

- **WHEN** an export includes a metadata block
- **THEN** it carries since-import task and elapsed-time counters, desktop
  random continuation, source provenance and adaptations, advancement
  provenance, a version, and a digest of its desktop components

#### Scenario: Keeping metadata private and client-compatible

- **WHEN** a metadata block is written to a desktop export
- **THEN** it contains no credentials and is placed where pq.exe 6.4.4 neither
  reads nor rewrites it

#### Scenario: Re-importing a Gyrognome export

- **WHEN** a desktop character that has completed tasks, has advanced locally,
  and has a random continuation is exported and imported into another
  Gyrognome store
- **THEN** its counters, random continuation, provenance, and local-only
  advancement equal the exported character's, and counters remain labeled as
  measured since the first import

#### Scenario: Saving the export in the original client

- **WHEN** pq.exe 6.4.4 saves a character that was loaded from a Gyrognome
  export
- **THEN** the resulting save has no metadata block and imports as a fresh
  official-client save

#### Scenario: Rejecting a mismatched block

- **WHEN** the desktop components of an export are changed but its metadata
  block is kept
- **THEN** import fails with a credential-safe error and does not register the
  character

#### Scenario: Rejecting unsupported metadata

- **WHEN** a metadata block has an unsupported version or content
- **THEN** import fails rather than ignoring the block

#### Scenario: Importing a save without metadata

- **WHEN** a desktop save has no Gyrognome metadata block
- **THEN** it imports with fresh registration values
