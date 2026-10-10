# Spec Delta

## REMOVED Requirements

### Requirement: Browser ruleset fidelity

**Reason**: Split the bundled contract into independently editable requirements without removing any behavior.
**Migration**: Use Bundled browser ruleset fidelity, Canonical Specialty selection, and Canonical Prime Stat selection. All seven original scenarios are retained across those replacements.

## ADDED Requirements

### Requirement: Bundled browser ruleset fidelity

The system SHALL resolve task selection, combat, rewards, equipment, spells,
quests, plots, and level-ups from a bundled Progress Quest ruleset with recorded
source revision and provenance. The ruleset SHALL be selected explicitly and
remain stable for a simulation run.

#### Scenario: Resolving a simulation outcome

- **WHEN** a simulation step requires a browser rule-table value
- **THEN** the system uses the selected bundled ruleset rather than an ad hoc or externally fetched value

#### Scenario: Reproducing a level transition

- **WHEN** task completion reaches an experience-bar level boundary
- **THEN** the system applies browser-compatible level, attribute, spell, progress-bar, and random-state changes using the selected ruleset

### Requirement: Canonical Specialty selection

Before state is persisted or used for a leaderboard report, the canonical
`bestspell` value SHALL equal the browser-compatible name-and-rank display
value of the learned spell with the greatest product of its zero-based
collection index plus one and its Roman-numeral rank. Equal products SHALL
retain the earliest spell. If no spells are learned, `bestspell` SHALL be empty.

#### Scenario: Recording the strongest spell as Specialty

- **WHEN** a simulated state is persisted or produces a leaderboard report
- **THEN** the canonical `bestspell` value reflects the official ranked
  selection across all learned spells

#### Scenario: Resolving an equal-ranked Specialty

- **WHEN** two learned spells have equal ranked-selection products
- **THEN** the canonical `bestspell` value reflects the earlier spell in the
  learned-spell collection

#### Scenario: Reporting without learned spells

- **WHEN** a simulated state with no learned spells is persisted or produces a
  leaderboard report
- **THEN** the canonical `bestspell` value is empty

### Requirement: Canonical Prime Stat selection

Before state is persisted or used for a leaderboard report, the canonical
`beststat` value SHALL equal the browser-compatible label-and-integer display
value of the highest current prime stat among `STR`, `CON`, `DEX`, `INT`, `WIS`,
and `CHA`, and `Stats.best` SHALL identify that same stat. Equal current values
SHALL retain the earliest stat in that browser order.

#### Scenario: Recording the current Prime Stat

- **WHEN** a simulated state is persisted or produces a leaderboard report after its prime statistics have changed
- **THEN** `beststat`, `Stats.best`, and the leaderboard Prime Stat value identify the highest current prime stat and its current integer value

#### Scenario: Resolving an equal current Prime Stat

- **WHEN** multiple current prime stats share the highest integer value at a persistence or report boundary
- **THEN** the canonical Prime Stat reflects the earliest tied name in `STR`, `CON`, `DEX`, `INT`, `WIS`, `CHA` order
