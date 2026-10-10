# Spec Delta

## MODIFIED Requirements

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

## ADDED Requirements

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
