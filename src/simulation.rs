//! Pure, deterministic Progress Quest state advancement.
//!
//! This module has no filesystem, clock, database, or HTTP dependencies: it
//! consumes an immutable canonical [`Character`] state, an explicit
//! [`Ruleset`], and a caller-supplied elapsed-millisecond duration, and
//! returns a new canonical state. It never reads a wall clock and never
//! mutates the source save document (see `openspec/changes/
//! establish-deterministic-simulation/design.md`).

use thiserror::Error;

use crate::rng::Alea;
use crate::ruleset::Ruleset;
use crate::state::{Character, InventoryEntry, ProgressBarKind, Spell};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SimulationError {
    /// A state transition would require browser behavior not yet ported.
    /// Rather than fabricate an effect, advancement stops and reports which
    /// unported behavior family it needed (see tasks 3.2/3.3).
    #[error("advancement requires unported behavior: {0}")]
    Unsupported(&'static str),
}

/// The largest single browser-compatible timer tick, in milliseconds. The
/// browser client's `Timer1Timer` caps each real-clock increment to 100 ms
/// before applying it to the active task's progress bar.
const MAX_TICK_MS: u64 = 100;

/// Advances `state` by `elapsed_ms` of caller-supplied (never wall-clock) time
/// and returns the resulting state.
///
/// This reproduces the browser client's `Timer1Timer` loop: while the active
/// task's bar is incomplete, elapsed time is split into ticks capped at
/// [`MAX_TICK_MS`] and applied to the task bar alone; once the bar is full,
/// completion is dispatched (task queue selection, per [`dispatch_completion`])
/// before any further ticking, exactly as the browser processes a completion
/// callback with no elapsed-time cost of its own.
pub fn advance(
    state: &Character,
    ruleset: &Ruleset,
    elapsed_ms: u64,
) -> Result<Character, SimulationError> {
    let mut next = state.clone();
    let mut remaining = elapsed_ms;
    loop {
        if next.progress.task.done() {
            dispatch_completion(&mut next, ruleset)?;
            continue;
        }
        if remaining == 0 {
            break;
        }
        let tick = remaining.min(MAX_TICK_MS);
        next.progress
            .task
            .increment(ProgressBarKind::Task, tick as f64);
        remaining -= tick;
    }
    Ok(next)
}

/// Ports the browser `Timer1Timer`'s `TaskBar.done()` branch: task-count and
/// elapsed-time bookkeeping, the experience/quest/plot "gain" advancement,
/// and `Dequeue()`'s task-queue selection. `ClearAllSelections()` only
/// touches transient UI selection state with no canonical representation, so
/// it has nothing to port.
fn dispatch_completion(state: &mut Character, ruleset: &Ruleset) -> Result<(), SimulationError> {
    let mut rng = Alea::from_state(state.seed);
    let task_max = state.progress.task.max;
    state.activity.tasks += 1;
    state.activity.elapsed += task_max / 1000;

    if state.activity.kill == "Loading...." {
        state.progress.task.reset(ProgressBarKind::Task, 0, 0.0);
    }

    // `gain` mirrors the browser's `Pos('kill|', game.task) == 1`: the task
    // just completed was a monster-kill task.
    let gain = state.activity.task.starts_with("kill|");
    let delta = (task_max / 1000) as f64;

    if gain {
        if state.progress.experience.done() {
            level_up(state, ruleset, &mut rng);
        } else {
            state
                .progress
                .experience
                .increment(ProgressBarKind::Experience, delta);
        }
    }

    if gain && state.plot.act >= 1 {
        if state.progress.quest.done() || state.quests.is_empty() {
            complete_quest(state, ruleset, &mut rng);
        } else {
            state
                .progress
                .quest
                .increment(ProgressBarKind::Quest, delta);
        }
    }

    if gain || state.plot.act == 0 {
        if state.progress.plot.done() {
            interplot_cinematic(state, ruleset, &mut rng)?;
        } else {
            state.progress.plot.increment(ProgressBarKind::Plot, delta);
        }
    }

    dequeue(state, ruleset, &mut rng)?;
    state.seed = rng.state();
    Ok(())
}

/// Ports the browser `Dequeue()`. Kill-task loot and subsequent monster-task
/// selection use the persisted Alea continuation in `state.seed`.
fn dequeue(
    state: &mut Character,
    ruleset: &Ruleset,
    rng: &mut Alea,
) -> Result<(), SimulationError> {
    while state.progress.task.done() {
        if split(&state.activity.task, 0) == "kill" {
            resolve_kill_loot(state, ruleset, rng);
        } else if state.activity.task == "buying" {
            subtract_inventory(state, "Gold", equipment_price(state));
            win_equip(state, ruleset, rng);
        } else if state.activity.task == "market" || state.activity.task == "sell" {
            if state.activity.task == "sell" {
                sell_inventory(state, rng);
            }
            if state.inventory.len() > 1 {
                let item = &state.inventory[1];
                let caption = format!("Selling {}", indefinite(&item.name, item.quantity as i64));
                set_task(state, &caption, 1_000);
                state.activity.task = "sell".to_owned();
                break;
            }
        }

        let old = std::mem::take(&mut state.activity.task);

        if let Some(entry) = state.queue.first().cloned() {
            let kind = split(&entry, 0);
            let duration_s: u64 = split(&entry, 1).parse().unwrap_or(0);
            match kind {
                "task" => {
                    let caption = split(&entry, 2).to_owned();
                    state.queue.remove(0);
                    set_task(state, &caption, duration_s * 1000);
                }
                "plot" => {
                    state.queue.remove(0);
                    complete_act(state, ruleset, rng);
                    set_task(
                        state,
                        &format!("Loading {}", state.plot.bestplot),
                        duration_s * 1000,
                    );
                }
                _ => {
                    return Err(SimulationError::Unsupported(
                        "unrecognized queue entry kind",
                    ));
                }
            }
        } else if state.progress.encumbrance.done() {
            set_task(state, "Heading to market to sell loot", 4_000);
            state.activity.task = "market".to_owned();
        } else if !old.contains("kill|") && old != "heading" {
            if inventory_quantity(state, "Gold") > equipment_price(state) {
                set_task(state, "Negotiating purchase of better equipment", 5_000);
                state.activity.task = "buying".to_owned();
            } else {
                set_task(state, "Heading to the killing fields", 4_000);
                state.activity.task = "heading".to_owned();
            }
        } else {
            let monster = monster_task(state, ruleset, rng);
            let duration_ms = (2 * 3 * monster.level * 1000)
                .checked_div(state.traits.level)
                .unwrap_or(0);
            set_task(
                state,
                &format!("Executing {}", monster.description),
                duration_ms,
            );
        }
    }
    Ok(())
}

fn resolve_kill_loot(state: &mut Character, ruleset: &Ruleset, rng: &mut Alea) {
    let monster = split(&state.activity.task, 1);
    let body_part = split(&state.activity.task, 3);
    if body_part == "*" {
        win_item(state, ruleset, rng);
    } else if !body_part.is_empty() {
        add_inventory(
            state,
            &format!("{monster} {}", proper_case(body_part)).to_lowercase(),
            1,
        );
    }
}

fn win_item(state: &mut Character, ruleset: &Ruleset, rng: &mut Alea) {
    if rng.bounded(999).max(250.0) < state.inventory.len() as f64 {
        let index = random_index(rng, state.inventory.len());
        let name = state.inventory[index].name.clone();
        add_inventory(state, &name, 1);
    } else {
        let attribute = pick(ruleset.item_attrib, rng);
        let special = pick(ruleset.specials, rng);
        let of = pick(ruleset.item_ofs, rng);
        add_inventory(state, &format!("{attribute} {special} of {of}"), 1);
    }
}

fn add_inventory(state: &mut Character, name: &str, quantity: u64) {
    if let Some(item) = state.inventory.iter_mut().find(|item| item.name == name) {
        item.quantity += quantity;
    } else {
        state.inventory.push(InventoryEntry {
            name: name.to_owned(),
            quantity,
        });
    }

    let cubits = state
        .inventory
        .iter()
        .skip(1)
        .map(|item| item.quantity)
        .sum::<u64>();
    state
        .progress
        .encumbrance
        .reposition(ProgressBarKind::Encumbrance, cubits as f64);
}

fn subtract_inventory(state: &mut Character, name: &str, quantity: u64) {
    let item = state
        .inventory
        .iter_mut()
        .find(|item| item.name == name)
        .expect("canonical inventory always contains Gold");
    item.quantity = item
        .quantity
        .checked_sub(quantity)
        .expect("browser Gold is sufficient");
}

fn inventory_quantity(state: &Character, name: &str) -> u64 {
    state
        .inventory
        .iter()
        .find(|item| item.name == name)
        .map_or(0, |item| item.quantity)
}

fn equipment_price(state: &Character) -> u64 {
    5 * state.traits.level * state.traits.level + 10 * state.traits.level + 20
}

fn sell_inventory(state: &mut Character, rng: &mut Alea) {
    let item = state.inventory[1].clone();
    let mut amount = item.quantity * state.traits.level;
    if item.name.contains(" of ") {
        amount *= 1 + random_low(rng, 10);
        amount *= 1 + random_low(rng, state.traits.level as u32);
    }
    state.inventory.remove(1);
    add_inventory(state, "Gold", amount);
}

fn level_up(state: &mut Character, ruleset: &Ruleset, rng: &mut Alea) {
    state.traits.level += 1;
    let hit_points = (stat_integer(state, "CON") / 3 + 1) as f64 + rng.bounded(4);
    let mana_points = (stat_integer(state, "INT") / 3 + 1) as f64 + rng.bounded(4);
    add_stat(state, "HP Max", hit_points);
    add_stat(state, "MP Max", mana_points);
    win_stat(state, ruleset, rng);
    win_stat(state, ruleset, rng);
    win_spell(state, ruleset, rng);
    let seconds = ((ruleset.level_up_base_minutes as f64
        + (ruleset.level_up_growth_numerator as f64 / ruleset.level_up_growth_denominator as f64)
            .powf(state.traits.level as f64))
        * 60.0)
        .round() as u64;
    state
        .progress
        .experience
        .reset(ProgressBarKind::Experience, seconds, 0.0);
}

fn add_stat(state: &mut Character, name: &str, value: f64) {
    match name {
        "STR" => {
            state.stats.strength += value;
            state.progress.encumbrance.reset(
                ProgressBarKind::Encumbrance,
                (10.0 + state.stats.strength) as u64,
                state.progress.encumbrance.position,
            );
        }
        "CON" => state.stats.constitution += value,
        "DEX" => state.stats.dexterity += value,
        "INT" => state.stats.intelligence += value,
        "WIS" => state.stats.wisdom += value,
        "CHA" => state.stats.charisma += value,
        "HP Max" => state.stats.hit_points_max += value,
        "MP Max" => state.stats.mana_points_max += value,
        _ => unreachable!("browser ruleset only contains known stats"),
    }
}

fn stat_value(state: &Character, name: &str) -> f64 {
    match name {
        "STR" => state.stats.strength,
        "CON" => state.stats.constitution,
        "DEX" => state.stats.dexterity,
        "INT" => state.stats.intelligence,
        "WIS" => state.stats.wisdom,
        "CHA" => state.stats.charisma,
        "HP Max" => state.stats.hit_points_max,
        "MP Max" => state.stats.mana_points_max,
        _ => unreachable!("browser ruleset only contains known stats"),
    }
}

fn stat_integer(state: &Character, name: &str) -> u64 {
    stat_value(state, name).trunc() as u64
}

fn win_stat(state: &mut Character, ruleset: &Ruleset, rng: &mut Alea) {
    let stat = if odds(rng, 1, 2) {
        pick(ruleset.stats, rng)
    } else {
        let mut total = ruleset
            .prime_stats
            .iter()
            .map(|stat| stat_integer(state, stat).pow(2))
            .sum::<u64>();
        total = rng.bounded(total as u32) as u64;
        let mut selected = ruleset.prime_stats[0];
        for stat in ruleset.prime_stats {
            selected = stat;
            let weight = stat_integer(state, stat).pow(2);
            if total < weight {
                break;
            }
            total -= weight;
        }
        selected
    };
    add_stat(state, stat, 1.0);
}

fn win_spell(state: &mut Character, ruleset: &Ruleset, rng: &mut Alea) {
    let limit = (stat_integer(state, "WIS") + state.traits.level).min(ruleset.spells.len() as u64);
    let spell = pick_low(&ruleset.spells[..limit as usize], rng);
    if let Some(entry) = state.spells.iter_mut().find(|entry| entry.name == spell) {
        entry.rank = to_roman(to_arabic(&entry.rank) + 1);
    } else {
        state.spells.push(Spell {
            name: spell.to_owned(),
            rank: "I".to_owned(),
        });
    }
}

fn win_equip(state: &mut Character, ruleset: &Ruleset, rng: &mut Alea) {
    let position = random_index(rng, ruleset.equips.len());
    let (stuff, better, worse) = if position == 0 {
        (ruleset.weapons, ruleset.offense_attrib, ruleset.offense_bad)
    } else {
        (
            if position == 1 {
                ruleset.shields
            } else {
                ruleset.armors
            },
            ruleset.defense_attrib,
            ruleset.defense_bad,
        )
    };
    let mut name = lpick(stuff, state.traits.level, rng).to_owned();
    let mut quality = split(&name, 1).parse::<i64>().unwrap();
    name = split(&name, 0).to_owned();
    let mut plus = state.traits.level as i64 - quality;
    let modifiers = if plus < 0 { worse } else { better };
    let mut count = 0;
    while count < 2 && plus != 0 {
        let modifier = pick(modifiers, rng);
        quality = split(modifier, 1).parse().unwrap();
        let modifier = split(modifier, 0);
        if name.contains(modifier) || plus.abs() < quality.abs() {
            break;
        }
        name = format!("{modifier} {name}");
        plus -= quality;
        count += 1;
    }
    if plus != 0 {
        name = format!("{plus} {name}");
    }
    if plus > 0 {
        name = format!("+{name}");
    }
    *equipment_at_mut(state, position) = name.clone();
    state.bestequip = name;
    if position > 1 {
        state.bestequip.push(' ');
        state.bestequip.push_str(ruleset.equips[position]);
    }
}

fn equipment_at_mut(state: &mut Character, position: usize) -> &mut String {
    match position {
        0 => &mut state.equipment.weapon,
        1 => &mut state.equipment.shield,
        2 => &mut state.equipment.helm,
        3 => &mut state.equipment.hauberk,
        4 => &mut state.equipment.brassairts,
        5 => &mut state.equipment.vambraces,
        6 => &mut state.equipment.gauntlets,
        7 => &mut state.equipment.gambeson,
        8 => &mut state.equipment.cuisses,
        9 => &mut state.equipment.greaves,
        10 => &mut state.equipment.sollerets,
        _ => unreachable!("browser ruleset has exactly eleven equipment slots"),
    }
}

fn complete_quest(state: &mut Character, ruleset: &Ruleset, rng: &mut Alea) {
    state
        .progress
        .quest
        .reset(ProgressBarKind::Quest, 50 + rng.bounded(100) as u64, 0.0);
    if !state.quests.is_empty() {
        match random_index(rng, 4) {
            0 => win_spell(state, ruleset, rng),
            1 => win_equip(state, ruleset, rng),
            2 => win_stat(state, ruleset, rng),
            3 => win_item(state, ruleset, rng),
            _ => unreachable!(),
        }
    }
    while state.quests.len() > 99 {
        state.quests.remove(0);
    }

    state.activity.questmonster.clear();
    let caption = match random_index(rng, 5) {
        0 => {
            let monster = closest_monster(ruleset, state.traits.level as i64, 4, rng);
            state.activity.questmonster = monster.0.to_owned();
            state.activity.questmonsterindex = monster.1 as u64;
            format!("Exterminate {}", definite(split(monster.0, 0), 2))
        }
        1 => format!("Seek {}", definite(&interesting_item(ruleset, rng), 1)),
        2 => format!("Deliver this {}", boring_item(ruleset, rng)),
        3 => format!("Fetch me {}", indefinite(boring_item(ruleset, rng), 1)),
        4 => {
            let monster = closest_monster(ruleset, state.traits.level as i64, 2, rng);
            state.activity.questmonster = monster.0.to_owned();
            state.activity.questmonster.clear();
            format!("Placate {}", definite(split(monster.0, 0), 2))
        }
        _ => unreachable!(),
    };
    while state.quests.len() > 99 {
        state.quests.remove(0);
    }
    state.quests.push(caption.clone());
    state.bestquest = caption;
}

fn closest_monster<'a>(
    ruleset: &'a Ruleset,
    level: i64,
    attempts: usize,
    rng: &mut Alea,
) -> (&'a str, usize) {
    let first = random_index(rng, ruleset.monsters.len());
    let mut result = (ruleset.monsters[first], first);
    let mut best_level = split(result.0, 1).parse::<i64>().unwrap();
    for attempt in 1..attempts {
        let index = random_index(rng, ruleset.monsters.len());
        let candidate = ruleset.monsters[index];
        let candidate_level = split(candidate, 1).parse::<i64>().unwrap();
        if (candidate_level - level).abs() < (best_level - level).abs() {
            result = (candidate, index);
            best_level = candidate_level;
        }
        debug_assert!(attempt < attempts);
    }
    result
}

