## ADDED Requirements

### Requirement: Desktop save export

The system SHALL encode a managed desktop-profile character as a `.pq` save
that the original Progress Quest 6.4.4 client can open and that imports back
through this system to a canonical state identical to the stored state. The
export SHALL include the character's stored passkey and online credentials so
the client can continue the character. State that no desktop save carries
(import counters, provenance, unavailable history, random continuation) MAY be
omitted. Encoding SHALL NOT require launching the original client, contacting a
server, or retaining a raw binary save.

#### Scenario: Round-tripping an exported desktop character

- **WHEN** a registered desktop character is exported and the file is imported
- **THEN** the imported canonical state equals the stored state

#### Scenario: Opening an export in the original client

- **WHEN** an exported `.pq` is opened in pq.exe 6.4.4
- **THEN** the client loads the character and shows the same traits, stats,
  equipment, inventory, spells, plots, quests, bars, and activity
