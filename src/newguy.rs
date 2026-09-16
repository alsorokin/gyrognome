//! Offline-only creation of fresh local characters.

use std::{
    fs::File,
    io::Read,
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::Value;
use thiserror::Error;

use crate::{
    rng::AleaState,
    ruleset::Ruleset,
    state::{
        Activity, Attributes, Character, Equipment, InventoryEntry, Plot, Progress, ProgressBar,
        ProgressBarKind, Traits,
    },
};

#[derive(Debug, Error)]
pub enum NewGuyError {
    #[error("character name must contain at least one non-whitespace character")]
    EmptyName,
    #[error("character name exceeds the 30-character limit")]
    NameTooLong,
    #[error("character name contains a control character")]
    NameControlCharacter,
    #[error("race is not present in the bundled ruleset: {0}")]
    InvalidRace(String),
    #[error("class is not present in the bundled ruleset: {0}")]
    InvalidClass(String),
    #[error("could not obtain local randomness: {0}")]
    Randomness(#[from] std::io::Error),
}

/// Supplies local entropy to the otherwise side-effect-free creation logic.
pub trait RandomSource {
    fn next_u32(&mut self) -> Result<u32, NewGuyError>;
}

/// Reads fresh bytes from the operating system random source.
pub struct OsRandom(File);

impl OsRandom {
    pub fn open() -> Result<Self, NewGuyError> {
        Ok(Self(File::open("/dev/urandom")?))
    }
}

impl RandomSource for OsRandom {
    fn next_u32(&mut self) -> Result<u32, NewGuyError> {
        let mut bytes = [0; 4];
        self.0.read_exact(&mut bytes)?;
        Ok(u32::from_ne_bytes(bytes))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub name: String,
    pub race: String,
    pub class: String,
}

pub type BaseStats = [u8; 6];
pub const NAME_MAX_LENGTH: usize = 30;

/// Validates a character name using the desktop creator's character limit.
pub fn validate_name(name: &str) -> Result<(), NewGuyError> {
    if name.chars().count() > NAME_MAX_LENGTH {
        return Err(NewGuyError::NameTooLong);
    }
    if name.chars().any(char::is_control) {
        return Err(NewGuyError::NameControlCharacter);
    }
    if name.trim().is_empty() {
        return Err(NewGuyError::EmptyName);
    }
    Ok(())
}

pub fn random_selection(
    ruleset: &Ruleset,
    random: &mut impl RandomSource,
) -> Result<Selection, NewGuyError> {
    Ok(Selection {
        name: random_name(ruleset, random)?,
        race: trait_name(ruleset.races, random)?.to_owned(),
        class: trait_name(ruleset.klasses, random)?.to_owned(),
    })
}

pub fn random_name(
    ruleset: &Ruleset,
    random: &mut impl RandomSource,
) -> Result<String, NewGuyError> {
    let mut name = String::new();
    for part in ruleset.name_parts {
        name.push_str(part[index(part.len(), random)?]);
    }
    let mut characters = name.chars();
    Ok(match characters.next() {
        Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
        None => "Nameless".to_owned(),
    })
}

/// Generates complete, level-one canonical state without any online identity.
pub fn generate(
    selection: &Selection,
    ruleset: &Ruleset,
    random: &mut impl RandomSource,
) -> Result<Character, NewGuyError> {
    validate_name(&selection.name)?;
    let stats = roll_stats(random)?;
    generate_from_roll(selection, ruleset, stats, random)
}

/// Rolls the six pre-bonus prime statistics using the desktop New Guy formula.
pub fn roll_stats(random: &mut impl RandomSource) -> Result<BaseStats, NewGuyError> {
    let mut stats = [0; 6];
    for stat in &mut stats {
        *stat = 3;
        for _ in 0..3 {
            *stat += index(6, random)? as u8;
        }
    }
    Ok(stats)
}

/// Produces canonical state from a previously rolled set of base statistics.
pub fn generate_from_roll(
    selection: &Selection,
    ruleset: &Ruleset,
    base_stats: BaseStats,
    random: &mut impl RandomSource,
) -> Result<Character, NewGuyError> {
    validate_name(&selection.name)?;
    let dna = alea_state(random)?;
    let stat_seed = alea_state(random)?;
    let mut character = Character {
        document: Value::Null,
        traits: Traits {
            name: selection.name.clone(),
            race: selection.race.clone(),
            class: selection.class.clone(),
            level: 1,
        },
        dna,
        seed: stat_seed,
        birthday: "Offline character".to_owned(),
        birthstamp: unix_millis(),
        stats: Attributes {
            seed: stat_seed,
            strength: 0.0,
            constitution: 0.0,
            dexterity: 0.0,
            intelligence: 0.0,
            wisdom: 0.0,
            charisma: 0.0,
            hit_points_max: 0.0,
            mana_points_max: 0.0,
            best: String::new(),
        },
        beststat: String::new(),
        activity: Activity {
            task: "heading".to_owned(),
            tasks: 0,
            elapsed: 0,
            kill: "Heading to the killing fields...".to_owned(),
            questmonster: String::new(),
            questmonsterindex: 0,
        },
        bestequip: "Sharp Rock".to_owned(),
        equipment: Equipment {
            weapon: "Sharp Rock".to_owned(),
            shield: String::new(),
            helm: String::new(),
            hauberk: String::new(),
            brassairts: String::new(),
            vambraces: String::new(),
            gauntlets: String::new(),
            gambeson: String::new(),
            cuisses: String::new(),
            greaves: String::new(),
            sollerets: String::new(),
        },
        inventory: vec![InventoryEntry {
            name: "Gold".to_owned(),
            quantity: 0,
        }],
        spells: Vec::new(),
        plot: Plot {
            act: 0,
            bestplot: "Prologue".to_owned(),
        },
        quests: Vec::new(),
        progress: Progress {
            experience: bar(ProgressBarKind::Experience, 1_269),
            encumbrance: bar(ProgressBarKind::Encumbrance, 10),
            plot: bar(ProgressBarKind::Plot, 26),
            quest: bar(ProgressBarKind::Quest, 1),
            task: bar(ProgressBarKind::Task, 4_000),
        },
        queue: Vec::new(),
        date: "Offline character".to_owned(),
        stamp: unix_millis(),
        online: None,
        save_name: selection.name.clone(),
        bestspell: String::new(),
        bestquest: String::new(),
    };
    apply_stats(&mut character, selection, ruleset, base_stats)?;
    Ok(character)
}

/// Applies base statistics and selected trait bonuses to an existing draft.
pub fn apply_stats(
    character: &mut Character,
    selection: &Selection,
    ruleset: &Ruleset,
    base_stats: BaseStats,
) -> Result<(), NewGuyError> {
    validate_name(&selection.name)?;
    apply_stats_unchecked(character, selection, ruleset, base_stats)
}

pub(crate) fn apply_stats_unchecked(
    character: &mut Character,
    selection: &Selection,
    ruleset: &Ruleset,
    base_stats: BaseStats,
) -> Result<(), NewGuyError> {
    let race_bonus = bonuses(ruleset.races, &selection.race)
        .ok_or_else(|| NewGuyError::InvalidRace(selection.race.clone()))?;
    let class_bonus = bonuses(ruleset.klasses, &selection.class)
        .ok_or_else(|| NewGuyError::InvalidClass(selection.class.clone()))?;
    character.traits.name.clone_from(&selection.name);
    character.traits.race.clone_from(&selection.race);
    character.traits.class.clone_from(&selection.class);
    character.save_name.clone_from(&selection.name);
    character.stats.strength = base_stats[0] as f64;
    character.stats.constitution = base_stats[1] as f64;
    character.stats.dexterity = base_stats[2] as f64;
    character.stats.intelligence = base_stats[3] as f64;
    character.stats.wisdom = base_stats[4] as f64;
    character.stats.charisma = base_stats[5] as f64;
    character.stats.hit_points_max = 1.0;
    character.stats.mana_points_max = 1.0;
    for bonus in race_bonus.into_iter().chain(class_bonus) {
        add_stat(&mut *character, bonus);
    }
    character.stats.hit_points_max += (character.stats.constitution / 3.0).floor();
    character.stats.mana_points_max += (character.stats.intelligence / 3.0).floor();
    let (best, value) = [
        ("STR", character.stats.strength),
        ("CON", character.stats.constitution),
        ("DEX", character.stats.dexterity),
        ("INT", character.stats.intelligence),
        ("WIS", character.stats.wisdom),
        ("CHA", character.stats.charisma),
    ]
    .into_iter()
    .max_by(|left, right| left.1.total_cmp(&right.1))
    .expect("prime stats are nonempty");
    character.stats.best = best.to_owned();
    character.beststat = format!("{best} {}", value as u64);
    Ok(())
}

/// Replaces only the random starting statistics for the supplied traits.
pub fn reroll(
    selection: &Selection,
    ruleset: &Ruleset,
    random: &mut impl RandomSource,
) -> Result<Character, NewGuyError> {
    generate(selection, ruleset, random)
}

pub fn generate_local(selection: &Selection, ruleset: &Ruleset) -> Result<Character, NewGuyError> {
    generate(selection, ruleset, &mut OsRandom::open()?)
}

fn trait_name<'a>(
    values: &'a [&'a str],
    random: &mut impl RandomSource,
) -> Result<&'a str, NewGuyError> {
    Ok(values[index(values.len(), random)?]
        .split('|')
        .next()
        .unwrap_or_default())
}

