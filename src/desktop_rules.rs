use std::{collections::BTreeMap, sync::OnceLock};

use serde::Deserialize;
use thiserror::Error;

use crate::compatibility::DesktopRandomState;

pub const SOURCE_TAG: &str = "v6.4.4";
pub const SOURCE_COMMIT: &str = "95f5d66f97a3697a7446fba951ab04156c752274";
pub const MAIN_PAS_SHA256: &str =
    "fc61edcd1723e4d69724699d1fc60e69f02d0c0530a3c0d90eff446303829f24";
pub const CONFIG_DFM_SHA256: &str =
    "ac42dfaf747713e5f91080fec698b697c69000f75405ed1d787e70ea71619600";
pub const RANDOM_ALGORITHM: &str = "delphi-6-randseed-lcg-source-derived/v1";
pub const RANDOM_MULTIPLIER: u32 = 0x0808_8405;
pub const EXACT_DELPHI_RUNTIME_EQUIVALENCE: &str = "unverified";
pub const CLASSIC_ONLINE_ELIGIBLE: bool = false;

const BUNDLED_RULES: &str = include_str!("desktop_rules.json");
const MAX_REFERENCE_LEVEL: u32 = 1_000;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopRules {
    pub format: String,
    pub source: DesktopRuleSource,
    pub identity: DesktopRuleIdentity,
    pub runtime_equivalence: DesktopRuntimeEquivalence,
    pub tables: DesktopRuleTables,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopRuleSource {
    pub repository: String,
    pub tag: String,
    pub commit: String,
    pub main_pas_sha256: String,
    pub config_dfm_sha256: String,
    pub captured_on: String,
}

#[derive(Debug, Deserialize)]
pub struct DesktopRuleIdentity {
    pub algorithm: String,
    pub tables: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopRuntimeEquivalence {
    pub status: String,
    pub official_executable_corroboration: String,
    pub local_continuation_only: bool,
    pub classic_online_eligible: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopRuleTables {
    pub traits: Vec<String>,
    pub prime_stats: Vec<String>,
    pub stats: Vec<String>,
    pub equips: Vec<String>,
    pub spells: Vec<String>,
    pub offense_attrib: Vec<String>,
    pub defense_attrib: Vec<String>,
    pub shields: Vec<String>,
    pub armors: Vec<String>,
    pub weapons: Vec<String>,
    pub specials: Vec<String>,
    pub item_attrib: Vec<String>,
    pub item_ofs: Vec<String>,
    pub boring_items: Vec<String>,
    pub monsters: Vec<String>,
    pub mon_mods: Vec<String>,
    pub offense_bad: Vec<String>,
    pub defense_bad: Vec<String>,
    pub races: Vec<String>,
    pub klasses: Vec<String>,
    pub titles: Vec<String>,
    pub impressive_titles: Vec<String>,
}

impl DesktopRuleTables {
    pub fn ordered(&self) -> [(&'static str, &[String]); 22] {
        [
            ("traits", &self.traits),
            ("primeStats", &self.prime_stats),
            ("stats", &self.stats),
            ("equips", &self.equips),
            ("spells", &self.spells),
            ("offenseAttrib", &self.offense_attrib),
            ("defenseAttrib", &self.defense_attrib),
            ("shields", &self.shields),
            ("armors", &self.armors),
            ("weapons", &self.weapons),
            ("specials", &self.specials),
            ("itemAttrib", &self.item_attrib),
            ("itemOfs", &self.item_ofs),
            ("boringItems", &self.boring_items),
            ("monsters", &self.monsters),
            ("monMods", &self.mon_mods),
            ("offenseBad", &self.offense_bad),
            ("defenseBad", &self.defense_bad),
            ("races", &self.races),
            ("klasses", &self.klasses),
            ("titles", &self.titles),
            ("impressiveTitles", &self.impressive_titles),
        ]
    }
}

pub fn bundled() -> &'static DesktopRules {
    static RULES: OnceLock<DesktopRules> = OnceLock::new();
    RULES.get_or_init(|| {
        serde_json::from_str(BUNDLED_RULES).expect("bundled desktop rules must be valid")
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DelphiRandom {
    state: u32,
}

impl DelphiRandom {
    pub const fn from_state(state: DesktopRandomState) -> Self {
        Self { state: state.0 }
    }

    pub const fn state(self) -> DesktopRandomState {
        DesktopRandomState(self.state)
    }

    pub fn bounded(&mut self, upper_bound: u32) -> Result<u32, DesktopNumericError> {
        if upper_bound == 0 {
            return Err(DesktopNumericError::InvalidRandomBound);
        }
        self.state = self.state.wrapping_mul(RANDOM_MULTIPLIER).wrapping_add(1);
        Ok(((u64::from(self.state) * u64::from(upper_bound)) >> 32) as u32)
    }

    pub fn random64(&mut self) -> Result<u64, DesktopNumericError> {
        let high = u64::from(self.bounded(0x3fff_ffff)?);
        let low = u64::from(self.bounded(u32::MAX)?);
        Ok((high << 32) | low)
    }

    pub fn random64_below(&mut self, upper_bound: u64) -> Result<u64, DesktopNumericError> {
        if upper_bound == 0 || upper_bound > i64::MAX as u64 {
            return Err(DesktopNumericError::InvalidRandom64Bound);
        }
        Ok(self.random64()? % upper_bound)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DesktopNumericError {
    #[error("desktop bounded random upper bound is outside the supported range")]
    InvalidRandomBound,
    #[error("desktop 64-bit random upper bound is outside the supported range")]
    InvalidRandom64Bound,
    #[error("desktop integer division is undefined")]
    InvalidDivision,
    #[error("desktop numeric input is outside the supported range")]
    OutOfRange,
    #[error("desktop weighted-stat input is invalid")]
    InvalidWeights,
}

pub fn pascal_div(dividend: i64, divisor: i64) -> Result<i64, DesktopNumericError> {
    dividend
        .checked_div(divisor)
        .ok_or(DesktopNumericError::InvalidDivision)
}

pub fn round_half_even(value: f64) -> Result<i64, DesktopNumericError> {
    if !value.is_finite() || value < i64::MIN as f64 || value > i64::MAX as f64 {
        return Err(DesktopNumericError::OutOfRange);
    }
    Ok(value.round_ties_even() as i64)
}

pub fn level_up_time(level: u32) -> Result<u64, DesktopNumericError> {
    if level > MAX_REFERENCE_LEVEL {
        return Err(DesktopNumericError::OutOfRange);
    }
    let seconds = (20.0 + 1.15_f64.powi(level as i32)) * 60.0;
    u64::try_from(round_half_even(seconds)?).map_err(|_| DesktopNumericError::OutOfRange)
}

pub fn weighted_stat_index(
    stats: &[u64],
    random: &mut DelphiRandom,
) -> Result<usize, DesktopNumericError> {
    if stats.len() < 6 || stats.len() > u32::MAX as usize {
        return Err(DesktopNumericError::InvalidWeights);
    }
    let mut weights = [0_u64; 6];
    let mut total = 0_u64;
    for (index, stat) in stats.iter().take(6).copied().enumerate() {
        let weight = stat
            .checked_mul(stat)
            .ok_or(DesktopNumericError::InvalidWeights)?;
        total = total
            .checked_add(weight)
            .filter(|value| *value <= i64::MAX as u64)
            .ok_or(DesktopNumericError::InvalidWeights)?;
        weights[index] = weight;
    }
    if total == 0 {
        return Err(DesktopNumericError::InvalidWeights);
    }

    if random.bounded(2)? == 0 {
        return Ok(random.bounded(stats.len() as u32)? as usize);
    }

    let mut target = random.random64_below(total)?;
    for (index, weight) in weights.into_iter().enumerate() {
        if target < weight {
            return Ok(index);
        }
        target -= weight;
    }
    Err(DesktopNumericError::InvalidWeights)
}

#[cfg(test)]
mod tests {
    use ring::digest::{SHA256, digest};

    use super::*;

    fn sha256_lines(entries: &[String]) -> String {
        digest(&SHA256, entries.join("\n").as_bytes())
            .as_ref()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    #[test]
    fn bundled_tables_match_pinned_source_identity_and_order() {
        let rules = bundled();
        assert_eq!(rules.format, "gyrognome-desktop-rules/v1");
        assert_eq!(rules.source.tag, SOURCE_TAG);
        assert_eq!(rules.source.commit, SOURCE_COMMIT);
        assert_eq!(rules.source.main_pas_sha256, MAIN_PAS_SHA256);
        assert_eq!(rules.source.config_dfm_sha256, CONFIG_DFM_SHA256);
        assert_eq!(
            rules.identity.algorithm,
            "sha256(utf8(entries joined by LF))"
        );
        for (name, entries) in rules.tables.ordered() {
            assert_eq!(
                rules.identity.tables.get(name).map(String::as_str),
                Some(sha256_lines(entries).as_str()),
                "{name}"
            );
        }
        assert_eq!(rules.tables.spells.len(), 46);
        assert_eq!(
            &rules.tables.spells[6..9],
            ["Gyp", "Shoelaces", "Innoculate"]
        );
        assert_eq!(rules.tables.monsters.len(), 231);
        assert_eq!(rules.tables.monsters[0], "Anhkheg|6|chitin");
        assert_eq!(
            rules.tables.monsters.last().map(String::as_str),
            Some("Hogbird|3|curl")
        );
        assert_eq!(rules.tables.mon_mods[0], "-4 fœtal *");
    }

    #[test]
    fn desktop_rules_are_independent_from_browser_tables() {
        let desktop = bundled();
        assert_eq!(crate::ruleset::SPELLS.len(), 47);
        assert_eq!(desktop.tables.spells.len(), 46);
        assert_eq!(crate::ruleset::SPELLS[6], "Shoelaces");
        assert_eq!(desktop.tables.spells[6], "Gyp");
        assert_eq!(crate::ruleset::MONSTERS.len(), 232);
        assert_eq!(desktop.tables.monsters.len(), 231);
    }

    #[test]
    fn matches_source_derived_random_vectors() {
        let mut random = DelphiRandom::from_state(DesktopRandomState(0x1234_5678));
        assert_eq!(random.bounded(2).unwrap(), 1);
        assert_eq!(random.bounded(10).unwrap(), 4);
        assert_eq!(random.bounded(u32::MAX).unwrap(), 3_651_437_750);
        assert_eq!(random.random64().unwrap(), 4_366_025_317_568_549_348);
        assert_eq!(random.random64_below(1_000_003).unwrap(), 401_907);
        assert_eq!(random.state(), DesktopRandomState(2_635_074_403));
    }

    #[test]
    fn matches_weighted_numeric_and_level_vectors() {
        let mut random = DelphiRandom::from_state(DesktopRandomState(0x0bad_f00d));
        assert_eq!(
            weighted_stat_index(&[12, 11, 10, 9, 8, 7, 1_000], &mut random).unwrap(),
            2
        );
        assert_eq!(random.state(), DesktopRandomState(4_265_254_520));
        assert_eq!(pascal_div(5_114, 1_000).unwrap(), 5);
        assert_eq!(pascal_div(-5_114, 1_000).unwrap(), -5);
        assert_eq!(round_half_even(1_200.5).unwrap(), 1_200);
        assert_eq!(round_half_even(1_201.5).unwrap(), 1_202);
        assert_eq!(round_half_even(-1_200.5).unwrap(), -1_200);
        assert_eq!(round_half_even(-1_201.5).unwrap(), -1_202);
        assert_eq!(
            [1, 2, 10, 20, 30, 40].map(|level| level_up_time(level).unwrap()),
            [1_269, 1_279, 1_443, 2_182, 5_173, 17_272]
        );
    }

    #[test]
    fn rejects_unsupported_numeric_ranges_without_panicking() {
        let mut random = DelphiRandom::from_state(DesktopRandomState(1));
        assert_eq!(
            random.bounded(0),
            Err(DesktopNumericError::InvalidRandomBound)
        );
        assert_eq!(
            random.random64_below(0),
            Err(DesktopNumericError::InvalidRandom64Bound)
        );
        assert_eq!(
            random.random64_below(i64::MAX as u64 + 1),
            Err(DesktopNumericError::InvalidRandom64Bound)
        );
        assert_eq!(pascal_div(1, 0), Err(DesktopNumericError::InvalidDivision));
        assert_eq!(
            pascal_div(i64::MIN, -1),
            Err(DesktopNumericError::InvalidDivision)
        );
        assert_eq!(
            round_half_even(f64::NAN),
            Err(DesktopNumericError::OutOfRange)
        );
        assert_eq!(
            level_up_time(MAX_REFERENCE_LEVEL + 1),
            Err(DesktopNumericError::OutOfRange)
        );
        assert_eq!(
            weighted_stat_index(&[0; 6], &mut random),
            Err(DesktopNumericError::InvalidWeights)
        );
    }

    #[test]
    fn discloses_source_derived_runtime_limitations() {
        let limitations = &bundled().runtime_equivalence;
        assert_eq!(EXACT_DELPHI_RUNTIME_EQUIVALENCE, "unverified");
        assert_eq!(limitations.status, EXACT_DELPHI_RUNTIME_EQUIVALENCE);
        assert_eq!(limitations.official_executable_corroboration, "not-run");
        assert!(limitations.local_continuation_only);
        assert!(!limitations.classic_online_eligible);
        assert!(!CLASSIC_ONLINE_ELIGIBLE);
    }
}
