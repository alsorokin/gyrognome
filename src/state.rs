use serde::Serialize;
use serde_json::{Map, Value};

use crate::save::SaveError;

#[derive(Debug, Clone, Serialize)]
pub struct Character {
    #[serde(skip_serializing)]
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

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct AleaState(pub [f64; 4]);

#[derive(Debug, Clone, Serialize)]
pub struct Attributes {
    pub seed: AleaState,
    #[serde(rename = "STR")]
    pub strength: u64,
    #[serde(rename = "CON")]
    pub constitution: u64,
    #[serde(rename = "DEX")]
    pub dexterity: u64,
    #[serde(rename = "INT")]
    pub intelligence: u64,
    #[serde(rename = "WIS")]
    pub wisdom: u64,
    #[serde(rename = "CHA")]
    pub charisma: u64,
    #[serde(rename = "HP Max")]
    pub hit_points_max: u64,
    #[serde(rename = "MP Max")]
    pub mana_points_max: u64,
    pub best: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Activity {
    pub task: String,
    pub tasks: u64,
    pub elapsed: u64,
    pub kill: String,
    pub questmonster: String,
    pub questmonsterindex: u64,
}

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
pub struct InventoryEntry {
    pub name: String,
    pub quantity: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Spell {
    pub name: String,
    pub rank: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Plot {
    pub act: u64,
    pub bestplot: String,
}

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
pub struct ProgressBar {
    pub position: f64,
    pub max: u64,
    pub percent: u64,
    pub remaining: u64,
    pub time: String,
    pub hint: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OnlineMetadata {
    pub realm: String,
    pub host: String,
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
                strength: integer(stats, "STR", "Stats.STR")?,
                constitution: integer(stats, "CON", "Stats.CON")?,
                dexterity: integer(stats, "DEX", "Stats.DEX")?,
                intelligence: integer(stats, "INT", "Stats.INT")?,
                wisdom: integer(stats, "WIS", "Stats.WIS")?,
                charisma: integer(stats, "CHA", "Stats.CHA")?,
                hit_points_max: integer(stats, "HP Max", "Stats.HP Max")?,
                mana_points_max: integer(stats, "MP Max", "Stats.MP Max")?,
                best: string(stats, "best", "Stats.best")?,
            },
            beststat: string(root, "beststat", "beststat")?,
            activity: Activity {
                task: string(root, "task", "task")?,
                tasks: integer(root, "tasks", "tasks")?,
                elapsed: integer(root, "elapsed", "elapsed")?,
                kill: string(root, "kill", "kill")?,
                questmonster: string(root, "questmonster", "questmonster")?,
                questmonsterindex: integer(root, "questmonsterindex", "questmonsterindex")?,
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
            save_name: string(root, "saveName", "saveName")?,
            bestspell: string(root, "bestspell", "bestspell")?,
            bestquest: string(root, "bestquest", "bestquest")?,
            document,
        })
    }

    pub fn summary(&self) -> String {
        let online = self
            .online
            .as_ref()
            .map(|online| format!("{} (passkey: [redacted])", online.realm))
            .unwrap_or_else(|| "offline".to_owned());
        format!(
            "Identity\n  Name: {}\n  Race: {}\n  Class: {}\n  Level: {}\n  Online: {online}\n\
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
fn integer(object: &Map<String, Value>, key: &str, field: &str) -> Result<u64, SaveError> {
    object
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| SaveError::invalid(field))
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
