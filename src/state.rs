use serde::{Deserialize, Deserializer, Serialize, de};
use serde_json::{Map, Value};

use crate::rng::AleaState;
use crate::save::SaveError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    #[serde(skip_serializing, default)]
    pub document: Value,
    #[serde(rename = "Traits")]
    pub traits: Traits,
    pub dna: AleaState,
    pub seed: AleaState,
    pub birthday: String,
    pub birthstamp: u64,
    #[serde(rename = "Stats")]
    pub stats: Attributes,
    pub beststat: String,
    pub activity: Activity,
    pub bestequip: String,
    #[serde(rename = "Equips")]
    pub equipment: Equipment,
    #[serde(rename = "Inventory")]
    pub inventory: Vec<InventoryEntry>,
    #[serde(rename = "Spells")]
    pub spells: Vec<Spell>,
    pub plot: Plot,
    #[serde(rename = "Quests")]
    pub quests: Vec<String>,
    pub progress: Progress,
    pub queue: Vec<String>,
    pub date: String,
    pub stamp: u64,
    pub online: Option<OnlineMetadata>,
    #[serde(default)]
    pub profile: OnlineProfile,
    pub save_name: String,
    pub bestspell: String,
    pub bestquest: String,
}
fn number(object: &Map<String, Value>, key: &str, field: &str) -> Result<f64, SaveError> {
    object
        .get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| SaveError::invalid(field))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Traits {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Race")]
    pub race: String,
    #[serde(rename = "Class")]
    pub class: String,
    #[serde(rename = "Level")]
    pub level: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attributes {
    pub seed: AleaState,
    #[serde(rename = "STR")]
    pub strength: f64,
    #[serde(rename = "CON")]
    pub constitution: f64,
    #[serde(rename = "DEX")]
    pub dexterity: f64,
    #[serde(rename = "INT")]
    pub intelligence: f64,
    #[serde(rename = "WIS")]
    pub wisdom: f64,
    #[serde(rename = "CHA")]
    pub charisma: f64,
    #[serde(rename = "HP Max")]
    pub hit_points_max: f64,
    #[serde(rename = "MP Max")]
    pub mana_points_max: f64,
    pub best: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub task: String,
    pub tasks: u64,
    #[serde(deserialize_with = "deserialize_elapsed")]
    pub elapsed: u64,
    pub kill: String,
    pub questmonster: String,
    pub questmonsterindex: u64,
}

fn deserialize_elapsed<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let elapsed = f64::deserialize(deserializer)?;
    if !elapsed.is_finite() || elapsed < 0.0 || elapsed > u64::MAX as f64 {
        return Err(de::Error::custom(
            "elapsed must be a non-negative finite integer",
        ));
    }
    Ok(elapsed.floor() as u64)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Equipment {
    #[serde(rename = "Weapon")]
    pub weapon: String,
    #[serde(rename = "Shield")]
    pub shield: String,
    #[serde(rename = "Helm")]
    pub helm: String,
    #[serde(rename = "Hauberk")]
    pub hauberk: String,
    #[serde(rename = "Brassairts")]
    pub brassairts: String,
    #[serde(rename = "Vambraces")]
    pub vambraces: String,
    #[serde(rename = "Gauntlets")]
    pub gauntlets: String,
    #[serde(rename = "Gambeson")]
    pub gambeson: String,
    #[serde(rename = "Cuisses")]
    pub cuisses: String,
    #[serde(rename = "Greaves")]
    pub greaves: String,
    #[serde(rename = "Sollerets")]
    pub sollerets: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryEntry {
    pub name: String,
    pub quantity: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spell {
    pub name: String,
    pub rank: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plot {
    pub act: u64,
    pub bestplot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    #[serde(rename = "ExpBar")]
    pub experience: ProgressBar,
    #[serde(rename = "EncumBar")]
    pub encumbrance: ProgressBar,
    #[serde(rename = "PlotBar")]
    pub plot: ProgressBar,
    #[serde(rename = "QuestBar")]
    pub quest: ProgressBar,
    #[serde(rename = "TaskBar")]
    pub task: ProgressBar,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressBar {
    pub position: f64,
    pub max: u64,
    pub percent: u64,
    pub remaining: u64,
    pub time: String,
    pub hint: String,
}

/// Identifies which of the five browser progress bars a `ProgressBar` value
/// represents, since each bar renders its `hint` from a different template
/// (`FormCreate` in the browser client wires up one `ProgressBar(id, tmpl)`
/// per bar).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressBarKind {
    Experience,
    Encumbrance,
    Plot,
    Quest,
    Task,
}

impl ProgressBar {
    /// Ports the browser `ProgressBar.reset`: assigns a new `max` and
    /// repositions (defaulting the position to `0`, matching `newposition ||
    /// 0` in the browser client).
    pub fn reset(&mut self, kind: ProgressBarKind, new_max: u64, new_position: f64) {
        self.max = new_max;
        self.reposition(kind, new_position);
    }

    /// Ports the browser `ProgressBar.reposition`: clamps to `[0, max]` and
    /// recomputes the derived percent/remaining/time/hint fields.
    pub fn reposition(&mut self, kind: ProgressBarKind, new_position: f64) {
        let max = self.max as f64;
        self.position = new_position.clamp(0.0, max);
        let remaining = (max - self.position).floor();
        self.percent = if max > 0.0 {
            ((100.0 * self.position) / max).floor() as u64
        } else {
            0
        };
        self.remaining = remaining as u64;
        self.time = rough_time(remaining);
        self.hint = kind.render_hint(self);
    }

    /// Ports the browser `ProgressBar.increment`.
    pub fn increment(&mut self, kind: ProgressBarKind, delta: f64) {
        self.reposition(kind, self.position + delta);
    }

    /// Ports the browser `ProgressBar.done`.
    pub fn done(&self) -> bool {
        self.position >= self.max as f64
    }
}

impl ProgressBarKind {
    fn render_hint(self, bar: &ProgressBar) -> String {
        match self {
            ProgressBarKind::Experience => format!("{} XP needed for next level", bar.remaining),
            ProgressBarKind::Encumbrance => format!("{}/{} cubits", bar.position, bar.max),
            ProgressBarKind::Plot => format!("{} remaining", bar.time),
            ProgressBarKind::Quest => format!("{}% complete", bar.percent),
            ProgressBarKind::Task => format!("{}%", bar.percent),
        }
    }
}

/// Ports the browser `RoughTime`, which buckets a duration (in seconds) into a
/// human-readable unit using floor division per bucket.
fn rough_time(seconds: f64) -> String {
    let seconds = seconds.max(0.0);
    if seconds < 120.0 {
        format!("{} seconds", seconds.floor())
    } else if seconds < 60.0 * 120.0 {
        format!("{} minutes", (seconds / 60.0).floor())
    } else if seconds < 60.0 * 60.0 * 48.0 {
        format!("{} hours", (seconds / 3600.0).floor())
    } else if seconds < 60.0 * 60.0 * 24.0 * 60.0 {
        format!("{} days", (seconds / (3600.0 * 24.0)).floor())
    } else if seconds < 60.0 * 60.0 * 24.0 * 30.0 * 24.0 {
        format!("{} months", (seconds / (3600.0 * 24.0 * 30.0)).floor())
    } else {
        format!(
            "{} years",
            (seconds / (3600.0 * 24.0 * 30.0 * 12.0)).floor()
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineMetadata {
    pub realm: String,
    pub host: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OnlineProfile {
    pub motto: String,
    pub guild: String,
}

impl Character {
    pub(crate) fn from_document(document: Value) -> Result<Self, SaveError> {
        let root = object(&document, "document")?;
        let traits = object_field(root, "Traits", "Traits")?;
        let stats = object_field(root, "Stats", "Stats")?;
        let equipment = object_field(root, "Equips", "Equips")?;

        Ok(Self {
            traits: Traits {
                name: string(traits, "Name", "Traits.Name")?,
                race: string(traits, "Race", "Traits.Race")?,
                class: string(traits, "Class", "Traits.Class")?,
                level: positive_integer(traits, "Level", "Traits.Level")?,
            },
            dna: alea(array_field(root, "dna", "dna")?, "dna")?,
            seed: alea(array_field(root, "seed", "seed")?, "seed")?,
            birthday: string(root, "birthday", "birthday")?,
            birthstamp: integer(root, "birthstamp", "birthstamp")?,
            stats: Attributes {
                seed: alea(array_field(stats, "seed", "Stats.seed")?, "Stats.seed")?,
                strength: number(stats, "STR", "Stats.STR")?,
                constitution: number(stats, "CON", "Stats.CON")?,
                dexterity: number(stats, "DEX", "Stats.DEX")?,
                intelligence: number(stats, "INT", "Stats.INT")?,
                wisdom: number(stats, "WIS", "Stats.WIS")?,
                charisma: number(stats, "CHA", "Stats.CHA")?,
                hit_points_max: number(stats, "HP Max", "Stats.HP Max")?,
                mana_points_max: number(stats, "MP Max", "Stats.MP Max")?,
                best: string(stats, "best", "Stats.best")?,
            },
            beststat: string(root, "beststat", "beststat")?,
            activity: Activity {
                task: string(root, "task", "task")?,
                tasks: integer(root, "tasks", "tasks")?,
                elapsed: integer(root, "elapsed", "elapsed")?,
                kill: string(root, "kill", "kill")?,
                questmonster: string(root, "questmonster", "questmonster")?,
                questmonsterindex: integer_or_default(
                    root,
                    "questmonsterindex",
                    "questmonsterindex",
                )?,
            },
            bestequip: string(root, "bestequip", "bestequip")?,
            equipment: Equipment {
                weapon: string(equipment, "Weapon", "Equips.Weapon")?,
                shield: string(equipment, "Shield", "Equips.Shield")?,
                helm: string(equipment, "Helm", "Equips.Helm")?,
                hauberk: string(equipment, "Hauberk", "Equips.Hauberk")?,
                brassairts: string(equipment, "Brassairts", "Equips.Brassairts")?,
                vambraces: string(equipment, "Vambraces", "Equips.Vambraces")?,
                gauntlets: string(equipment, "Gauntlets", "Equips.Gauntlets")?,
                gambeson: string(equipment, "Gambeson", "Equips.Gambeson")?,
                cuisses: string(equipment, "Cuisses", "Equips.Cuisses")?,
                greaves: string(equipment, "Greaves", "Equips.Greaves")?,
                sollerets: string(equipment, "Sollerets", "Equips.Sollerets")?,
            },
            inventory: inventory(array_field(root, "Inventory", "Inventory")?)?,
            spells: spells(array_field(root, "Spells", "Spells")?)?,
            plot: Plot {
                act: integer(root, "act", "act")?,
                bestplot: string(root, "bestplot", "bestplot")?,
            },
            quests: strings(array_field(root, "Quests", "Quests")?, "Quests")?,
            progress: Progress {
                experience: bar(object_field(root, "ExpBar", "ExpBar")?, "ExpBar")?,
                encumbrance: bar(object_field(root, "EncumBar", "EncumBar")?, "EncumBar")?,
                plot: bar(object_field(root, "PlotBar", "PlotBar")?, "PlotBar")?,
                quest: bar(object_field(root, "QuestBar", "QuestBar")?, "QuestBar")?,
                task: bar(object_field(root, "TaskBar", "TaskBar")?, "TaskBar")?,
            },
            queue: strings(array_field(root, "queue", "queue")?, "queue")?,
            date: string(root, "date", "date")?,
            stamp: integer(root, "stamp", "stamp")?,
            online: root
                .get("online")
                .map(|value| {
                    let online = value
                        .as_object()
                        .ok_or_else(|| SaveError::invalid("online"))?;
                    Ok::<OnlineMetadata, SaveError>(OnlineMetadata {
                        realm: string(online, "realm", "online.realm")?,
                        host: string(online, "host", "online.host")?,
                    })
                })
                .transpose()?,
            profile: OnlineProfile {
                motto: string_or_default(root, "motto", "motto")?,
                guild: string_or_default(root, "guild", "guild")?,
            },
            save_name: string(root, "saveName", "saveName")?,
            bestspell: string(root, "bestspell", "bestspell")?,
            bestquest: string_or_default(root, "bestquest", "bestquest")?,
            document,
        })
    }

    /// Builds a browser save document from current state, keeping unrelated
    /// fields (including the credential-bearing `online.passkey`) from the
    /// original document.
    pub fn to_document(&self) -> Value {
        let mut root = match &self.document {
            Value::Object(map) => map.clone(),
            _ => Map::new(),
        };
        let number = |value: f64| {
            if value.is_finite() && value.fract() == 0.0 && value.abs() < 9_007_199_254_740_992.0 {
                Value::from(value as i64)
            } else {
                Value::from(value)
            }
        };
        let alea = |state: &AleaState| Value::Array(state.0.iter().map(|v| number(*v)).collect());
        let pairs = |rows: Vec<(&str, Value)>| {
            Value::Array(
                rows.into_iter()
                    .map(|(name, value)| Value::Array(vec![Value::from(name), value]))
                    .collect(),
            )
        };
        let merge = |root: &mut Map<String, Value>, key: &str, values: Vec<(&str, Value)>| {
            let mut object = match root.get(key) {
                Some(Value::Object(map)) => map.clone(),
                _ => Map::new(),
            };
            for (name, value) in values {
                object.insert(name.to_owned(), value);
            }
            root.insert(key.to_owned(), Value::Object(object));
        };
        let bar = |bar: &ProgressBar| {
            vec![
                ("position", number(bar.position)),
                ("max", Value::from(bar.max)),
                ("percent", Value::from(bar.percent)),
                ("remaining", Value::from(bar.remaining)),
                ("time", Value::from(bar.time.as_str())),
                ("hint", Value::from(bar.hint.as_str())),
            ]
        };

        merge(
            &mut root,
            "Traits",
            vec![
                ("Name", Value::from(self.traits.name.as_str())),
                ("Race", Value::from(self.traits.race.as_str())),
                ("Class", Value::from(self.traits.class.as_str())),
                ("Level", Value::from(self.traits.level)),
            ],
        );
        root.insert("dna".to_owned(), alea(&self.dna));
        root.insert("seed".to_owned(), alea(&self.seed));
        root.insert("birthday".to_owned(), Value::from(self.birthday.as_str()));
        root.insert("birthstamp".to_owned(), Value::from(self.birthstamp));
        merge(
            &mut root,
            "Stats",
            vec![
                ("seed", alea(&self.stats.seed)),
                ("STR", number(self.stats.strength)),
                ("CON", number(self.stats.constitution)),
                ("DEX", number(self.stats.dexterity)),
                ("INT", number(self.stats.intelligence)),
                ("WIS", number(self.stats.wisdom)),
                ("CHA", number(self.stats.charisma)),
                ("HP Max", number(self.stats.hit_points_max)),
                ("MP Max", number(self.stats.mana_points_max)),
                ("best", Value::from(self.stats.best.as_str())),
            ],
        );
        root.insert("beststat".to_owned(), Value::from(self.beststat.as_str()));
        root.insert("task".to_owned(), Value::from(self.activity.task.as_str()));
        root.insert("tasks".to_owned(), Value::from(self.activity.tasks));
        root.insert("elapsed".to_owned(), Value::from(self.activity.elapsed));
        root.insert("kill".to_owned(), Value::from(self.activity.kill.as_str()));
        root.insert(
            "questmonster".to_owned(),
            Value::from(self.activity.questmonster.as_str()),
        );
        root.insert(
            "questmonsterindex".to_owned(),
            Value::from(self.activity.questmonsterindex),
        );
        root.insert("bestequip".to_owned(), Value::from(self.bestequip.as_str()));
        let equipment = &self.equipment;
        merge(
            &mut root,
            "Equips",
            [
                ("Weapon", &equipment.weapon),
                ("Shield", &equipment.shield),
                ("Helm", &equipment.helm),
                ("Hauberk", &equipment.hauberk),
                ("Brassairts", &equipment.brassairts),
                ("Vambraces", &equipment.vambraces),
                ("Gauntlets", &equipment.gauntlets),
                ("Gambeson", &equipment.gambeson),
                ("Cuisses", &equipment.cuisses),
                ("Greaves", &equipment.greaves),
                ("Sollerets", &equipment.sollerets),
            ]
            .into_iter()
            .map(|(name, value)| (name, Value::from(value.as_str())))
            .collect(),
        );
        root.insert(
            "Inventory".to_owned(),
            pairs(
                self.inventory
                    .iter()
                    .map(|entry| (entry.name.as_str(), Value::from(entry.quantity)))
                    .collect(),
            ),
        );
        root.insert(
            "Spells".to_owned(),
            pairs(
                self.spells
                    .iter()
                    .map(|spell| (spell.name.as_str(), Value::from(spell.rank.as_str())))
                    .collect(),
            ),
        );
        root.insert("act".to_owned(), Value::from(self.plot.act));
        root.insert(
            "bestplot".to_owned(),
            Value::from(self.plot.bestplot.as_str()),
        );
        root.insert(
            "Quests".to_owned(),
            Value::Array(
                self.quests
                    .iter()
                    .map(|q| Value::from(q.as_str()))
                    .collect(),
            ),
        );
        merge(&mut root, "ExpBar", bar(&self.progress.experience));
        merge(&mut root, "EncumBar", bar(&self.progress.encumbrance));
        merge(&mut root, "PlotBar", bar(&self.progress.plot));
        merge(&mut root, "QuestBar", bar(&self.progress.quest));
        merge(&mut root, "TaskBar", bar(&self.progress.task));
        root.insert(
            "queue".to_owned(),
            Value::Array(self.queue.iter().map(|q| Value::from(q.as_str())).collect()),
        );
        root.insert("date".to_owned(), Value::from(self.date.as_str()));
        root.insert("stamp".to_owned(), Value::from(self.stamp));
        if let Some(online) = &self.online {
            merge(
                &mut root,
                "online",
                vec![
                    ("realm", Value::from(online.realm.as_str())),
                    ("host", Value::from(online.host.as_str())),
                ],
            );
        }
        for (key, value) in [
            ("motto", &self.profile.motto),
            ("guild", &self.profile.guild),
        ] {
            if value.is_empty() && !root.contains_key(key) {
                continue;
            }
            root.insert(key.to_owned(), Value::from(value.as_str()));
        }
        root.insert("saveName".to_owned(), Value::from(self.save_name.as_str()));
        root.insert("bestspell".to_owned(), Value::from(self.bestspell.as_str()));
        if !self.bestquest.is_empty() || root.contains_key("bestquest") {
            root.insert("bestquest".to_owned(), Value::from(self.bestquest.as_str()));
        }
        Value::Object(root)
    }

    pub fn summary(&self) -> String {
        let online = self
            .online
            .as_ref()
            .map(|online| format!("{} (passkey: [redacted])", online.realm))
            .unwrap_or_else(|| "offline".to_owned());
        format!(
            "Identity\n  Name: {}\n  Race: {}\n  Class: {}\n  Level: {}\n  Online: {online}\n\
             \nOnline Profile\n  Motto: {}\n  Guild: {}\n\
             \nAttributes\n  STR: {}\n  CON: {}\n  DEX: {}\n  INT: {}\n  WIS: {}\n  CHA: {}\n  HP Max: {}\n  MP Max: {}\n\
             \nActivity\n  Task: {}\n  Tasks: {}\n  Elapsed: {}\n  Status: {}\n  Quest monster: {}\n\
             \nProgress\n  Experience: {}/{}\n  Encumbrance: {}/{}\n  Plot: {}/{}\n  Quest: {}/{}\n  Task: {}/{}\n\
             \nEquipment\n  Weapon: {}\n  Shield: {}\n  Helm: {}\n  Hauberk: {}\n  Brassairts: {}\n  Vambraces: {}\n  Gauntlets: {}\n  Gambeson: {}\n  Cuisses: {}\n  Greaves: {}\n  Sollerets: {}\n\
             \nInventory\n{}\n\
             \nSpells\n{}\n\
             \nPlot and Quests\n  Act: {}\n  Best plot: {}\n  Quests:\n{}",
            self.traits.name,
            self.traits.race,
            self.traits.class,
            self.traits.level,
            self.profile.motto,
            self.profile.guild,
            self.stats.strength,
            self.stats.constitution,
            self.stats.dexterity,
            self.stats.intelligence,
            self.stats.wisdom,
            self.stats.charisma,
            self.stats.hit_points_max,
            self.stats.mana_points_max,
            self.activity.task,
            self.activity.tasks,
            self.activity.elapsed,
            self.activity.kill,
            self.activity.questmonster,
            self.progress.experience.position,
            self.progress.experience.max,
            self.progress.encumbrance.position,
            self.progress.encumbrance.max,
            self.progress.plot.position,
            self.progress.plot.max,
            self.progress.quest.position,
            self.progress.quest.max,
            self.progress.task.position,
            self.progress.task.max,
            self.equipment.weapon,
            self.equipment.shield,
            self.equipment.helm,
            self.equipment.hauberk,
            self.equipment.brassairts,
            self.equipment.vambraces,
            self.equipment.gauntlets,
            self.equipment.gambeson,
            self.equipment.cuisses,
            self.equipment.greaves,
            self.equipment.sollerets,
            listed(
                self.inventory
                    .iter()
                    .map(|entry| format!("  {} x{}", entry.name, entry.quantity))
            ),
            listed(
                self.spells
                    .iter()
                    .map(|spell| format!("  {} {}", spell.name, spell.rank))
            ),
            self.plot.act,
            self.plot.bestplot,
            listed(self.quests.iter().map(|quest| format!("  {quest}"))),
        )
    }
}

fn listed(items: impl Iterator<Item = String>) -> String {
    let items = items.collect::<Vec<_>>();
    if items.is_empty() {
        "  (none)".to_owned()
    } else {
        items.join("\n")
    }
}

fn object<'a>(value: &'a Value, field: &str) -> Result<&'a Map<String, Value>, SaveError> {
    value.as_object().ok_or_else(|| SaveError::invalid(field))
}
fn object_field<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    field: &str,
) -> Result<&'a Map<String, Value>, SaveError> {
    object
        .get(key)
        .and_then(Value::as_object)
        .ok_or_else(|| SaveError::invalid(field))
}
fn array_field<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    field: &str,
) -> Result<&'a [Value], SaveError> {
    object
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| SaveError::invalid(field))
}
fn string(object: &Map<String, Value>, key: &str, field: &str) -> Result<String, SaveError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| SaveError::invalid(field))
}
/// Like [`string`], but defaults to `""` when the key is absent, matching a
/// browser field the client never assigned for a fresh character (e.g.
/// `game.bestquest` is unset until the first quest completes, and `undefined`
/// is omitted by `JSON.stringify`). An explicitly present but invalid value
/// is still rejected.
fn string_or_default(
    object: &Map<String, Value>,
    key: &str,
    field: &str,
) -> Result<String, SaveError> {
    match object.get(key) {
        None => Ok(String::new()),
        Some(value) => value
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| SaveError::invalid(field)),
    }
}
fn integer(object: &Map<String, Value>, key: &str, field: &str) -> Result<u64, SaveError> {
    object
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| SaveError::invalid(field))
}
/// Like [`integer`], but defaults to `0` when the key is absent, matching the
/// browser client dropping a field it never assigned (e.g. `undefined` is
/// omitted by `JSON.stringify`, and `StrToIntDef`/`GetI` default to `0` when
/// reading it back). An explicitly present but invalid value is still
/// rejected.
fn integer_or_default(
    object: &Map<String, Value>,
    key: &str,
    field: &str,
) -> Result<u64, SaveError> {
    match object.get(key) {
        None => Ok(0),
        Some(value) => value.as_u64().ok_or_else(|| SaveError::invalid(field)),
    }
}
fn positive_integer(object: &Map<String, Value>, key: &str, field: &str) -> Result<u64, SaveError> {
    integer(object, key, field).and_then(|value| {
        (value > 0)
            .then_some(value)
            .ok_or_else(|| SaveError::invalid(field))
    })
}
fn alea(values: &[Value], field: &str) -> Result<AleaState, SaveError> {
    let values = values
        .iter()
        .map(Value::as_f64)
        .collect::<Option<Vec<_>>>()
        .filter(|values| values.len() == 4)
        .ok_or_else(|| SaveError::invalid(field))?;
    Ok(AleaState([values[0], values[1], values[2], values[3]]))
}
fn strings(values: &[Value], field: &str) -> Result<Vec<String>, SaveError> {
    values
        .iter()
        .map(Value::as_str)
        .collect::<Option<Vec<_>>>()
        .map(|values| values.into_iter().map(str::to_owned).collect())
        .ok_or_else(|| SaveError::invalid(field))
}
fn inventory(values: &[Value]) -> Result<Vec<InventoryEntry>, SaveError> {
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let field = format!("Inventory[{index}]");
            let pair = value
                .as_array()
                .filter(|pair| pair.len() == 2)
                .ok_or_else(|| SaveError::invalid(&field))?;
            Ok(InventoryEntry {
                name: pair[0]
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| SaveError::invalid(format!("{field}[0]")))?,
                quantity: pair[1]
                    .as_u64()
                    .ok_or_else(|| SaveError::invalid(format!("{field}[1]")))?,
            })
        })
        .collect()
}
fn spells(values: &[Value]) -> Result<Vec<Spell>, SaveError> {
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let field = format!("Spells[{index}]");
            let pair = value
                .as_array()
                .filter(|pair| pair.len() == 2)
                .ok_or_else(|| SaveError::invalid(&field))?;
            Ok(Spell {
                name: pair[0]
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| SaveError::invalid(format!("{field}[0]")))?,
                rank: pair[1]
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| SaveError::invalid(format!("{field}[1]")))?,
            })
        })
        .collect()
}
fn bar(object: &Map<String, Value>, prefix: &str) -> Result<ProgressBar, SaveError> {
    Ok(ProgressBar {
        position: number(object, "position", &format!("{prefix}.position"))?,
        max: integer(object, "max", &format!("{prefix}.max"))?,
        percent: integer(object, "percent", &format!("{prefix}.percent"))?,
        remaining: integer(object, "remaining", &format!("{prefix}.remaining"))?,
        time: string(object, "time", &format!("{prefix}.time"))?,
        hint: string(object, "hint", &format!("{prefix}.hint"))?,
    })
}