fn complete_act(state: &mut Character, ruleset: &Ruleset, rng: &mut Alea) {
    state.plot.act += 1;
    state.progress.plot.reset(
        ProgressBarKind::Plot,
        60 * 60 * (1 + 5 * state.plot.act),
        0.0,
    );
    state.plot.bestplot = format!("Act {}", to_roman(state.plot.act));
    if state.plot.act > 1 {
        win_item(state, ruleset, rng);
        win_equip(state, ruleset, rng);
    }
}

fn interplot_cinematic(
    state: &mut Character,
    ruleset: &Ruleset,
    rng: &mut Alea,
) -> Result<(), SimulationError> {
    match random_index(rng, 3) {
        0 => {
            enqueue(
                state,
                "task|1|Exhausted, you arrive at a friendly oasis in a hostile land",
                ruleset,
                rng,
            )?;
            enqueue(
                state,
                "task|2|You greet old friends and meet new allies",
                ruleset,
                rng,
            )?;
            enqueue(
                state,
                "task|2|You are privy to a council of powerful do-gooders",
                ruleset,
                rng,
            )?;
            enqueue(
                state,
                "task|1|There is much to be done. You are chosen!",
                ruleset,
                rng,
            )?;
        }
        1 => {
            enqueue(
                state,
                "task|1|Your quarry is in sight, but a mighty enemy bars your path!",
                ruleset,
                rng,
            )?;
            let nemesis = named_monster(ruleset, state.traits.level + 3, rng);
            enqueue(
                state,
                &format!("task|4|A desperate struggle commences with {nemesis}"),
                ruleset,
                rng,
            )?;
            let mut sequence = random_index(rng, 3);
            for _ in 1..=random_index(rng, (state.plot.act + 2) as usize) {
                sequence += 1 + random_index(rng, 2);
                let caption = match sequence % 3 {
                    0 => format!("Locked in grim combat with {nemesis}"),
                    1 => format!("{nemesis} seems to have the upper hand"),
                    2 => format!("You seem to gain the advantage over {nemesis}"),
                    _ => unreachable!(),
                };
                enqueue(state, &format!("task|2|{caption}"), ruleset, rng)?;
            }
            enqueue(
                state,
                &format!("task|3|Victory! {nemesis} is slain! Exhausted, you lose consciousness"),
                ruleset,
                rng,
            )?;
            enqueue(
                state,
                "task|2|You awake in a friendly place, but the road awaits",
                ruleset,
                rng,
            )?;
        }
        2 => {
            let nemesis = impressive_guy(ruleset, rng);
            enqueue(
                state,
                &format!("task|2|Oh sweet relief! You've reached the kind protection of {nemesis}"),
                ruleset,
                rng,
            )?;
            enqueue(
                state,
                &format!(
                    "task|3|There is rejoicing, and an unnerving encounter with {nemesis} in private"
                ),
                ruleset,
                rng,
            )?;
            enqueue(
                state,
                &format!(
                    "task|2|You forget your {} and go back to get it",
                    boring_item(ruleset, rng)
                ),
                ruleset,
                rng,
            )?;
            enqueue(
                state,
                "task|2|What's this!? You overhear something shocking!",
                ruleset,
                rng,
            )?;
            enqueue(
                state,
                &format!("task|2|Could {nemesis} be a dirty double-dealer?"),
                ruleset,
                rng,
            )?;
            enqueue(
                state,
                "task|3|Who can possibly be trusted with this news!? -- Oh yes, of course",
                ruleset,
                rng,
            )?;
        }
        _ => unreachable!(),
    }
    enqueue(state, "plot|1|Loading", ruleset, rng)
}

