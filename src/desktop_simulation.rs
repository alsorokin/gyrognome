use thiserror::Error;

use crate::{
    compatibility::DesktopCanonicalState,
    desktop_rules::{DelphiRandom, bundled, level_up_time, weighted_stat_index},
    desktop_save::{
        DesktopQuestMarker, DesktopQueueCommand, DesktopQueueKind, DesktopValidatedBar,
        DesktopValidatedRow,
    },
};

const MAX_DEQUEUE_TRANSITIONS: usize = 1_024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesktopProgressObservation {
    pub credited_seconds: u64,
    pub combat: bool,
    pub level_up_dispatched: bool,
    pub quest_completion_dispatched: bool,
    pub interplot_dispatched: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DesktopProgressionError {
    #[error("desktop numeric state is invalid: {0}")]
    InvalidNumber(&'static str),
    #[error("desktop task state is invalid")]
    InvalidTask,
    #[error("desktop queue transition is invalid")]
    InvalidQueue,
    #[error("desktop progression did not select a new task")]
    MissingNextTask,
    #[error("desktop progression exceeded the supported transition bound")]
    TransitionLimit,
    #[error("desktop reward transition failed: {0}")]
    Reward(&'static str),
}

pub trait DesktopProgressionHooks {
    fn level_up(
        &mut self,
        state: &mut DesktopCanonicalState,
        random: &mut DelphiRandom,
    ) -> Result<(), DesktopProgressionError>;

    fn quest_reward(
        &mut self,
        state: &mut DesktopCanonicalState,
        random: &mut DelphiRandom,
    ) -> Result<(), DesktopProgressionError>;

    fn complete_act(
        &mut self,
        state: &mut DesktopCanonicalState,
        random: &mut DelphiRandom,
    ) -> Result<(), DesktopProgressionError>;

    fn win_item(
        &mut self,
        state: &mut DesktopCanonicalState,
        random: &mut DelphiRandom,
    ) -> Result<(), DesktopProgressionError>;

    fn win_equipment(
        &mut self,
        state: &mut DesktopCanonicalState,
        random: &mut DelphiRandom,
    ) -> Result<(), DesktopProgressionError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopReportTrigger {
    Level,
    Act,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopReportSnapshot {
    pub trigger: DesktopReportTrigger,
    pub state: DesktopCanonicalState,
}

#[derive(Debug, Default)]
pub struct SourceDerivedDesktopHooks {
    capture_reports: bool,
    reports: Vec<DesktopReportSnapshot>,
}

impl SourceDerivedDesktopHooks {
    pub fn traced() -> Self {
        Self {
            capture_reports: true,
            reports: Vec::new(),
        }
    }

    pub fn untraced() -> Self {
        Self::default()
    }

    pub fn reports(&self) -> &[DesktopReportSnapshot] {
        &self.reports
    }

    pub fn into_reports(self) -> Vec<DesktopReportSnapshot> {
        self.reports
    }

    fn capture(&mut self, trigger: DesktopReportTrigger, state: &DesktopCanonicalState) {
        if self.capture_reports {
            self.reports.push(DesktopReportSnapshot {
                trigger,
                state: state.clone(),
            });
        }
    }
}

impl DesktopProgressionHooks for SourceDerivedDesktopHooks {
    fn level_up(
        &mut self,
        state: &mut DesktopCanonicalState,
        random: &mut DelphiRandom,
    ) -> Result<(), DesktopProgressionError> {
        add_row_value(&mut state.traits, "Level", 1)?;
        let constitution = named_row_value(&state.stats, "CON")?;
        let hp = constitution / 3 + 1 + u64::from(bounded(random, 4)?);
        add_row_value(
            &mut state.stats,
            "HP Max",
            i64::try_from(hp).map_err(|_| DesktopProgressionError::InvalidNumber("HP Max"))?,
        )?;
        let intelligence = named_row_value(&state.stats, "INT")?;
        let mp = intelligence / 3 + 1 + u64::from(bounded(random, 4)?);
        add_row_value(
            &mut state.stats,
            "MP Max",
            i64::try_from(mp).map_err(|_| DesktopProgressionError::InvalidNumber("MP Max"))?,
        )?;
        win_stat(state, random)?;
        win_stat(state, random)?;
        win_spell(state, random)?;
        let level = u32::try_from(character_level(state)?)
            .map_err(|_| DesktopProgressionError::InvalidNumber("level"))?;
        state.bars.experience = DesktopValidatedBar {
            position: 0,
            maximum: level_up_time(level)
                .map_err(|_| DesktopProgressionError::InvalidNumber("experience maximum"))?,
        };
        self.capture(DesktopReportTrigger::Level, state);
        Ok(())
    }

    fn quest_reward(
        &mut self,
        state: &mut DesktopCanonicalState,
        random: &mut DelphiRandom,
    ) -> Result<(), DesktopProgressionError> {
        match bounded(random, 4)? {
            0 => win_spell(state, random),
            1 => win_equipment(state, random),
            2 => win_stat(state, random),
            3 => win_item(state, random),
            _ => unreachable!(),
        }
    }

    fn complete_act(
        &mut self,
        state: &mut DesktopCanonicalState,
        random: &mut DelphiRandom,
    ) -> Result<(), DesktopProgressionError> {
        let current = state
            .plots
            .last_mut()
            .ok_or(DesktopProgressionError::InvalidQueue)?;
        current.header[1] = 1;
        let act_number = state.plots.len();
        let act_number_u64 = u64::try_from(act_number)
            .map_err(|_| DesktopProgressionError::InvalidNumber("act number"))?;
        state.bars.plot = DesktopValidatedBar {
            position: 0,
            maximum: 60_u64
                .checked_mul(60)
                .and_then(|value| {
                    5_u64
                        .checked_mul(act_number_u64)
                        .and_then(|acts| acts.checked_add(1))
                        .and_then(|acts| value.checked_mul(acts))
                })
                .ok_or(DesktopProgressionError::InvalidNumber("plot maximum"))?,
        };
        state.plots.push(DesktopValidatedRow {
            header: [-1, 0, -1, 0, 0],
            caption: format!("Act {}", desktop_roman(act_number as u64)),
            subitems: Vec::new(),
        });
        if state.plots.len() > 2 {
            win_item(state, random)?;
        }
        if state.plots.len() > 3 {
            win_equipment(state, random)?;
        }
        self.capture(DesktopReportTrigger::Act, state);
        Ok(())
    }

    fn win_item(
        &mut self,
        state: &mut DesktopCanonicalState,
        random: &mut DelphiRandom,
    ) -> Result<(), DesktopProgressionError> {
        win_item(state, random)
    }

    fn win_equipment(
        &mut self,
        state: &mut DesktopCanonicalState,
        random: &mut DelphiRandom,
    ) -> Result<(), DesktopProgressionError> {
        win_equipment(state, random)
    }
}

pub fn complete_task(
    state: &mut DesktopCanonicalState,
    random: &mut DelphiRandom,
    hooks: &mut impl DesktopProgressionHooks,
) -> Result<DesktopProgressObservation, DesktopProgressionError> {
    validate_full_task(state)?;
    resolve_legacy_quest_marker(state)?;

    let combat = state.current_task.starts_with("kill|");
    let credited_seconds = state.bars.task.maximum / 1_000;
    let mut observation = DesktopProgressObservation {
        credited_seconds,
        combat,
        level_up_dispatched: false,
        quest_completion_dispatched: false,
        interplot_dispatched: false,
    };

    if combat {
        if state.bars.experience.position >= state.bars.experience.maximum {
            hooks.level_up(state, random)?;
            observation.level_up_dispatched = true;
        } else {
            credit_bar(&mut state.bars.experience, credited_seconds);
        }

        if state.plots.len() > 1 {
            if state.bars.quest.position >= state.bars.quest.maximum {
                complete_quest(state, random, hooks)?;
                observation.quest_completion_dispatched = true;
            } else if !state.quests.is_empty() {
                credit_bar(&mut state.bars.quest, credited_seconds);
            }
        }
    }

    if state.bars.plot.position >= state.bars.plot.maximum && combat {
        interplot_cinematic(state, random)?;
        observation.interplot_dispatched = true;
    } else if state.current_task != "load" {
        credit_bar(&mut state.bars.plot, credited_seconds);
    }

    dequeue(state, random, hooks)?;
    Ok(observation)
}

pub fn resolve_legacy_quest_marker(
    state: &mut DesktopCanonicalState,
) -> Result<(), DesktopProgressionError> {
    let DesktopQuestMarker::LegacyPlaceholder { index } = state.quest else {
        return Ok(());
    };
    let marker = bundled()
        .tables
        .monsters
        .get(index)
        .cloned()
        .ok_or(DesktopProgressionError::InvalidNumber("quest index"))?;
    state.quest = DesktopQuestMarker::Value { marker, index };
    Ok(())
}

fn validate_full_task(state: &DesktopCanonicalState) -> Result<(), DesktopProgressionError> {
    let task = &state.bars.task;
    if task.maximum == 0 || task.position != task.maximum {
        return Err(DesktopProgressionError::InvalidTask);
    }
    Ok(())
}

fn credit_bar(bar: &mut DesktopValidatedBar, amount: u64) {
    bar.position = bar.position.saturating_add(amount).min(bar.maximum);
}

fn dequeue(
    state: &mut DesktopCanonicalState,
    random: &mut DelphiRandom,
    hooks: &mut impl DesktopProgressionHooks,
) -> Result<(), DesktopProgressionError> {
    for _ in 0..MAX_DEQUEUE_TRANSITIONS {
        if state.bars.task.position < state.bars.task.maximum {
            return Ok(());
        }

        apply_completed_task_reward(state, random, hooks)?;
        if continue_market_sale(state, random)? {
            return Ok(());
        }

        let old_task = std::mem::take(&mut state.current_task);
        if let Some(command) = state.queue.first().cloned() {
            start_queued_task(state, random, hooks, command)?;
            state.queue.remove(0);
        } else if state.bars.encumbrance.position >= state.bars.encumbrance.maximum {
            start_task(state, "market", "Heading to market to sell loot", 4_000)?;
        } else if !old_task.starts_with("kill|") && old_task != "heading" {
            if inventory_value(state, "Gold")? > equipment_price(state)? {
                start_task(
                    state,
                    "buying",
                    "Negotiating purchase of better equipment",
                    5_000,
                )?;
            } else {
                start_task(state, "heading", "Heading to the killing fields", 4_000)?;
            }
        } else {
            select_combat_task(state, random)?;
        }
    }
    Err(DesktopProgressionError::TransitionLimit)
}

fn apply_completed_task_reward(
    state: &mut DesktopCanonicalState,
    random: &mut DelphiRandom,
    hooks: &mut impl DesktopProgressionHooks,
) -> Result<(), DesktopProgressionError> {
    if let Some((monster, drop)) = parse_kill_task(&state.current_task)? {
        if drop == "*" {
            hooks.win_item(state, random)?;
        } else if !drop.is_empty() {
            let name = format!("{} {}", monster, proper_case(drop)).to_lowercase();
            add_inventory(state, &name, 1)?;
        }
    } else if state.current_task == "buying" {
        add_inventory(state, "Gold", -equipment_price(state)?)?;
        hooks.win_equipment(state, random)?;
    }
    Ok(())
}

fn continue_market_sale(
    state: &mut DesktopCanonicalState,
    random: &mut DelphiRandom,
) -> Result<bool, DesktopProgressionError> {
    if state.current_task != "market" && state.current_task != "sell" {
        return Ok(false);
    }

    if state.current_task == "sell" {
        let item = state
            .inventory
            .get(1)
            .cloned()
            .ok_or(DesktopProgressionError::InvalidTask)?;
        let quantity = row_value(&item, "inventory quantity")?;
        let level = character_level(state)?;
        let mut price = quantity
            .checked_mul(level)
            .ok_or(DesktopProgressionError::InvalidNumber("sale price"))?;
        if item.caption.contains(" of ") {
            let first = 1_u64
                .checked_add(random_low(10, random)?)
                .ok_or(DesktopProgressionError::InvalidNumber("sale price"))?;
            let second = 1_u64
                .checked_add(random_low(level, random)?)
                .ok_or(DesktopProgressionError::InvalidNumber("sale price"))?;
            price = price
                .checked_mul(first)
                .and_then(|value| value.checked_mul(second))
                .ok_or(DesktopProgressionError::InvalidNumber("sale price"))?;
        }
        state.inventory.remove(1);
        add_inventory(
            state,
            "Gold",
            i64::try_from(price)
                .map_err(|_| DesktopProgressionError::InvalidNumber("sale price"))?,
        )?;
        refresh_encumbrance(state)?;
    }

    if let Some(item) = state.inventory.get(1) {
        let quantity = row_value(item, "inventory quantity")?;
        start_task(
            state,
            "sell",
            &format!("Selling {}", indefinite(&item.caption, quantity)),
            1_000,
        )?;
        return Ok(true);
    }
    Ok(false)
}

fn start_queued_task(
    state: &mut DesktopCanonicalState,
    random: &mut DelphiRandom,
    hooks: &mut impl DesktopProgressionHooks,
    command: DesktopQueueCommand,
) -> Result<(), DesktopProgressionError> {
    let caption = match command.kind {
        DesktopQueueKind::Task => command.caption,
        DesktopQueueKind::Plot => {
            hooks.complete_act(state, random)?;
            let plot = state
                .plots
                .last()
                .ok_or(DesktopProgressionError::InvalidQueue)?;
            format!("Loading {}", plot.caption)
        }
    };
    let duration = u64::from(command.duration_seconds)
        .checked_mul(1_000)
        .ok_or(DesktopProgressionError::InvalidNumber("queue duration"))?;
    start_task(state, "", &caption, duration)
}

fn start_task(
    state: &mut DesktopCanonicalState,
    internal_task: &str,
    caption: &str,
    duration_milliseconds: u64,
) -> Result<(), DesktopProgressionError> {
    if duration_milliseconds == 0 {
        return Err(DesktopProgressionError::InvalidTask);
    }
    state.current_task = internal_task.to_owned();
    state.activity = format!("{caption}...");
    state.bars.task = DesktopValidatedBar {
        position: 0,
        maximum: duration_milliseconds,
    };
    Ok(())
}

fn parse_kill_task(task: &str) -> Result<Option<(&str, &str)>, DesktopProgressionError> {
    if !task.starts_with("kill|") {
        return Ok(None);
    }
    let mut parts = task.split('|');
    if parts.next() != Some("kill") {
        return Err(DesktopProgressionError::InvalidTask);
    }
    let monster = parts.next().ok_or(DesktopProgressionError::InvalidTask)?;
    let _level = parts.next().ok_or(DesktopProgressionError::InvalidTask)?;
    let drop = parts.next().ok_or(DesktopProgressionError::InvalidTask)?;
    if monster.is_empty() || parts.next().is_some() {
        return Err(DesktopProgressionError::InvalidTask);
    }
    Ok(Some((monster, drop)))
}

fn character_level(state: &DesktopCanonicalState) -> Result<u64, DesktopProgressionError> {
    row_value(
        state
            .traits
            .get(3)
            .ok_or(DesktopProgressionError::InvalidNumber("level"))?,
        "level",
    )
}

fn equipment_price(state: &DesktopCanonicalState) -> Result<i64, DesktopProgressionError> {
    let level = i64::try_from(character_level(state)?)
        .map_err(|_| DesktopProgressionError::InvalidNumber("level"))?;
    5_i64
        .checked_mul(level)
        .and_then(|value| value.checked_mul(level))
        .and_then(|value| {
            10_i64
                .checked_mul(level)
                .and_then(|part| value.checked_add(part))
        })
        .and_then(|value| value.checked_add(20))
        .ok_or(DesktopProgressionError::InvalidNumber("equipment price"))
}

fn inventory_value(
    state: &DesktopCanonicalState,
    caption: &str,
) -> Result<i64, DesktopProgressionError> {
    state
        .inventory
        .iter()
        .find(|row| row.caption == caption)
        .map(|row| {
            row.subitems
                .first()
                .ok_or(DesktopProgressionError::InvalidNumber("inventory quantity"))?
                .parse::<i64>()
                .map_err(|_| DesktopProgressionError::InvalidNumber("inventory quantity"))
        })
        .transpose()
        .map(Option::unwrap_or_default)
}

fn row_value(
    row: &DesktopValidatedRow,
    field: &'static str,
) -> Result<u64, DesktopProgressionError> {
    row.subitems
        .first()
        .ok_or(DesktopProgressionError::InvalidNumber(field))?
        .parse()
        .map_err(|_| DesktopProgressionError::InvalidNumber(field))
}

fn add_inventory(
    state: &mut DesktopCanonicalState,
    caption: &str,
    change: i64,
) -> Result<(), DesktopProgressionError> {
    let index = state
        .inventory
        .iter()
        .position(|row| row.caption == caption);
    let current = index
        .map(|index| inventory_value(state, &state.inventory[index].caption))
        .transpose()?
        .unwrap_or_default();
    let next = current
        .checked_add(change)
        .ok_or(DesktopProgressionError::InvalidNumber("inventory quantity"))?;
    if next < 0 {
        return Err(DesktopProgressionError::InvalidNumber("inventory quantity"));
    }
    match index {
        Some(index) => state.inventory[index].subitems[0] = next.to_string(),
        None => state.inventory.push(DesktopValidatedRow {
            header: [0, -1, -1, 0, 1],
            caption: caption.to_owned(),
            subitems: vec![next.to_string()],
        }),
    }
    refresh_encumbrance(state)
}

fn refresh_encumbrance(state: &mut DesktopCanonicalState) -> Result<(), DesktopProgressionError> {
    let mut total = 0_u64;
    for row in &state.inventory {
        if row.caption != "Gold" {
            total = total
                .checked_add(row_value(row, "inventory quantity")?)
                .ok_or(DesktopProgressionError::InvalidNumber("encumbrance"))?;
        }
    }
    state.bars.encumbrance.position = total.min(state.bars.encumbrance.maximum);
    Ok(())
}

fn random_low(upper_bound: u64, random: &mut DelphiRandom) -> Result<u64, DesktopProgressionError> {
    let upper_bound = u32::try_from(upper_bound)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(DesktopProgressionError::InvalidNumber("random bound"))?;
    let first = random
        .bounded(upper_bound)
        .map_err(|_| DesktopProgressionError::InvalidNumber("random bound"))?;
    let second = random
        .bounded(upper_bound)
        .map_err(|_| DesktopProgressionError::InvalidNumber("random bound"))?;
    Ok(u64::from(first.min(second)))
}

fn proper_case(value: &str) -> String {
    let mut characters = value.chars();
    match characters.next() {
        Some(first) => first.to_uppercase().chain(characters).collect(),
        None => String::new(),
    }
}

fn named_row_value(
    rows: &[DesktopValidatedRow],
    caption: &'static str,
) -> Result<u64, DesktopProgressionError> {
    row_value(
        rows.iter()
            .find(|row| row.caption == caption)
            .ok_or(DesktopProgressionError::InvalidNumber(caption))?,
        caption,
    )
}

fn add_row_value(
    rows: &mut [DesktopValidatedRow],
    caption: &str,
    change: i64,
) -> Result<(), DesktopProgressionError> {
    let row = rows
        .iter_mut()
        .find(|row| row.caption == caption)
        .ok_or(DesktopProgressionError::InvalidNumber("row value"))?;
    let current = row
        .subitems
        .first()
        .ok_or(DesktopProgressionError::InvalidNumber("row value"))?
        .parse::<i64>()
        .map_err(|_| DesktopProgressionError::InvalidNumber("row value"))?;
    let next = current
        .checked_add(change)
        .filter(|value| *value >= 0)
        .ok_or(DesktopProgressionError::InvalidNumber("row value"))?;
    row.subitems[0] = next.to_string();
    Ok(())
}

fn win_stat(
    state: &mut DesktopCanonicalState,
    random: &mut DelphiRandom,
) -> Result<(), DesktopProgressionError> {
    let values = state
        .stats
        .iter()
        .map(|row| row_value(row, "stat"))
        .collect::<Result<Vec<_>, _>>()?;
    let index = weighted_stat_index(&values, random)
        .map_err(|_| DesktopProgressionError::InvalidNumber("stat weights"))?;
    let caption = state
        .stats
        .get(index)
        .map(|row| row.caption.clone())
        .ok_or(DesktopProgressionError::InvalidNumber("stat index"))?;
    add_row_value(&mut state.stats, &caption, 1)?;
    if caption == "STR" {
        state.bars.encumbrance.maximum = 10_u64
            .checked_add(named_row_value(&state.stats, "STR")?)
            .ok_or(DesktopProgressionError::InvalidNumber(
                "encumbrance maximum",
            ))?;
        state.bars.encumbrance.position = state
            .bars
            .encumbrance
            .position
            .min(state.bars.encumbrance.maximum);
    }
    Ok(())
}

fn win_spell(
    state: &mut DesktopCanonicalState,
    random: &mut DelphiRandom,
) -> Result<(), DesktopProgressionError> {
    let rules = &bundled().tables.spells;
    let upper = named_row_value(&state.stats, "WIS")?
        .checked_add(character_level(state)?)
        .map(|value| value.min(rules.len() as u64))
        .filter(|value| *value > 0)
        .ok_or(DesktopProgressionError::InvalidNumber("spell bound"))?;
    let index = usize::try_from(random_low(upper, random)?)
        .map_err(|_| DesktopProgressionError::InvalidNumber("spell index"))?;
    let name = &rules[index];
    if let Some(spell) = state.spells.iter_mut().find(|spell| spell.caption == *name) {
        let rank = spell
            .subitems
            .first()
            .ok_or(DesktopProgressionError::InvalidNumber("spell rank"))
            .and_then(|value| roman_to_integer(value))?;
        spell.subitems[0] = desktop_roman(
            rank.checked_add(1)
                .ok_or(DesktopProgressionError::InvalidNumber("spell rank"))?,
        );
    } else {
        state.spells.push(DesktopValidatedRow {
            header: [-1, -1, -1, 0, 1],
            caption: name.clone(),
            subitems: vec!["I".to_owned()],
        });
    }
    Ok(())
}

fn win_item(
    state: &mut DesktopCanonicalState,
    random: &mut DelphiRandom,
) -> Result<(), DesktopProgressionError> {
    let roll = bounded(random, 999)?;
    let name = if roll.max(250)
        < u32::try_from(state.inventory.len())
            .map_err(|_| DesktopProgressionError::InvalidNumber("inventory length"))?
    {
        let index = usize::try_from(bounded(
            random,
            u32::try_from(state.inventory.len())
                .map_err(|_| DesktopProgressionError::InvalidNumber("inventory length"))?,
        )?)
        .map_err(|_| DesktopProgressionError::InvalidNumber("inventory index"))?;
        state.inventory[index].caption.clone()
    } else {
        let rules = &bundled().tables;
        format!(
            "{} {} of {}",
            pick(&rules.item_attrib, random)?,
            pick(&rules.specials, random)?,
            pick(&rules.item_ofs, random)?
        )
    };
    add_inventory(state, &name, 1)
}

fn win_equipment(
    state: &mut DesktopCanonicalState,
    random: &mut DelphiRandom,
) -> Result<(), DesktopProgressionError> {
    let position = usize::try_from(bounded(
        random,
        u32::try_from(state.equipment.len())
            .ok()
            .filter(|value| *value > 0)
            .ok_or(DesktopProgressionError::InvalidNumber("equipment length"))?,
    )?)
    .map_err(|_| DesktopProgressionError::InvalidNumber("equipment index"))?;
    let rules = &bundled().tables;
    let (items, better, worse) = if position == 0 {
        (&rules.weapons, &rules.offense_attrib, &rules.offense_bad)
    } else if position == 1 {
        (&rules.shields, &rules.defense_attrib, &rules.defense_bad)
    } else {
        (&rules.armors, &rules.defense_attrib, &rules.defense_bad)
    };
    let level = i64::try_from(character_level(state)?)
        .map_err(|_| DesktopProgressionError::InvalidNumber("level"))?;
    let selected = closest_rule(items, level, random)?;
    let (name, quality) = split_rule(selected)?;
    let mut name = name.to_owned();
    let mut plus = level
        .checked_sub(quality)
        .ok_or(DesktopProgressionError::InvalidNumber("equipment quality"))?;
    let modifiers = if plus < 0 { worse } else { better };
    for _ in 0..2 {
        if plus == 0 {
            break;
        }
        let (modifier, quality) = split_rule(pick(modifiers, random)?)?;
        if name.contains(modifier) || plus.abs() < quality.abs() {
            break;
        }
        name = format!("{modifier} {name}");
        plus = plus
            .checked_sub(quality)
            .ok_or(DesktopProgressionError::InvalidNumber("equipment modifier"))?;
    }
    if plus != 0 {
        name = if plus > 0 {
            format!("+{plus} {name}")
        } else {
            format!("{plus} {name}")
        };
    }
    state
        .equipment
        .get_mut(position)
        .ok_or(DesktopProgressionError::InvalidNumber("equipment index"))?
        .subitems[0] = name;
    state.prized_equipment = position;
    Ok(())
}

fn closest_rule<'a>(
    values: &'a [String],
    goal: i64,
    random: &mut DelphiRandom,
) -> Result<&'a str, DesktopProgressionError> {
    let mut selected = pick(values, random)?;
    for _ in 0..5 {
        let candidate = pick(values, random)?;
        if (goal - split_rule(selected)?.1).abs() > (goal - split_rule(candidate)?.1).abs() {
            selected = candidate;
        }
    }
    Ok(selected)
}

fn split_rule(value: &str) -> Result<(&str, i64), DesktopProgressionError> {
    let (name, quality) = value
        .rsplit_once('|')
        .ok_or(DesktopProgressionError::InvalidNumber("rule"))?;
    let quality = quality
        .parse()
        .map_err(|_| DesktopProgressionError::InvalidNumber("rule quality"))?;
    Ok((name, quality))
}

fn roman_to_integer(value: &str) -> Result<u64, DesktopProgressionError> {
    let mut rest = value;
    let mut total = 0_u64;
    for (token, amount, repeat) in [
        ("T", 10_000, true),
        ("MT", 9_000, false),
        ("A", 5_000, false),
        ("MA", 4_000, false),
        ("M", 1_000, true),
        ("CM", 900, false),
        ("D", 500, false),
        ("CD", 400, false),
        ("C", 100, true),
        ("XC", 90, false),
        ("L", 50, false),
        ("XL", 40, false),
        ("X", 10, true),
        ("IX", 9, false),
        ("V", 5, false),
        ("IV", 4, false),
        ("I", 1, true),
    ] {
        let mut matched = false;
        while rest.starts_with(token) && (repeat || !matched) {
            total = total
                .checked_add(amount)
                .ok_or(DesktopProgressionError::InvalidNumber("spell rank"))?;
            rest = &rest[token.len()..];
            matched = true;
        }
    }
    if total == 0 || !rest.is_empty() {
        return Err(DesktopProgressionError::InvalidNumber("spell rank"));
    }
    Ok(total)
}

fn desktop_roman(mut value: u64) -> String {
    let mut encoded = String::new();
    for (amount, token) in [
        (10_000, "T"),
        (9_000, "MT"),
        (5_000, "A"),
        (4_000, "MA"),
        (1_000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ] {
        while value >= amount {
            encoded.push_str(token);
            value -= amount;
        }
    }
    encoded
}

pub fn best_spell_index(
    spells: &[DesktopValidatedRow],
) -> Result<Option<usize>, DesktopProgressionError> {
    let Some(first) = spells.first() else {
        return Ok(None);
    };
    let mut best = 0;
    let mut best_score = roman_to_integer(
        first
            .subitems
            .first()
            .ok_or(DesktopProgressionError::InvalidNumber("spell rank"))?,
    )?;
    for (index, spell) in spells.iter().enumerate().skip(1) {
        let rank = roman_to_integer(
            spell
                .subitems
                .first()
                .ok_or(DesktopProgressionError::InvalidNumber("spell rank"))?,
        )?;
        let score = u64::try_from(index + 1)
            .ok()
            .and_then(|position| position.checked_mul(rank))
            .ok_or(DesktopProgressionError::InvalidNumber("spell specialty"))?;
        if score > best_score {
            best = index;
            best_score = score;
        }
    }
    Ok(Some(best))
}

pub fn best_prime_stat_index(
    stats: &[DesktopValidatedRow],
) -> Result<usize, DesktopProgressionError> {
    if stats.len() < 6 {
        return Err(DesktopProgressionError::InvalidNumber("prime stats"));
    }
    let mut best = 0;
    let mut best_value = row_value(&stats[0], "prime stat")?;
    for (index, stat) in stats.iter().take(6).enumerate().skip(1) {
        let value = row_value(stat, "prime stat")?;
        if value > best_value {
            best = index;
            best_value = value;
        }
    }
    Ok(best)
}

fn complete_quest(
    state: &mut DesktopCanonicalState,
    random: &mut DelphiRandom,
    hooks: &mut impl DesktopProgressionHooks,
) -> Result<(), DesktopProgressionError> {
    state.bars.quest = DesktopValidatedBar {
        position: 0,
        maximum: 50 + u64::from(bounded(random, 100)?),
    };
    if let Some(quest) = state.quests.last_mut() {
        quest.header[1] = 1;
        hooks.quest_reward(state, random)?;
    }
    while state.quests.len() > 99 {
        state.quests.remove(0);
    }

    let rules = &bundled().tables;
    let level = character_level(state)?;
    let (caption, quest) = match bounded(random, 5)? {
        0 => {
            let (index, monster) = closest_monster(level, 4, random)?;
            let name = rule_name(monster)?;
            (
                format!("Exterminate {}", definite(name, 2)),
                DesktopQuestMarker::Value {
                    marker: monster.to_owned(),
                    index,
                },
            )
        }
        1 => {
            let item = interesting_item(random)?;
            (
                format!("Seek {}", definite(&item, 1)),
                DesktopQuestMarker::None,
            )
        }
        2 => {
            let item = pick(&rules.boring_items, random)?;
            (format!("Deliver this {item}"), DesktopQuestMarker::None)
        }
        3 => {
            let item = pick(&rules.boring_items, random)?;
            (
                format!("Fetch me {}", indefinite(item, 1)),
                DesktopQuestMarker::None,
            )
        }
        4 => {
            let (_, monster) = closest_monster(level, 2, random)?;
            (
                format!("Placate {}", definite(rule_name(monster)?, 2)),
                DesktopQuestMarker::None,
            )
        }
        _ => unreachable!(),
    };
    state.quest = quest;
    state.quests.push(DesktopValidatedRow {
        header: [-1, 0, -1, 0, 0],
        caption,
        subitems: Vec::new(),
    });
    Ok(())
}

fn interplot_cinematic(
    state: &mut DesktopCanonicalState,
    random: &mut DelphiRandom,
) -> Result<(), DesktopProgressionError> {
    match bounded(random, 3)? {
        0 => {
            queue_task(
                state,
                1,
                "Exhausted, you arrive at a friendly oasis in a hostile land",
            );
            queue_task(state, 2, "You greet old friends and meet new allies");
            queue_task(
                state,
                2,
                "You are privy to a council of powerful do-gooders",
            );
            queue_task(state, 1, "There is much to be done. You are chosen!");
        }
        1 => {
            queue_task(
                state,
                1,
                "Your quarry is in sight, but a mighty enemy bars your path!",
            );
            let nemesis = named_monster(character_level(state)? + 3, random)?;
            queue_task(
                state,
                4,
                &format!("A desperate struggle commences with {nemesis}"),
            );
            let mut sequence = bounded(random, 3)?;
            let repeats = bounded(
                random,
                u32::try_from(1 + state.plots.len())
                    .map_err(|_| DesktopProgressionError::InvalidNumber("plot count"))?,
            )?;
            for _ in 0..repeats {
                sequence += 1 + bounded(random, 2)?;
                let caption = match sequence % 3 {
                    0 => format!("Locked in grim combat with {nemesis}"),
                    1 => format!("{nemesis} seems to have the upper hand"),
                    _ => format!("You seem to gain the advantage over {nemesis}"),
                };
                queue_task(state, 2, &caption);
            }
            queue_task(
                state,
                3,
                &format!("Victory! {nemesis} is slain! Exhausted, you lose conciousness"),
            );
            queue_task(
                state,
                2,
                "You awake in a friendly place, but the road awaits",
            );
        }
        2 => {
            let nemesis = impressive_guy(random)?;
            queue_task(
                state,
                2,
                &format!("Oh sweet relief! You've reached the kind protection of {nemesis}"),
            );
            queue_task(
                state,
                3,
                &format!("There is rejoicing, and an unnerving encouter with {nemesis} in private"),
            );
            let item = pick(&bundled().tables.boring_items, random)?;
            queue_task(
                state,
                2,
                &format!("You forget your {item} and go back to get it"),
            );
            queue_task(state, 2, "What's this!? You overhear something shocking!");
            queue_task(
                state,
                2,
                &format!("Could {nemesis} be a dirty double-dealer?"),
            );
            queue_task(
                state,
                3,
                "Who can possibly be trusted with this news!? -- Oh yes, of course",
            );
        }
        _ => unreachable!(),
    }
    state.queue.push(DesktopQueueCommand {
        kind: DesktopQueueKind::Plot,
        duration_seconds: 2,
        caption: "Loading".to_owned(),
    });
    Ok(())
}

fn select_combat_task(
    state: &mut DesktopCanonicalState,
    random: &mut DelphiRandom,
) -> Result<(), DesktopProgressionError> {
    let original_level = character_level(state)?;
    let mut level = i64::try_from(original_level)
        .map_err(|_| DesktopProgressionError::InvalidNumber("level"))?;
    for _ in 0..original_level {
        if odds(random, 2, 5)? {
            level += if bounded(random, 2)? == 0 { -1 } else { 1 };
        }
    }
    level = level.max(1);

    let rules = &bundled().tables;
    let (monster_rule, mut display, monster_level, definite_name) = if odds(random, 1, 25)? {
        let race = rule_name(pick(&rules.races, random)?)?;
        if odds(random, 1, 2)? {
            let class = rule_name(pick(&rules.klasses, random)?)?;
            (
                format!("passing {race} {class}|{level}|*"),
                format!("passing {race} {class}"),
                level,
                false,
            )
        } else {
            let title = pick_low(&rules.titles, random)?;
            let name = generate_name(random)?;
            (
                format!("{title} {name} the {race}|{level}|*"),
                format!("{title} {name} the {race}"),
                level,
                true,
            )
        }
    } else if !matches!(state.quest, DesktopQuestMarker::None) && odds(random, 1, 4)? {
        let index = match state.quest {
            DesktopQuestMarker::LegacyPlaceholder { index }
            | DesktopQuestMarker::Value { index, .. } => index,
            DesktopQuestMarker::None => unreachable!(),
        };
        let monster = rules
            .monsters
            .get(index)
            .ok_or(DesktopProgressionError::InvalidNumber("quest index"))?;
        (
            monster.clone(),
            rule_name(monster)?.to_owned(),
            rule_quality(monster)?,
            false,
        )
    } else {
        let (_, monster) = closest_monster(
            u64::try_from(level)
                .map_err(|_| DesktopProgressionError::InvalidNumber("monster level"))?,
            6,
            random,
        )?;
        (
            monster.to_owned(),
            rule_name(monster)?.to_owned(),
            rule_quality(monster)?,
            false,
        )
    };

    let difference = level - monster_level;
    let mut quantity = 1_i64;
    if difference > 10 {
        let bound = u32::try_from(monster_level)
            .ok()
            .filter(|value| *value > 0)
            .ok_or(DesktopProgressionError::InvalidNumber("monster level"))?;
        quantity = (level + i64::from(bounded(random, bound)?)) / monster_level.max(1);
        quantity = quantity.max(1);
        level /= quantity;
    }

    let difference = level - monster_level;
    display = if difference <= -10 {
        format!("imaginary {display}")
    } else if difference < -5 {
        let first = 5 - i64::from(bounded(
            random,
            u32::try_from(11 + difference)
                .map_err(|_| DesktopProgressionError::InvalidNumber("monster modifier"))?,
        )?);
        sick(first, &young(-difference - first, &display))
    } else if difference < 0 && bounded(random, 2)? == 1 {
        sick(difference, &display)
    } else if difference < 0 {
        young(difference, &display)
    } else if difference >= 10 {
        format!("messianic {display}")
    } else if difference > 5 {
        let first = 5 - i64::from(bounded(
            random,
            u32::try_from(11 - difference)
                .map_err(|_| DesktopProgressionError::InvalidNumber("monster modifier"))?,
        )?);
        big(first, &special(difference - first, &display))
    } else if difference > 0 && bounded(random, 2)? == 1 {
        big(difference, &display)
    } else if difference > 0 {
        special(difference, &display)
    } else {
        display
    };

    let adjusted_level = level
        .checked_mul(quantity)
        .ok_or(DesktopProgressionError::InvalidNumber("monster level"))?;
    if !definite_name {
        display = indefinite(
            &display,
            u64::try_from(quantity)
                .map_err(|_| DesktopProgressionError::InvalidNumber("monster quantity"))?,
        );
    }
    let duration = 2_u64
        .checked_mul(u64::from(state.game_style))
        .and_then(|value| value.checked_mul(u64::try_from(adjusted_level).ok()?))
        .and_then(|value| value.checked_mul(1_000))
        .map(|value| value / original_level)
        .filter(|value| *value > 0)
        .ok_or(DesktopProgressionError::InvalidNumber("task duration"))?;
    start_task(
        state,
        &format!("kill|{monster_rule}"),
        &format!("Executing {display}"),
        duration,
    )
}

fn queue_task(state: &mut DesktopCanonicalState, duration_seconds: u32, caption: &str) {
    state.queue.push(DesktopQueueCommand {
        kind: DesktopQueueKind::Task,
        duration_seconds,
        caption: caption.to_owned(),
    });
}

fn closest_monster(
    level: u64,
    samples: usize,
    random: &mut DelphiRandom,
) -> Result<(usize, &'static str), DesktopProgressionError> {
    let monsters = &bundled().tables.monsters;
    let mut selected = 0;
    let mut selected_level = 0_i64;
    for sample in 0..samples {
        let index = usize::try_from(bounded(
            random,
            u32::try_from(monsters.len())
                .map_err(|_| DesktopProgressionError::InvalidNumber("monster table"))?,
        )?)
        .map_err(|_| DesktopProgressionError::InvalidNumber("monster index"))?;
        let candidate_level = rule_quality(&monsters[index])?;
        if sample == 0
            || (i64::try_from(level).unwrap_or(i64::MAX) - candidate_level).abs()
                < (i64::try_from(level).unwrap_or(i64::MAX) - selected_level).abs()
        {
            selected = index;
            selected_level = candidate_level;
        }
    }
    Ok((selected, bundled().tables.monsters[selected].as_str()))
}

fn named_monster(level: u64, random: &mut DelphiRandom) -> Result<String, DesktopProgressionError> {
    let (_, monster) = closest_monster(level, 5, random)?;
    Ok(format!(
        "{} the {}",
        generate_name(random)?,
        rule_name(monster)?
    ))
}

fn impressive_guy(random: &mut DelphiRandom) -> Result<String, DesktopProgressionError> {
    let rules = &bundled().tables;
    let title = pick(&rules.impressive_titles, random)?;
    if bounded(random, 2)? == 0 {
        let race = rule_name(pick(&rules.races, random)?)?;
        Ok(format!("the {title} of the {}", plural(race)))
    } else {
        Ok(format!(
            "{title} {} of {}",
            generate_name(random)?,
            generate_name(random)?
        ))
    }
}

fn interesting_item(random: &mut DelphiRandom) -> Result<String, DesktopProgressionError> {
    let rules = &bundled().tables;
    Ok(format!(
        "{} {}",
        pick(&rules.item_attrib, random)?,
        pick(&rules.specials, random)?
    ))
}

fn generate_name(random: &mut DelphiRandom) -> Result<String, DesktopProgressionError> {
    const PARTS: [&[&str]; 3] = [
        &[
            "br", "cr", "dr", "fr", "gr", "j", "kr", "l", "m", "n", "pr", "", "", "", "r", "sh",
            "tr", "v", "wh", "x", "y", "z",
        ],
        &[
            "a", "a", "e", "e", "i", "i", "o", "o", "u", "u", "ae", "ie", "oo", "ou",
        ],
        &["b", "ck", "d", "g", "k", "m", "n", "p", "t", "v", "x", "z"],
    ];
    let mut result = String::new();
    for index in 0..6 {
        result.push_str(pick_slice(PARTS[index % 3], random)?);
    }
    Ok(proper_case(&result))
}

fn pick<'a>(
    values: &'a [String],
    random: &mut DelphiRandom,
) -> Result<&'a str, DesktopProgressionError> {
    pick_slice(values, random).map(String::as_str)
}

fn pick_slice<'a, T>(
    values: &'a [T],
    random: &mut DelphiRandom,
) -> Result<&'a T, DesktopProgressionError> {
    let bound = u32::try_from(values.len())
        .ok()
        .filter(|value| *value > 0)
        .ok_or(DesktopProgressionError::InvalidNumber("rule table"))?;
    let index = usize::try_from(bounded(random, bound)?)
        .map_err(|_| DesktopProgressionError::InvalidNumber("rule index"))?;
    Ok(&values[index])
}