#[cfg(test)]
mod tests {
    use super::{Activity, Character};

    #[test]
    fn document_round_trips_current_state_and_preserves_unknown_fields() {
        let document: serde_json::Value =
            serde_json::from_str(include_str!("../tests/fixtures/reference-save.json")).unwrap();
        let mut character = Character::from_document(document.clone()).unwrap();
        character.traits.level = 9;
        character.inventory.push(super::InventoryEntry {
            name: "Rat tail".to_owned(),
            quantity: 3,
        });
        character.profile.motto = "Onward".to_owned();
        character
            .progress
            .task
            .reposition(super::ProgressBarKind::Task, 12.5);
        let exported = character.to_document();
        assert_eq!(
            exported["unrecognized-future-field"],
            document["unrecognized-future-field"]
        );
        assert_eq!(exported["online"]["passkey"], document["online"]["passkey"]);
        let reimported = Character::from_document(exported).unwrap();
        assert_eq!(
            serde_json::to_value(&reimported).unwrap(),
            serde_json::to_value(&character).unwrap()
        );
    }

    #[test]
    fn floors_legacy_fractional_elapsed_values_from_local_state() {
        let activity: Activity = serde_json::from_str(
            r#"{
                "task": "kill|Goblin|1|ear",
                "tasks": 7,
                "elapsed": 7107.699999999997,
                "kill": "Executing a Goblin...",
                "questmonster": "",
                "questmonsterindex": 0
            }"#,
        )
        .unwrap();

        assert_eq!(activity.elapsed, 7_107);
    }
}