fn enqueue(
    state: &mut Character,
    entry: &str,
    ruleset: &Ruleset,
    rng: &mut Alea,
) -> Result<(), SimulationError> {
    state.queue.push(entry.to_owned());
    dequeue(state, ruleset, rng)
}

fn named_monster(ruleset: &Ruleset, level: u64, rng: &mut Alea) -> String {
    let mut result = "";
    let mut closest_level = 0_i64;
    for _ in 0..5 {
        let monster = pick(ruleset.monsters, rng);
        let monster_level = split(monster, 1).parse::<i64>().unwrap();
        if result.is_empty()
            || (level as i64 - monster_level).abs() < (level as i64 - closest_level).abs()
        {
            result = split(monster, 0);
            closest_level = monster_level;
        }
    }
    format!("{} the {result}", generate_name(ruleset, rng))
}

fn impressive_guy(ruleset: &Ruleset, rng: &mut Alea) -> String {
    if random_index(rng, 2) != 0 {
        format!(
            "the {} of the {}",
            pick(ruleset.impressive_titles, rng),
            plural(split(pick(ruleset.races, rng), 0))
        )
    } else {
        format!(
            "{} {} of {}",
            pick(ruleset.impressive_titles, rng),
            generate_name(ruleset, rng),
            generate_name(ruleset, rng)
        )
    }
}

