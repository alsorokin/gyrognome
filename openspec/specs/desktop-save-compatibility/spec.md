# desktop-save-compatibility Specification

## Purpose

Import original desktop Progress Quest saves without executing serialized
components, and continue supported characters under an explicit, evidenced
desktop-6.4.4 compatibility contract rather than browser rules.

## Requirements

### Requirement: Bounded desktop save decoding

The system SHALL recognize supported desktop save content independently of
filename extension, including `.pq` and same-format `.bak` files. It SHALL
decode the zlib container and concatenated Delphi component streams as data
only, using explicit bounds on compressed input, inflated output, component
nesting, strings, lists, and row counts. It SHALL reject malformed lengths,
truncation, unsupported layouts, duplicate state-bearing components, missing
required state, and unexplained trailing content with credential-safe errors.
A single bounded Gyrognome metadata block after the last component is the
only accepted trailing content. Inspection and registration SHALL NOT execute
components, launch the original client, mutate source files, or contact a
server.

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

The system SHALL import traits, attributes, equipment, ordered inventory and
spell rows, quest and plot history, numeric progress bars, current activity,
queued commands, quest markers/indexes, prized equipment, and game style from
supported desktop layouts. It SHALL interpret omitted defaults only where
desktop load semantics establish their value. Numeric state SHALL take
precedence over stale visual hints. It SHALL validate ranks, indexes, commands,
and numeric ranges before continuation, without normalizing desktop identities
to browser table entries.

The initial supported text subset SHALL be ASCII. Non-ASCII serialized text
that would affect imported state or credentials SHALL produce a safe
unsupported-encoding error, not lossy decoding or a guessed code page.

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

The system SHALL distinguish source format, supported legacy adaptation, and
the selected desktop-6.4.4 continuation profile. It SHALL NOT claim an exact
writer version from a filename, executable version resource, realm name, or
the absence of newer fields. It SHALL support observed 6.2 and 6.4.4 save
layouts only through validated 6.4.4 load/continuation semantics.

Legacy prologue queues, placeholder quest markers, and load-time spelling
changes SHALL follow the pinned source-derived desktop reference behavior,
including collection-order effects. An unsupported ambiguous state SHALL be
rejected with an actionable safe explanation rather than silently converted to
browser rules. Historical RNG state, birthday, seed history, task counts, or
elapsed totals not present in the save SHALL be identified as unavailable. New
local counters SHALL be labeled as measured since import.

#### Scenario: Importing a legacy prologue

- **WHEN** a supported 6.2-shaped prologue is imported
- **THEN** it follows the verified 6.4.4 loading adaptation for that queue and
  records the adaptation without claiming exact 6.2 runtime continuation

#### Scenario: Encountering a carried-forward quest placeholder

- **WHEN** a supported save retains the legacy `fQuest` placeholder
- **THEN** interpretation follows the evidenced load and next-transition
  behavior rather than parsing the placeholder as a normal monster identity

#### Scenario: Displaying unavailable history

- **WHEN** an imported character has no saved RNG or lifetime task history
- **THEN** inspection identifies the missing history and distinguishes newly
  measured counters from historical totals

### Requirement: Desktop rules and random continuation

The desktop profile SHALL use the pinned 6.4.4 ordered rules, integer random
operations, weighted-stat selection, numeric rounding, and load behavior
validated against the separately authored source-derived desktop reference
harness. An import SHALL initialize and persist a new desktop random
continuation state without claiming to reproduce the original client's unsaved
next random outcome or exact uncorroborated Delphi runtime edge behavior.
Identical imported state, supplied desktop random state, and callback inputs
SHALL yield identical results and restored-state continuation.

The profile SHALL preserve desktop integer XP/quest/plot credit, non-loading
task plot advancement, level-up ordering, and act rewards. Act II SHALL award
an item but no equipment; Act III onward SHALL award both. Specialty SHALL
select by saved learned-list position plus one times rank, retaining the
earliest tie. Prime-stat ties SHALL retain the earliest of STR, CON, DEX, INT,
WIS, and CHA.

#### Scenario: Crediting a fractional-second task duration

- **WHEN** a supported task with maximum duration 5,114 milliseconds completes
- **THEN** each applicable XP, quest, and plot credit uses 5 whole seconds,
  not 5.114 seconds

#### Scenario: Completing a noncombat task

- **WHEN** a non-loading noncombat task completes with applicable plot progress
- **THEN** desktop plot credit is applied even though the task is not combat

#### Scenario: Entering Act II

- **WHEN** the desktop transition enters Act II
- **THEN** its reward includes an item and excludes the later-act equipment
  award

#### Scenario: Resuming desktop randomness

- **WHEN** a character is saved and restored after a known desktop random
  sequence
- **THEN** its subsequent outcomes match uninterrupted desktop-profile
  execution and do not use Alea

### Requirement: Offline desktop protocol construction

The system SHALL construct desktop-6.4.4 progress, motto, and guild request
data without transport, using revision 8, evidenced desktop byte encoding and
field order, passkey validation, and exact transition snapshots. An empty
learned-spell list SHALL omit the specialty field rather than send `z=`.
Account authentication SHALL be represented separately and retained where
present; it SHALL NOT be assumed unnecessary because a passkey is available.
Unsupported encoding or authentication SHALL fail explicitly.

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

### Requirement: Desktop save export

The system SHALL encode a managed desktop-profile character as a `.pq` save
that the original Progress Quest 6.4.4 client can open and that imports back
through this system to a canonical state identical to the stored state. The
export SHALL include the character's stored passkey and online credentials so
the client can continue the character. The export SHALL also include a
Gyrognome metadata block with the state that no desktop component carries.
Encoding SHALL NOT require launching the original client, contacting a server,
or retaining a raw binary save.

#### Scenario: Round-tripping an exported desktop character

- **WHEN** a registered desktop character is exported and the file is imported
- **THEN** the imported canonical state equals the stored state

#### Scenario: Opening an export in the original client

- **WHEN** an exported `.pq` is opened in pq.exe 6.4.4
- **THEN** the client loads the character and shows the same traits, stats,
  equipment, inventory, spells, plots, quests, bars, and activity, without
  an error caused by the metadata block

### Requirement: Gyrognome metadata in desktop exports

A desktop export's metadata block SHALL carry the since-import counters
(tasks completed and elapsed milliseconds), the desktop random continuation,
the source provenance and adaptations, and the advancement provenance. It
SHALL also carry a version and a digest of the desktop component content it
accompanies. The block SHALL NOT contain credentials, and it SHALL be placed
where pq.exe 6.4.4 neither reads nor rewrites it.

On import, a valid block SHALL restore those values in place of fresh
registration values. Restored counters SHALL keep their "measured since
import" labeling, counted from the character's first import. A block whose
digest doesn't match its components, or whose version or content is
unsupported, SHALL fail the import rather than being ignored. Saves without a
block SHALL import with fresh registration values as before.

#### Scenario: Re-importing a Gyrognome export

- **WHEN** a desktop character that has completed tasks, has advanced locally,
  and has a random continuation is exported and imported into another
  Gyrognome store
- **THEN** its counters, random continuation, provenance, and local-only
  advancement equal the exported character's

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