fn pick_low<'a>(
    values: &'a [String],
    random: &mut DelphiRandom,
) -> Result<&'a str, DesktopProgressionError> {
    let index = usize::try_from(random_low(values.len() as u64, random)?)
        .map_err(|_| DesktopProgressionError::InvalidNumber("rule index"))?;
    Ok(&values[index])
}

fn bounded(random: &mut DelphiRandom, upper: u32) -> Result<u32, DesktopProgressionError> {
    random
        .bounded(upper)
        .map_err(|_| DesktopProgressionError::InvalidNumber("random bound"))
}

fn odds(
    random: &mut DelphiRandom,
    chance: u32,
    out_of: u32,
) -> Result<bool, DesktopProgressionError> {
    Ok(bounded(random, out_of)? < chance)
}

fn rule_name(rule: &str) -> Result<&str, DesktopProgressionError> {
    rule.split('|')
        .next()
        .filter(|value| !value.is_empty())
        .ok_or(DesktopProgressionError::InvalidNumber("rule"))
}

fn rule_quality(rule: &str) -> Result<i64, DesktopProgressionError> {
    rule.split('|')
        .nth(1)
        .ok_or(DesktopProgressionError::InvalidNumber("rule quality"))?
        .parse()
        .map_err(|_| DesktopProgressionError::InvalidNumber("rule quality"))
}