struct MonsterTask {
    description: String,
    level: u64,
}

fn monster_task(state: &mut Character, ruleset: &Ruleset, rng: &mut Alea) -> MonsterTask {
    let mut level = state.traits.level as i64;
    for _ in 1..=state.traits.level {
        if odds(rng, 2, 5) {
            level += rand_sign(rng);
        }
    }
    level = level.max(1);

    let (monster, monster_level, definite) = if odds(rng, 1, 25) {
        let race = split(pick(ruleset.races, rng), 0);
        let description = if odds(rng, 1, 2) {
            format!("passing {race} {}", split(pick(ruleset.klasses, rng), 0))
        } else {
            format!(
                "{} {} the {race}",
                pick_low(ruleset.titles, rng),
                generate_name(ruleset, rng)
            )
        };
        (
            format!("{description}|{level}|*"),
            level,
            !description.starts_with("passing "),
        )
    } else if !state.activity.questmonster.is_empty() && odds(rng, 1, 4) {
        let monster = ruleset.monsters[state.activity.questmonsterindex as usize].to_owned();
        let monster_level = split(&monster, 1).parse().unwrap_or(0);
        (monster, monster_level, false)
    } else {
        let mut monster = pick(ruleset.monsters, rng).to_owned();
        let mut monster_level = split(&monster, 1).parse().unwrap_or(0);
        for _ in 0..5 {
            let candidate = pick(ruleset.monsters, rng);
            let candidate_level = split(candidate, 1).parse().unwrap_or(0);
            if (level - candidate_level).abs() < (level - monster_level).abs() {
                monster = candidate.to_owned();
                monster_level = candidate_level;
            }
        }
        (monster, monster_level, false)
    };

    let mut description = split(&monster, 0).to_owned();
    let mut quantity = 1_i64;
    if level - monster_level > 10 {
        quantity = ((level as f64 + rng.bounded(monster_level.max(1) as u32))
            / monster_level.max(1) as f64)
            .floor() as i64;
        quantity = quantity.max(1);
        level = (level as f64 / quantity as f64).floor() as i64;
    }
    // The remaining descriptive branches consume randomness only when the
    // generated opponent differs materially from the character's level.
    description = describe_monster(description, level, monster_level, rng);
    let combat_level = level * quantity;
    if !definite {
        description = indefinite(&description, quantity);
    }
    state.activity.task = format!("kill|{monster}");
    MonsterTask {
        description,
        level: combat_level as u64,
    }
}