fn bonuses<'a>(values: &'a [&'a str], name: &str) -> Option<Vec<&'a str>> {
    values.iter().find_map(|value| {
        let (candidate, bonuses) = value.split_once('|')?;
        (candidate == name).then(|| bonuses.split(',').collect())
    })
}

fn index(length: usize, random: &mut impl RandomSource) -> Result<usize, NewGuyError> {
    Ok(random.next_u32()? as usize % length)
}

fn alea_state(random: &mut impl RandomSource) -> Result<AleaState, NewGuyError> {
    Ok(AleaState([
        random.next_u32()? as f64 / (u32::MAX as f64 + 1.0),
        random.next_u32()? as f64 / (u32::MAX as f64 + 1.0),
        random.next_u32()? as f64 / (u32::MAX as f64 + 1.0),
        1.0,
    ]))
}

fn add_stat(character: &mut Character, stat: &str) {
    match stat {
        "STR" => character.stats.strength += 1.0,
        "CON" => character.stats.constitution += 1.0,
        "DEX" => character.stats.dexterity += 1.0,
        "INT" => character.stats.intelligence += 1.0,
        "WIS" => character.stats.wisdom += 1.0,
        "CHA" => character.stats.charisma += 1.0,
        "HP Max" => character.stats.hit_points_max += 1.0,
        "MP Max" => character.stats.mana_points_max += 1.0,
        _ => unreachable!("bundled ruleset contains only canonical stat names"),
    }
}

fn bar(kind: ProgressBarKind, max: u64) -> ProgressBar {
    let mut bar = ProgressBar {
        position: 0.0,
        max: 0,
        percent: 0,
        remaining: 0,
        time: String::new(),
        hint: String::new(),
    };
    bar.reset(kind, max, 0.0);
    bar
}

fn unix_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Numbers(u32);
    impl RandomSource for Numbers {
        fn next_u32(&mut self) -> Result<u32, NewGuyError> {
            let next = self.0;
            self.0 = self.0.wrapping_add(1);
            Ok(next)
        }
    }

    #[test]
    fn generates_offline_canonical_level_one_state() {
        let selection = Selection {
            name: "Offline Hero".to_owned(),
            race: "Gyrognome".to_owned(),
            class: "Robot Monk".to_owned(),
        };
        let character = generate(&selection, &crate::ruleset::BUNDLED, &mut Numbers(1)).unwrap();
        assert_eq!(character.traits.level, 1);
        assert!((3.0..=18.0).contains(&character.stats.dexterity));
        assert!((3.0..=18.0).contains(&character.stats.strength));
        assert!(character.online.is_none());
        assert!(character.document.is_null());
        assert_eq!(character.inventory[0].name, "Gold");
        let original = serde_json::to_string(&character).unwrap();
        assert!(crate::simulation::advance(&character, &crate::ruleset::BUNDLED, 4_000).is_ok());
        assert_eq!(serde_json::to_string(&character).unwrap(), original);
    }

    #[test]
    fn rolls_each_prime_stat_as_three_plus_three_six_sided_dice() {
        assert_eq!(roll_stats(&mut Numbers(0)).unwrap(), [6, 15, 6, 15, 6, 15]);
    }

    #[test]
    fn rejects_unknown_traits() {
        let mut random = Numbers(1);
        let selection = Selection {
            name: "Offline Hero".to_owned(),
            race: "Unknown".to_owned(),
            class: "Robot Monk".to_owned(),
        };
        assert!(matches!(
            generate(&selection, &crate::ruleset::BUNDLED, &mut random),
            Err(NewGuyError::InvalidRace(_))
        ));
    }

    #[test]
    fn validates_name_length_content_and_unicode() {
        for name in [
            "",
            " \t",
            "a".repeat(NAME_MAX_LENGTH + 1).as_str(),
            "bad\nname",
        ] {
            assert!(validate_name(name).is_err(), "{name:?} should be rejected");
        }
        assert!(validate_name("Ari the Gyrognome").is_ok());
        assert!(validate_name("Жасмин the Gyrognome").is_ok());
        assert!(validate_name(&"é".repeat(NAME_MAX_LENGTH)).is_ok());
    }
}