fn definite(value: &str, quantity: u64) -> String {
    let value = if quantity > 1 {
        plural(value)
    } else {
        value.to_owned()
    };
    format!("the {value}")
}

fn sick(amount: i64, value: &str) -> String {
    let adjective = match amount.abs() {
        1 => "undernourished",
        2 => "sick",
        3 => "crippled",
        4 => "comatose",
        5 => "dead",
        _ => return format!("{amount}{value}"),
    };
    format!("{adjective} {value}")
}

fn young(amount: i64, value: &str) -> String {
    let adjective = match amount.abs() {
        1 => "underage",
        2 => "teenage",
        3 => "preadolescent",
        4 => "baby",
        5 => "foetal",
        _ => return format!("{amount}{value}"),
    };
    format!("{adjective} {value}")
}

fn big(amount: i64, value: &str) -> String {
    let adjective = match amount.abs() {
        1 => "greater",
        2 => "massive",
        3 => "enormous",
        4 => "giant",
        5 => "titanic",
        _ => return value.to_owned(),
    };
    format!("{adjective} {value}")
}

fn special(amount: i64, value: &str) -> String {
    match amount.abs() {
        1 if value.contains(' ') => format!("veteran {value}"),
        1 => format!("Battle-{value}"),
        2 => format!("cursed {value}"),
        3 if value.contains(' ') => format!("warrior {value}"),
        3 => format!("Were-{value}"),
        4 => format!("undead {value}"),
        5 => format!("demon {value}"),
        _ => value.to_owned(),
    }
}