fn describe_monster(description: String, level: i64, monster_level: i64, rng: &mut Alea) -> String {
    let delta = level - monster_level;
    if delta <= -10 {
        format!("imaginary {description}")
    } else if delta < -5 {
        let i = 5 - rng.bounded((10 + delta + 1) as u32) as i64;
        prefix(
            &["dead", "comatose", "crippled", "sick", "undernourished"],
            6 - i,
            &prefix(
                &["foetal", "baby", "preadolescent", "teenage", "underage"],
                6 - ((-delta) - i),
                &description,
                " ",
            ),
            " ",
        )
    } else if delta < 0 && rng.bounded(2) == 1.0 {
        prefix(
            &["dead", "comatose", "crippled", "sick", "undernourished"],
            6 - (-delta),
            &description,
            " ",
        )
    } else if delta < 0 {
        prefix(
            &["foetal", "baby", "preadolescent", "teenage", "underage"],
            6 - (-delta),
            &description,
            " ",
        )
    } else if delta >= 10 {
        format!("messianic {description}")
    } else if delta > 5 {
        let i = 5 - rng.bounded((10 - delta + 1) as u32) as i64;
        let special = if description.contains(' ') {
            prefix(
                &["veteran", "cursed", "warrior", "undead", "demon"],
                delta - i,
                &description,
                " ",
            )
        } else {
            prefix(
                &["Battle-", "cursed ", "Were-", "undead ", "demon "],
                delta - i,
                &description,
                "",
            )
        };
        prefix(
            &["greater", "massive", "enormous", "giant", "titanic"],
            i,
            &special,
            " ",
        )
    } else if delta > 0 && rng.bounded(2) == 1.0 {
        prefix(
            &["greater", "massive", "enormous", "giant", "titanic"],
            delta,
            &description,
            " ",
        )
    } else if delta > 0 {
        if description.contains(' ') {
            prefix(
                &["veteran", "cursed", "warrior", "undead", "demon"],
                delta,
                &description,
                " ",
            )
        } else {
            prefix(
                &["Battle-", "cursed ", "Were-", "undead ", "demon "],
                delta,
                &description,
                "",
            )
        }
    } else {
        description
    }
}

fn odds(rng: &mut Alea, chance: u32, out_of: u32) -> bool {
    rng.bounded(out_of) < chance as f64
}

fn rand_sign(rng: &mut Alea) -> i64 {
    rng.bounded(2) as i64 * 2 - 1
}

fn random_index(rng: &mut Alea, length: usize) -> usize {
    rng.bounded(length as u32) as usize
}

fn random_low(rng: &mut Alea, upper_bound: u32) -> u64 {
    (rng.bounded(upper_bound) as u64).min(rng.bounded(upper_bound) as u64)
}

fn pick<'a>(values: &'a [&'a str], rng: &mut Alea) -> &'a str {
    values[random_index(rng, values.len())]
}

fn pick_low<'a>(values: &'a [&'a str], rng: &mut Alea) -> &'a str {
    let index = random_index(rng, values.len()).min(random_index(rng, values.len()));
    values[index]
}

fn lpick<'a>(values: &'a [&'a str], goal: u64, rng: &mut Alea) -> &'a str {
    let mut result = pick(values, rng);
    for _ in 1..=5 {
        let best = split(result, 1).parse::<i64>().unwrap();
        let candidate = pick(values, rng);
        let candidate_quality = split(candidate, 1).parse::<i64>().unwrap();
        if (goal as i64 - best).abs() > (goal as i64 - candidate_quality).abs() {
            result = candidate;
        }
    }
    result
}

fn plural(value: &str) -> String {
    if let Some(stem) = value.strip_suffix('y') {
        format!("{stem}ies")
    } else if let Some(stem) = value.strip_suffix("us") {
        format!("{stem}i")
    } else if value.ends_with("ch")
        || value.ends_with('x')
        || value.ends_with('s')
        || value.ends_with("sh")
    {
        format!("{value}es")
    } else if let Some(stem) = value.strip_suffix('f') {
        format!("{stem}ves")
    } else if let Some(stem) = value.strip_suffix("man") {
        format!("{stem}men")
    } else if let Some(stem) = value.strip_suffix("Man") {
        format!("{stem}Men")
    } else {
        format!("{value}s")
    }
}

fn definite(value: &str, quantity: i64) -> String {
    let value = if quantity > 1 {
        plural(value)
    } else {
        value.to_owned()
    };
    format!("the {value}")
}

fn interesting_item(ruleset: &Ruleset, rng: &mut Alea) -> String {
    format!(
        "{} {}",
        pick(ruleset.item_attrib, rng),
        pick(ruleset.specials, rng)
    )
}

fn boring_item<'a>(ruleset: &'a Ruleset, rng: &mut Alea) -> &'a str {
    pick(ruleset.boring_items, rng)
}

fn to_roman(mut number: u64) -> String {
    let numerals = [
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
    ];
    let mut result = String::new();
    for (value, numeral) in numerals {
        while number >= value {
            number -= value;
            result.push_str(numeral);
        }
    }
    result
}

fn to_arabic(value: &str) -> u64 {
    let mut value = value.to_uppercase();
    let numerals = [
        ("MT", 9_000),
        ("MA", 4_000),
        ("CM", 900),
        ("CD", 400),
        ("XC", 90),
        ("XL", 40),
        ("IX", 9),
        ("IV", 4),
        ("T", 10_000),
        ("A", 5_000),
        ("M", 1_000),
        ("D", 500),
        ("C", 100),
        ("L", 50),
        ("X", 10),
        ("V", 5),
        ("I", 1),
    ];
    let mut result = 0;
    while !value.is_empty() {
        let (prefix, amount) = numerals
            .iter()
            .find(|(prefix, _)| value.starts_with(prefix))
            .expect("canonical spell ranks are browser Roman numerals");
        value = value[prefix.len()..].to_owned();
        result += amount;
    }
    result
}

fn generate_name(ruleset: &Ruleset, rng: &mut Alea) -> String {
    let mut name = String::new();
    for index in 0..=5 {
        name.push_str(pick(ruleset.name_parts[index % 3], rng));
    }
    let mut characters = name.chars();
    match characters.next() {
        Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
        None => name,
    }
}

fn prefix(parts: &[&str], amount: i64, suffix: &str, separator: &str) -> String {
    let index = amount.unsigned_abs() as usize;
    if index == 0 || index > parts.len() {
        suffix.to_owned()
    } else {
        format!("{}{}{}", parts[index - 1], separator, suffix)
    }
}

fn indefinite(name: &str, quantity: i64) -> String {
    if quantity == 1 {
        if matches!(
            name.chars().next(),
            Some('A' | 'E' | 'I' | 'O' | 'U' | 'Ü' | 'a' | 'e' | 'i' | 'o' | 'u' | 'ü')
        ) {
            format!("an {name}")
        } else {
            format!("a {name}")
        }
    } else {
        format!("{quantity} {name}s")
    }
}

fn proper_case(value: &str) -> String {
    let mut characters = value.chars();
    match characters.next() {
        Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
        None => String::new(),
    }
}

/// Ports the browser `Task(caption, msec)`: sets the activity status text
/// (with the browser's trailing `"..."`) and resets the task bar.
fn set_task(state: &mut Character, caption: &str, duration_ms: u64) {
    state.activity.kill = format!("{caption}...");
    state
        .progress
        .task
        .reset(ProgressBarKind::Task, duration_ms, 0.0);
}

/// Ports the browser `Split(s, index)`: splits `s` on `|` and returns the
/// 0-based `index`th field, or `""` if the field is absent.
fn split(s: &str, index: usize) -> &str {
    s.split('|').nth(index).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use base64::{Engine, engine::general_purpose::STANDARD};

    use super::*;
    use crate::save::import_text;

    fn character() -> Character {
        import_text(&STANDARD.encode(include_str!("../tests/fixtures/reference-save.json")))
            .unwrap()
    }

    #[test]
    fn incomplete_advancement_only_moves_the_task_bar() {
        let before = character();
        let ruleset = Ruleset::default();
        let after = advance(&before, &ruleset, 250).unwrap();

        assert_eq!(
            after.progress.task.position,
            before.progress.task.position + 250.0
        );
        assert!(!after.progress.task.done());
        assert_eq!(after.activity.tasks, before.activity.tasks);
        assert_eq!(after.activity.elapsed, before.activity.elapsed);
        assert_eq!(
            after.progress.experience.position,
            before.progress.experience.position
        );
        assert_eq!(after.stats.seed.0, before.stats.seed.0);
        assert_eq!(after.dna.0, before.dna.0);
        assert_eq!(after.seed.0, before.seed.0);
        assert_eq!(after.inventory.len(), before.inventory.len());
    }

    #[test]
    fn advancement_is_pure_and_has_no_side_effects() {
        // No filesystem, clock, database, or HTTP access is reachable from
        // this module: `advance` only touches its `Character`/`Ruleset`
        // arguments and returns a new value.
        let before = character();
        let ruleset = Ruleset::default();
        let _ = advance(&before, &ruleset, 40).unwrap();
        // The input state itself must remain untouched (immutable input).
        assert_eq!(before.progress.task.position, before.progress.task.position);
    }

    #[test]
    fn ticks_are_capped_at_one_hundred_milliseconds() {
        let mut before = character();
        before.progress.task.max = 1_000;
        before.progress.task.position = 0.0;
        let ruleset = Ruleset::default();

        // A single 250 ms request must not move the bar by more than three
        // capped 100 ms ticks (300 ms would be wrong; here it's exactly 250
        // because ticks are only capped, not padded).
        let after = advance(&before, &ruleset, 250).unwrap();
        assert_eq!(after.progress.task.position, 250.0);
    }

    #[test]
    fn completing_a_sell_task_pays_for_and_removes_the_item() {
        // The reference fixture's active task is "sell"; its only non-Gold
        // inventory entry is worth quantity × level.
        let mut before = character();
        before.progress.task.max = 100;
        before.progress.task.position = 0.0;
        let ruleset = Ruleset::default();

        let after = advance(&before, &ruleset, 100).unwrap();
        assert_eq!(inventory_quantity(&after, "Gold"), 11);
        assert_eq!(after.inventory.len(), 1);
        assert_eq!(after.activity.task, "heading");
        assert_eq!(after.activity.kill, "Heading to the killing fields...");
    }

    #[test]
    fn completing_a_kill_task_awards_loot_and_selects_a_new_monster_task() {
        let mut before = character();
        // Browser-derived disposable-session checkpoint: an exact non-leveling
        // Goblin kill followed by `Dequeue()`'s next-monster selection.
        before.traits.level = 1;
        before.seed = crate::rng::AleaState([0.1, 0.2, 0.3, 1.0]);
        before.activity.task = "kill|Goblin|1|ear".to_owned();
        before.activity.tasks = 7;
        before.activity.elapsed = 42;
        before.activity.kill = "Executing a Goblin...".to_owned();
        before.queue.clear();
        before.inventory.truncate(1);
        before.inventory[0] = InventoryEntry {
            name: "Gold".to_owned(),
            quantity: 0,
        };
        before
            .progress
            .experience
            .reset(ProgressBarKind::Experience, 1_000, 0.0);
        before.progress.plot.reset(ProgressBarKind::Plot, 100, 0.0);
        before
            .progress
            .encumbrance
            .reset(ProgressBarKind::Encumbrance, 19, 0.0);
        before.progress.task.max = 1_000;
        before.progress.task.position = 0.0;
        let ruleset = Ruleset::default();

        let after = advance(&before, &ruleset, 1_000).unwrap();
        assert!(
            after
                .inventory
                .iter()
                .any(|item| item.name == "goblin ear" && item.quantity == 1)
        );
        assert_eq!(after.activity.task, "kill|Mrs. Brambrig the Skraeling|1|*");
        assert_eq!(
            after.activity.kill,
            "Executing Mrs. Brambrig the Skraeling..."
        );
        assert_eq!(after.activity.tasks, 8);
        assert_eq!(after.activity.elapsed, 43);
        assert_eq!(after.progress.experience.position, 1.0);
        assert_eq!(after.progress.plot.position, 1.0);
        assert_eq!(after.progress.encumbrance.position, 1.0);
        assert_eq!(
            after.seed,
            crate::rng::AleaState([
                0.8111408953554928,
                0.7811518528033048,
                0.9740190447773784,
                1_822_180.0,
            ])
        );
    }
}