fn indefinite(value: &str, quantity: u64) -> String {
    if quantity == 1 {
        let article = if value
            .chars()
            .next()
            .is_some_and(|character| "AEIOUaeiou".contains(character))
        {
            "an"
        } else {
            "a"
        };
        format!("{article} {value}")
    } else {
        format!("{quantity} {}", plural(value))
    }
}

fn plural(value: &str) -> String {
    if let Some(stem) = value.strip_suffix('y') {
        format!("{stem}ies")
    } else if let Some(stem) = value.strip_suffix("us") {
        format!("{stem}i")
    } else if value.ends_with("ch") || value.ends_with('x') || value.ends_with('s') {
        format!("{value}es")
    } else if let Some(stem) = value.strip_suffix('f') {
        format!("{stem}ves")
    } else if value.ends_with("man") || value.ends_with("Man") {
        format!("{}en", &value[..value.len() - 2])
    } else {
        format!("{value}s")
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        compatibility::DesktopRandomState,
        desktop_callback::DesktopCallbackCheckpoint,
        desktop_save::{DesktopValidatedBars, DesktopValidatedProfile},
    };

    use super::*;

    #[derive(Default)]
    struct Hooks {
        level_ups: usize,
        quests: usize,
        acts: usize,
        items: usize,
        equipment: usize,
    }

    impl DesktopProgressionHooks for Hooks {
        fn level_up(
            &mut self,
            state: &mut DesktopCanonicalState,
            _random: &mut DelphiRandom,
        ) -> Result<(), DesktopProgressionError> {
            self.level_ups += 1;
            state.bars.experience = bar(0, 100);
            Ok(())
        }

        fn quest_reward(
            &mut self,
            _state: &mut DesktopCanonicalState,
            _random: &mut DelphiRandom,
        ) -> Result<(), DesktopProgressionError> {
            self.quests += 1;
            Ok(())
        }

        fn complete_act(
            &mut self,
            state: &mut DesktopCanonicalState,
            _random: &mut DelphiRandom,
        ) -> Result<(), DesktopProgressionError> {
            self.acts += 1;
            state.plots.push(row(&format!("Act {}", self.acts), "0"));
            Ok(())
        }

        fn win_item(
            &mut self,
            _state: &mut DesktopCanonicalState,
            _random: &mut DelphiRandom,
        ) -> Result<(), DesktopProgressionError> {
            self.items += 1;
            Ok(())
        }

        fn win_equipment(
            &mut self,
            _state: &mut DesktopCanonicalState,
            _random: &mut DelphiRandom,
        ) -> Result<(), DesktopProgressionError> {
            self.equipment += 1;
            Ok(())
        }
    }

    fn row(caption: &str, value: &str) -> DesktopValidatedRow {
        DesktopValidatedRow {
            header: [0, -1, -1, 0, 1],
            caption: caption.to_owned(),
            subitems: vec![value.to_owned()],
        }
    }

    fn bar(position: u64, maximum: u64) -> DesktopValidatedBar {
        DesktopValidatedBar { position, maximum }
    }

    fn state(task: &str, task_maximum: u64) -> DesktopCanonicalState {
        DesktopCanonicalState {
            traits: vec![
                row("Name", "Hero"),
                row("Race", "Gyrognome"),
                row("Class", "Robot Monk"),
                row("Level", "2"),
            ],
            stats: Vec::new(),
            equipment: Vec::new(),
            inventory: vec![row("Gold", "0")],
            spells: Vec::new(),
            plots: vec![row("Prologue", "0"), row("Act I", "0")],
            quests: vec![row("Synthetic quest", "0")],
            current_task: task.to_owned(),
            quest: DesktopQuestMarker::None,
            queue: vec![DesktopQueueCommand {
                kind: DesktopQueueKind::Task,
                duration_seconds: 2,
                caption: "Continue".to_owned(),
            }],
            activity: "Working...".to_owned(),
            bars: DesktopValidatedBars {
                experience: bar(0, 100),
                encumbrance: bar(0, 20),
                plot: bar(0, 100),
                quest: bar(0, 100),
                task: bar(task_maximum, task_maximum),
            },
            prized_equipment: 0,
            game_style: 3,
            profile: DesktopValidatedProfile {
                motto: String::new(),
                guild: String::new(),
            },
        }
    }

    #[test]
    fn credits_fractional_task_durations_with_pascal_integer_division() {
        for (maximum, credit) in [(5_114, 5), (5_508, 5), (6_295, 6)] {
            let mut state = state("kill|Rat|1|tail", maximum);
            let mut random = DelphiRandom::from_state(DesktopRandomState(1));
            let observation =
                complete_task(&mut state, &mut random, &mut Hooks::default()).unwrap();
            assert_eq!(observation.credited_seconds, credit);
            assert_eq!(state.bars.experience.position, credit);
            assert_eq!(state.bars.quest.position, credit);
            assert_eq!(state.bars.plot.position, credit);
        }
    }

    #[test]
    fn credits_noncombat_nonloading_tasks_to_plot_only() {
        let mut state = state("market", 5_508);
        let mut random = DelphiRandom::from_state(DesktopRandomState(1));
        complete_task(&mut state, &mut random, &mut Hooks::default()).unwrap();
        assert_eq!(state.bars.experience.position, 0);
        assert_eq!(state.bars.quest.position, 0);
        assert_eq!(state.bars.plot.position, 5);
    }

    #[test]
    fn starts_and_drains_market_sales_in_inventory_order() {
        let mut state = state("market", 4_000);
        state.queue.clear();
        state.inventory.push(row("rat Tail", "2"));
        state.inventory.push(row("bat Wing", "1"));
        state.bars.encumbrance = bar(3, 3);
        let mut random = DelphiRandom::from_state(DesktopRandomState(1));
        let mut hooks = Hooks::default();

        complete_task(&mut state, &mut random, &mut hooks).unwrap();
        assert_eq!(state.current_task, "sell");
        assert_eq!(state.bars.task, bar(0, 1_000));
        assert_eq!(state.activity, "Selling 2 rat Tails...");

        state.bars.task.position = 1_000;
        complete_task(&mut state, &mut random, &mut hooks).unwrap();
        assert_eq!(state.inventory[0].subitems[0], "4");
        assert_eq!(state.inventory[1].caption, "bat Wing");
        assert_eq!(state.current_task, "sell");
        assert_eq!(state.bars.encumbrance.position, 1);
    }

    #[test]
    fn records_fixed_monster_drops_with_desktop_casing() {
        let mut state = state("kill|Goblin|1|Ear", 5_114);
        let mut random = DelphiRandom::from_state(DesktopRandomState(1));
        complete_task(&mut state, &mut random, &mut Hooks::default()).unwrap();
        assert_eq!(state.inventory.last().unwrap().caption, "goblin ear");
    }

    #[test]
    fn preserves_legacy_and_modern_prologue_queue_semantics() {
        let mut legacy = state("load", 2_000);
        legacy.plots = vec![row("Prologue", "0")];
        legacy.queue = vec![DesktopQueueCommand {
            kind: DesktopQueueKind::Task,
            duration_seconds: 2,
            caption: "Loading".to_owned(),
        }];
        let mut modern = legacy.clone();
        modern.queue[0].kind = DesktopQueueKind::Plot;
        let mut random = DelphiRandom::from_state(DesktopRandomState(1));

        complete_task(&mut legacy, &mut random, &mut Hooks::default()).unwrap();
        assert_eq!(legacy.activity, "Loading...");
        assert_eq!(legacy.plots.len(), 1);

        let mut hooks = Hooks::default();
        complete_task(&mut modern, &mut random, &mut hooks).unwrap();
        assert_eq!(modern.activity, "Loading Act 1...");
        assert_eq!(modern.plots.last().unwrap().caption, "Act 1");
        assert_eq!(hooks.acts, 1);
    }

    #[test]
    fn quest_placeholder_study_monster_selection_uses_nonempty_marker_and_exact_index() {
        let mut selected_quest_monster = false;
        let mut mismatched_index_changes_selection = false;
        for index in 0..bundled().tables.monsters.len() {
            for seed in 0..64 {
                let mut placeholder = state("heading", 4_000);
                placeholder.quest = DesktopQuestMarker::LegacyPlaceholder { index };
                let mut canonical = placeholder.clone();
                canonical.quest = DesktopQuestMarker::Value {
                    marker: bundled().tables.monsters[index].clone(),
                    index,
                };
                let mut left_random = DelphiRandom::from_state(DesktopRandomState(seed));
                let mut right_random = left_random;
                select_combat_task(&mut placeholder, &mut left_random).unwrap();
                select_combat_task(&mut canonical, &mut right_random).unwrap();
                assert_eq!(placeholder.current_task, canonical.current_task);
                assert_eq!(placeholder.activity, canonical.activity);
                assert_eq!(placeholder.bars, canonical.bars);
                assert_eq!(left_random.state(), right_random.state());
                selected_quest_monster |= placeholder.current_task
                    == format!("kill|{}", bundled().tables.monsters[index]);

                if index == 0 {
                    let mut mismatched = state("heading", 4_000);
                    mismatched.quest = DesktopQuestMarker::Value {
                        marker: bundled().tables.monsters[0].clone(),
                        index: 1,
                    };
                    let mut random = DelphiRandom::from_state(DesktopRandomState(seed));
                    select_combat_task(&mut mismatched, &mut random).unwrap();
                    mismatched_index_changes_selection |=
                        mismatched.current_task != placeholder.current_task;
                }
            }
        }
        assert!(selected_quest_monster);
        assert!(mismatched_index_changes_selection);
    }

    #[test]
    fn quest_placeholder_study_resolution_rejects_invalid_indexes_and_preserves_unknown_markers() {
        for index in [bundled().tables.monsters.len(), usize::MAX] {
            let mut candidate = state("heading", 4_000);
            candidate.quest = DesktopQuestMarker::LegacyPlaceholder { index };
            let original = candidate.clone();
            assert_eq!(
                resolve_legacy_quest_marker(&mut candidate),
                Err(DesktopProgressionError::InvalidNumber("quest index"))
            );
            assert_eq!(candidate, original);
        }
        let mut unknown = state("heading", 4_000);
        unknown.quest = DesktopQuestMarker::Value {
            marker: "Unknown synthetic caption".to_owned(),
            index: 1,
        };
        let original = unknown.clone();
        resolve_legacy_quest_marker(&mut unknown).unwrap();
        assert_eq!(unknown, original);
    }

    #[test]
    fn resolves_carried_forward_quest_placeholder_by_pinned_index() {
        let mut state = state("kill|Rat|1|tail", 5_114);
        state.quest = DesktopQuestMarker::LegacyPlaceholder { index: 1 };
        let expected = bundled().tables.monsters[1].clone();
        let mut random = DelphiRandom::from_state(DesktopRandomState(1));

        complete_task(&mut state, &mut random, &mut Hooks::default()).unwrap();
        assert_eq!(
            state.quest,
            DesktopQuestMarker::Value {
                marker: expected,
                index: 1,
            }
        );
    }

    #[test]
    fn dispatches_due_progression_before_dequeue() {
        let mut state = state("kill|Rat|1|tail", 6_000);
        state.bars.experience.position = state.bars.experience.maximum;
        state.bars.quest.position = state.bars.quest.maximum;
        state.bars.plot.position = state.bars.plot.maximum;
        state.queue.clear();
        let mut random = DelphiRandom::from_state(DesktopRandomState(1));
        let mut hooks = Hooks::default();

        let observation = complete_task(&mut state, &mut random, &mut hooks).unwrap();
        assert!(observation.level_up_dispatched);
        assert!(observation.quest_completion_dispatched);
        assert!(observation.interplot_dispatched);
        assert_eq!((hooks.level_ups, hooks.quests), (1, 1));
        assert_eq!(hooks.acts, 0);
        assert_eq!(state.current_task, "");
        assert!(!state.queue.is_empty());
        assert_ne!(state.activity, "Loading Act 1...");
    }

    #[test]
    fn completes_and_replaces_quests_in_desktop_list_order() {
        let mut state = state("kill|Rat|1|tail", 6_000);
        state.bars.quest.position = state.bars.quest.maximum;
        let mut random = DelphiRandom::from_state(DesktopRandomState(0x1234_5678));
        let mut hooks = Hooks::default();

        complete_task(&mut state, &mut random, &mut hooks).unwrap();
        assert_eq!(hooks.quests, 1);
        assert_eq!(state.quests.len(), 2);
        assert_eq!(state.quests[0].header[1], 1);
        assert_eq!(state.quests[1].header, [-1, 0, -1, 0, 0]);
        assert!((50..150).contains(&state.bars.quest.maximum));
    }

    #[test]
    fn selects_a_source_derived_combat_task_after_heading() {
        let mut state = state("heading", 4_000);
        state.queue.clear();
        let mut random = DelphiRandom::from_state(DesktopRandomState(0x1234_5678));

        complete_task(&mut state, &mut random, &mut Hooks::default()).unwrap();
        assert!(state.current_task.starts_with("kill|"));
        assert!(state.activity.starts_with("Executing "));
        assert_eq!(state.bars.task.position, 0);
        assert!(state.bars.task.maximum > 0);
        assert_ne!(random.state(), DesktopRandomState(0x1234_5678));
    }

    #[test]
    fn matches_source_derived_level_up_rewards_and_random_continuation() {
        let mut state = state("kill|Rat|1|tail", 6_000);
        state.traits[3].subitems[0] = "1".to_owned();
        state.stats = vec![
            row("STR", "12"),
            row("CON", "11"),
            row("DEX", "10"),
            row("INT", "9"),
            row("WIS", "8"),
            row("CHA", "7"),
            row("HP Max", "20"),
            row("MP Max", "15"),
        ];
        state.spells.clear();
        state.bars.experience = bar(1_269, 1_269);
        let mut random = DelphiRandom::from_state(DesktopRandomState(0x1357_9bdf));
        let mut hooks = SourceDerivedDesktopHooks::untraced();

        hooks.level_up(&mut state, &mut random).unwrap();

        assert_eq!(state.traits[3].subitems[0], "2");
        assert_eq!(
            state
                .stats
                .iter()
                .map(|row| row.subitems[0].as_str())
                .collect::<Vec<_>>(),
            ["12", "11", "10", "10", "8", "7", "25", "23"]
        );
        assert_eq!(state.spells[0].caption, "Slime Finger");
        assert_eq!(state.spells[0].subitems[0], "I");
        assert_eq!(state.bars.experience, bar(0, 1_279));
        assert_eq!(random.state(), DesktopRandomState(592_146_772));
    }

    #[test]
    fn act_two_awards_only_an_item_and_later_acts_add_equipment() {
        let mut state = state("load", 2_000);
        state.traits[3].subitems[0] = "8".to_owned();
        state.inventory = vec![row("Gold", "0")];
        state.equipment = vec![
            row("Weapon", "Stick"),
            row("Shield", "Plate"),
            row("Helm", "Burlap"),
        ];
        state.plots = vec![row("Prologue", ""), row("Act I", "")];
        let original_equipment = state.equipment.clone();
        let mut random = DelphiRandom::from_state(DesktopRandomState(0x2468_ace0));
        let mut hooks = SourceDerivedDesktopHooks::untraced();

        hooks.complete_act(&mut state, &mut random).unwrap();
        assert_eq!(state.plots.last().unwrap().caption, "Act II");
        assert_eq!(state.plots[state.plots.len() - 2].header[1], 1);
        assert_eq!(state.bars.plot, bar(0, 39_600));
        assert_eq!(state.inventory.len(), 2);
        assert_eq!(state.equipment, original_equipment);

        hooks.complete_act(&mut state, &mut random).unwrap();
        assert_eq!(state.plots.last().unwrap().caption, "Act III");
        assert_eq!(state.bars.plot, bar(0, 57_600));
        assert_eq!(state.inventory.len(), 3);
        assert_ne!(state.equipment, original_equipment);
        assert!(state.prized_equipment < state.equipment.len());
    }

    #[test]
    fn ordered_specialty_and_prime_stat_ties_keep_the_earliest_row() {
        let spells = vec![row("Slime Finger", "II"), row("Rabbit Punch", "I")];
        assert_eq!(best_spell_index(&spells).unwrap(), Some(0));
        let stats = vec![
            row("STR", "12"),
            row("CON", "12"),
            row("DEX", "10"),
            row("INT", "9"),
            row("WIS", "8"),
            row("CHA", "7"),
        ];
        assert_eq!(best_prime_stat_index(&stats).unwrap(), 0);
    }

    #[test]
    fn captures_level_then_act_snapshots_without_changing_final_state() {
        let mut initial = state("kill|Rat|1|tail", 6_000);
        initial.traits[3].subitems[0] = "1".to_owned();
        initial.stats = vec![
            row("STR", "12"),
            row("CON", "11"),
            row("DEX", "10"),
            row("INT", "9"),
            row("WIS", "8"),
            row("CHA", "7"),
            row("HP Max", "20"),
            row("MP Max", "15"),
        ];
        initial.equipment = vec![
            row("Weapon", "Stick"),
            row("Shield", "Plate"),
            row("Helm", "Burlap"),
        ];
        initial.bars.experience = bar(1_269, 1_269);
        initial.queue = vec![DesktopQueueCommand {
            kind: DesktopQueueKind::Plot,
            duration_seconds: 2,
            caption: "Loading".to_owned(),
        }];
        let checkpoint = DesktopCallbackCheckpoint {
            state: initial,
            random: DesktopRandomState(0x1357_9bdf),
        };
        let mut traced_checkpoint = checkpoint.clone();
        let mut untraced_checkpoint = checkpoint;
        let mut traced = SourceDerivedDesktopHooks::traced();
        let mut untraced = SourceDerivedDesktopHooks::untraced();

        traced_checkpoint
            .apply_progression_callback(100, &mut traced)
            .unwrap();
        untraced_checkpoint
            .apply_progression_callback(100, &mut untraced)
            .unwrap();

        assert_eq!(traced_checkpoint, untraced_checkpoint);
        assert!(untraced.reports().is_empty());
        assert_eq!(
            traced
                .reports()
                .iter()
                .map(|snapshot| snapshot.trigger)
                .collect::<Vec<_>>(),
            [DesktopReportTrigger::Level, DesktopReportTrigger::Act]
        );

        let level = &traced.reports()[0].state;
        assert_eq!(level.traits[3].subitems[0], "2");
        assert_eq!(level.plots.len(), 2);
        assert_eq!(level.inventory.len(), 1);
        assert_eq!(level.queue.len(), 1);
        assert_eq!(level.bars.task, bar(6_000, 6_000));

        let act = &traced.reports()[1].state;
        assert_eq!(act.plots.last().unwrap().caption, "Act II");
        assert_eq!(act.queue.len(), 1);
        assert_eq!(act.bars.task, bar(6_000, 6_000));
        assert!(act.current_task.is_empty());

        assert!(traced_checkpoint.state.queue.is_empty());
        assert_eq!(traced_checkpoint.state.bars.task, bar(0, 2_000));
        assert_eq!(traced_checkpoint.state.activity, "Loading Act II...");
    }
}
