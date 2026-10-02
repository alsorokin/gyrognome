use std::{collections::HashSet, fmt};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const MAX_COMPONENT_DEPTH: usize = 32;
pub const MAX_COMPONENTS: usize = 256;
pub const MAX_VALUE_BYTES: usize = 1024 * 1024;
pub const MAX_GAME_LIST_ROWS: usize = 100_000;

#[derive(Debug, Clone, PartialEq)]
pub struct DesktopDocument {
    pub components: Vec<DesktopComponent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DesktopComponent {
    pub class_name: String,
    pub name: String,
    pub properties: Vec<DesktopProperty>,
    pub children: Vec<DesktopComponent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DesktopProperty {
    pub name: String,
    pub value: DesktopValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DesktopValue {
    Null,
    List(Vec<DesktopValue>),
    Integer(i64),
    Extended([u8; 10]),
    String(Vec<u8>),
    Ident(Vec<u8>),
    Boolean(bool),
    Binary(Vec<u8>),
    Set(Vec<Vec<u8>>),
    Nil,
    Collection(Vec<DesktopCollectionItem>),
    Single([u8; 4]),
    Currency([u8; 8]),
    Date([u8; 8]),
    WideString(Vec<u8>),
    Utf8String(Vec<u8>),
    Double([u8; 8]),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DesktopCollectionItem {
    pub order: Option<i64>,
    pub properties: Vec<DesktopProperty>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameListRow {
    pub header: [i32; 5],
    pub caption: Vec<u8>,
    pub subitems: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopMappedSave {
    pub traits: Vec<GameListRow>,
    pub stats: Vec<GameListRow>,
    pub equipment: Vec<GameListRow>,
    pub inventory: Vec<GameListRow>,
    pub spells: Vec<GameListRow>,
    pub plots: Vec<GameListRow>,
    pub quests: Vec<GameListRow>,
    pub current_task: Option<Vec<u8>>,
    pub quest_marker: Option<Vec<u8>>,
    pub quest_index: Option<i64>,
    pub queue: Vec<Vec<u8>>,
    pub activity: Option<Vec<u8>>,
    pub bars: DesktopMappedBars,
    pub prized_equipment: Option<i64>,
    pub game_style: Option<i64>,
    pub profile: DesktopMappedProfile,
    pub private: DesktopPrivateMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopMappedBars {
    pub experience: DesktopMappedBar,
    pub encumbrance: DesktopMappedBar,
    pub plot: DesktopMappedBar,
    pub quest: DesktopMappedBar,
    pub task: DesktopMappedBar,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopMappedBar {
    pub position: Option<i64>,
    pub maximum: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopMappedProfile {
    pub motto: Option<Vec<u8>>,
    pub guild: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopPrivateMetadata {
    pub passkey: Option<i64>,
    pub passkey_text: Option<Vec<u8>>,
    pub realm: Option<Vec<u8>>,
    pub endpoint: Option<Vec<u8>>,
    pub account: Option<Vec<u8>>,
    pub password: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopValidatedSave {
    pub traits: Vec<DesktopValidatedRow>,
    pub stats: Vec<DesktopValidatedRow>,
    pub equipment: Vec<DesktopValidatedRow>,
    pub inventory: Vec<DesktopValidatedRow>,
    pub spells: Vec<DesktopValidatedRow>,
    pub plots: Vec<DesktopValidatedRow>,
    pub quests: Vec<DesktopValidatedRow>,
    pub current_task: String,
    pub quest: DesktopQuestMarker,
    pub queue: Vec<DesktopQueueCommand>,
    pub activity: String,
    pub bars: DesktopValidatedBars,
    pub prized_equipment: usize,
    pub game_style: u32,
    pub profile: DesktopValidatedProfile,
    pub private: DesktopValidatedPrivateMetadata,
    pub adaptations: DesktopAdaptations,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopValidatedRow {
    pub header: [i32; 5],
    pub caption: String,
    pub subitems: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DesktopQuestMarker {
    None,
    LegacyPlaceholder { index: usize },
    Value { marker: String, index: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopQueueCommand {
    pub kind: DesktopQueueKind,
    pub duration_seconds: u32,
    pub caption: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DesktopQueueKind {
    Task,
    Plot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopValidatedBars {
    pub experience: DesktopValidatedBar,
    pub encumbrance: DesktopValidatedBar,
    pub plot: DesktopValidatedBar,
    pub quest: DesktopValidatedBar,
    pub task: DesktopValidatedBar,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopValidatedBar {
    pub position: u64,
    pub maximum: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopValidatedProfile {
    pub motto: String,
    pub guild: String,
}

#[derive(Clone, PartialEq, Eq)]
pub struct DesktopValidatedPrivateMetadata {
    pub passkey: i32,
    pub realm: String,
    pub endpoint: String,
    pub account: String,
    pub password: String,
}

impl fmt::Debug for DesktopValidatedPrivateMetadata {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DesktopValidatedPrivateMetadata")
            .field("passkey", &"[redacted]")
            .field("realm", &"[redacted]")
            .field("endpoint", &"[redacted]")
            .field("account", &"[redacted]")
            .field("password", &"[redacted]")
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopAdaptations {
    pub legacy_prologue_62: bool,
    pub legacy_quest_placeholder: bool,
    pub spelling_patch_applied: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DesktopParseError {
    #[error("desktop component stream is truncated at byte {offset}")]
    Truncated { offset: usize },
    #[error("desktop component signature is invalid at byte {offset}")]
    InvalidSignature { offset: usize },
    #[error("desktop component prefix is invalid at byte {offset}")]
    InvalidPrefix { offset: usize },
    #[error("desktop component nesting exceeds {MAX_COMPONENT_DEPTH}")]
    DepthLimit,
    #[error("desktop component count exceeds {MAX_COMPONENTS}")]
    ComponentLimit,
    #[error("desktop length-delimited value exceeds {MAX_VALUE_BYTES} bytes at byte {offset}")]
    ValueLimit { offset: usize },
    #[error("desktop value tag {tag} is unsupported at byte {offset}")]
    UnsupportedValue { offset: usize, tag: u8 },
    #[error("desktop value framing is invalid at byte {offset}")]
    InvalidValue { offset: usize },
    #[error("desktop component name is duplicated")]
    DuplicateComponent,
    #[error("desktop component property is duplicated")]
    DuplicateProperty,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GameListError {
    #[error("desktop game-list data is truncated at byte {offset}")]
    Truncated { offset: usize },
    #[error("desktop game-list total length is inconsistent")]
    TotalLength,
    #[error("desktop game-list row count is invalid")]
    RowCount,
    #[error("desktop game-list aggregate rows exceed {MAX_GAME_LIST_ROWS}")]
    RowLimit,
    #[error("desktop game-list subitem count is invalid")]
    SubitemCount,
    #[error("desktop game-list has unexplained trailing bytes")]
    TrailingBytes,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DesktopMappingError {
    #[error("desktop save is missing required component {0}")]
    MissingComponent(&'static str),
    #[error("desktop component {component} has an unsupported class; expected {expected}")]
    UnsupportedClass {
        component: &'static str,
        expected: &'static str,
    },
    #[error("desktop component {component} property {property} has an unsupported value type")]
    PropertyType {
        component: &'static str,
        property: &'static str,
    },
    #[error(transparent)]
    GameList(#[from] GameListError),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DesktopValidationError {
    #[error("desktop field {0} uses unsupported non-ASCII text")]
    UnsupportedEncoding(&'static str),
    #[error("desktop field {0} is missing")]
    Missing(&'static str),
    #[error("desktop field {0} has an invalid numeric value")]
    InvalidNumber(&'static str),
    #[error("desktop field {0} has an invalid ordered layout")]
    InvalidLayout(&'static str),
    #[error("desktop spell rank is invalid")]
    InvalidRank,
    #[error("desktop field {0} has an out-of-range index")]
    InvalidIndex(&'static str),
    #[error("desktop queue command is unsupported")]
    InvalidCommand,
    #[error("desktop current task form is unsupported")]
    InvalidTask,
    #[error("desktop passkey representations disagree")]
    PasskeyMismatch,
}

pub fn parse_components(bytes: &[u8]) -> Result<DesktopDocument, DesktopParseError> {
    let mut parser = Parser {
        bytes,
        position: 0,
        component_count: 0,
        component_names: HashSet::new(),
    };
    let mut components = Vec::new();
    while parser.position < bytes.len() {
        components.push(parser.component(1, true)?);
    }
    Ok(DesktopDocument { components })
}

pub fn map_desktop_document(
    document: &DesktopDocument,
) -> Result<DesktopMappedSave, DesktopMappingError> {
    let traits = component(document, "Traits", "TListView")?;
    let stats = component(document, "Stats", "TListView")?;
    let equipment = component(document, "Equips", "TListView")?;
    let inventory = component(document, "Inventory", "TListView")?;
    let spells = component(document, "Spells", "TListView")?;
    let plots = component(document, "Plots", "TListView")?;
    let quests = component(document, "Quests", "TListView")?;
    let current_task = component(document, "fTask", "TLabel")?;
    let quest_marker = component(document, "fQuest", "TLabel")?;
    let queue = component(document, "fQueue", "TListBox")?;
    let activity = component(document, "Kill", "TStatusBar")?;
    let experience = component(document, "ExpBar", "TProgressBar")?;
    let encumbrance = component(document, "EncumBar", "TProgressBar")?;
    let plot = component(document, "PlotBar", "TProgressBar")?;
    let quest = component(document, "QuestBar", "TProgressBar")?;
    let task = component(document, "TaskBar", "TProgressBar")?;
    let game_style = component(document, "InventoryLabelAlsoGameStyle", "TLabel")?;
    let guild = component(document, "Label1", "TLabel")?;

    Ok(DesktopMappedSave {
        traits: list_rows(traits, "Traits")?,
        stats: list_rows(stats, "Stats")?,
        equipment: list_rows(equipment, "Equips")?,
        inventory: list_rows(inventory, "Inventory")?,
        spells: list_rows(spells, "Spells")?,
        plots: list_rows(plots, "Plots")?,
        quests: list_rows(quests, "Quests")?,
        current_task: optional_text(current_task, "fTask", "Caption")?,
        quest_marker: optional_text(quest_marker, "fQuest", "Caption")?,
        quest_index: optional_integer(quest_marker, "fQuest", "Tag")?,
        queue: string_list(queue, "fQueue", "Items.Strings")?,
        activity: optional_text(activity, "Kill", "SimpleText")?,
        bars: DesktopMappedBars {
            experience: bar(experience, "ExpBar")?,
            encumbrance: bar(encumbrance, "EncumBar")?,
            plot: bar(plot, "PlotBar")?,
            quest: bar(quest, "QuestBar")?,
            task: bar(task, "TaskBar")?,
        },
        prized_equipment: optional_integer(equipment, "Equips", "Tag")?,
        game_style: optional_integer(game_style, "InventoryLabelAlsoGameStyle", "Tag")?,
        profile: DesktopMappedProfile {
            motto: optional_text(stats, "Stats", "Hint")?,
            guild: optional_text(guild, "Label1", "Hint")?,
        },
        private: DesktopPrivateMetadata {
            passkey: optional_integer(traits, "Traits", "Tag")?,
            passkey_text: optional_text(traits, "Traits", "Hint")?,
            realm: optional_text(spells, "Spells", "Hint")?,
            endpoint: optional_text(equipment, "Equips", "Hint")?,
            account: optional_text(inventory, "Inventory", "Hint")?,
            password: optional_text(plots, "Plots", "Hint")?,
        },
    })
}

fn component<'a>(
    document: &'a DesktopDocument,
    name: &'static str,
    expected_class: &'static str,
) -> Result<&'a DesktopComponent, DesktopMappingError> {
    let component = document
        .components
        .iter()
        .find(|component| component.name.eq_ignore_ascii_case(name))
        .ok_or(DesktopMappingError::MissingComponent(name))?;
    if !component.class_name.eq_ignore_ascii_case(expected_class) {
        return Err(DesktopMappingError::UnsupportedClass {
            component: name,
            expected: expected_class,
        });
    }
    Ok(component)
}

fn property<'a>(component: &'a DesktopComponent, name: &str) -> Option<&'a DesktopValue> {
    component
        .properties
        .iter()
        .find(|property| property.name.eq_ignore_ascii_case(name))
        .map(|property| &property.value)
}

fn list_rows(
    component: &DesktopComponent,
    component_name: &'static str,
) -> Result<Vec<GameListRow>, DesktopMappingError> {
    match property(component, "Items.Data") {
        None => Ok(Vec::new()),
        Some(DesktopValue::Binary(bytes)) => Ok(decode_game_list(bytes)?),
        Some(_) => Err(DesktopMappingError::PropertyType {
            component: component_name,
            property: "Items.Data",
        }),
    }
}

fn optional_text(
    component: &DesktopComponent,
    component_name: &'static str,
    property_name: &'static str,
) -> Result<Option<Vec<u8>>, DesktopMappingError> {
    match property(component, property_name) {
        None => Ok(None),
        Some(DesktopValue::String(value) | DesktopValue::Utf8String(value)) => {
            Ok(Some(value.clone()))
        }
        Some(_) => Err(DesktopMappingError::PropertyType {
            component: component_name,
            property: property_name,
        }),
    }
}

fn optional_integer(
    component: &DesktopComponent,
    component_name: &'static str,
    property_name: &'static str,
) -> Result<Option<i64>, DesktopMappingError> {
    match property(component, property_name) {
        None => Ok(None),
        Some(DesktopValue::Integer(value)) => Ok(Some(*value)),
        Some(_) => Err(DesktopMappingError::PropertyType {
            component: component_name,
            property: property_name,
        }),
    }
}

fn string_list(
    component: &DesktopComponent,
    component_name: &'static str,
    property_name: &'static str,
) -> Result<Vec<Vec<u8>>, DesktopMappingError> {
    let value = property(component, property_name).or_else(|| property(component, "Items"));
    match value {
        None => Ok(Vec::new()),
        Some(DesktopValue::List(values)) => values
            .iter()
            .map(|value| match value {
                DesktopValue::String(value) | DesktopValue::Utf8String(value) => Ok(value.clone()),
                _ => Err(DesktopMappingError::PropertyType {
                    component: component_name,
                    property: property_name,
                }),
            })
            .collect(),
        Some(_) => Err(DesktopMappingError::PropertyType {
            component: component_name,
            property: property_name,
        }),
    }
}

fn bar(
    component: &DesktopComponent,
    component_name: &'static str,
) -> Result<DesktopMappedBar, DesktopMappingError> {
    Ok(DesktopMappedBar {
        position: optional_integer(component, component_name, "Position")?,
        maximum: optional_integer(component, component_name, "Max")?,
    })
}

pub fn validate_desktop_save(
    mapped: &DesktopMappedSave,
) -> Result<DesktopValidatedSave, DesktopValidationError> {
    let traits = validated_rows(&mapped.traits, "Traits")?;
    let stats = validated_rows(&mapped.stats, "Stats")?;
    let mut equipment = validated_rows(&mapped.equipment, "Equips")?;
    let inventory = validated_rows(&mapped.inventory, "Inventory")?;
    let mut spells = validated_rows(&mapped.spells, "Spells")?;
    let plots = validated_rows(&mapped.plots, "Plots")?;
    let quests = validated_rows(&mapped.quests, "Quests")?;

    require_layout(&traits, &["Name", "Race", "Class", "Level"], "Traits")?;
    require_layout(
        &stats,
        &["STR", "CON", "DEX", "INT", "WIS", "CHA", "HP Max", "MP Max"],
        "Stats",
    )?;
    require_layout(
        &equipment,
        &[
            "Weapon",
            "Shield",
            "Helm",
            "Hauberk",
            "Brassairts",
            "Vambraces",
            "Gauntlets",
            "Gambeson",
            "Cuisses",
            "Greaves",
            "Sollerets",
        ],
        "Equips",
    )?;
    require_one_subitem(&traits, "Traits")?;
    require_one_subitem(&stats, "Stats")?;
    normalize_optional_single_subitem(&mut equipment, "Equips")?;
    require_one_subitem(&inventory, "Inventory")?;
    require_one_subitem(&spells, "Spells")?;

    let level = parse_positive(&traits[3].subitems[0], "Traits.Level")?;
    if level > 1_000 {
        return Err(DesktopValidationError::InvalidNumber("Traits.Level"));
    }
    for (index, row) in stats.iter().enumerate() {
        parse_nonnegative(
            &row.subitems[0],
            if index < 6 { "Stats" } else { "Stats Max" },
        )?;
    }
    for row in &inventory {
        parse_nonnegative(&row.subitems[0], "Inventory")?;
    }

    let mut spelling_patch_applied = false;
    for (index, spell) in spells.iter_mut().enumerate() {
        if !is_canonical_roman(&spell.subitems[0]) {
            return Err(DesktopValidationError::InvalidRank);
        }
        if index > 0 && spell.caption == "Innoculate" {
            spell.caption = "Inoculate".to_owned();
            spelling_patch_applied = true;
        } else if index > 0 && spell.caption == "Tonsilectomy" {
            spell.caption = "Tonsillectomy".to_owned();
            spelling_patch_applied = true;
        }
    }

    let current_task = optional_ascii(&mapped.current_task, "fTask.Caption")?;
    if !valid_current_task(&current_task) {
        return Err(DesktopValidationError::InvalidTask);
    }
    let queue = mapped
        .queue
        .iter()
        .map(|command| parse_queue_command(ascii(command, "fQueue.Items")?))
        .collect::<Result<Vec<_>, _>>()?;
    let activity = optional_ascii(&mapped.activity, "Kill.SimpleText")?;

    let quest_marker = optional_ascii(&mapped.quest_marker, "fQuest.Caption")?;
    let quest = if quest_marker.is_empty() {
        DesktopQuestMarker::None
    } else {
        let index = nonnegative_index(mapped.quest_index.unwrap_or(0), "fQuest.Tag")?;
        if index >= crate::desktop_rules::bundled().tables.monsters.len() {
            return Err(DesktopValidationError::InvalidIndex("fQuest.Tag"));
        }
        if quest_marker == "fQuest" {
            DesktopQuestMarker::LegacyPlaceholder { index }
        } else {
            DesktopQuestMarker::Value {
                marker: quest_marker,
                index,
            }
        }
    };

    let prized_equipment = nonnegative_index(mapped.prized_equipment.unwrap_or(0), "Equips.Tag")?;
    if prized_equipment >= equipment.len() {
        return Err(DesktopValidationError::InvalidIndex("Equips.Tag"));
    }
    let game_style = u32::try_from(mapped.game_style.unwrap_or(3))
        .map_err(|_| DesktopValidationError::InvalidNumber("GameStyle.Tag"))?;
    if game_style != 3 {
        return Err(DesktopValidationError::InvalidNumber("GameStyle.Tag"));
    }

    let passkey = i32::try_from(mapped.private.passkey.unwrap_or(0))
        .map_err(|_| DesktopValidationError::InvalidNumber("Traits.Tag"))?;
    if passkey < 0 {
        return Err(DesktopValidationError::InvalidNumber("Traits.Tag"));
    }
    let passkey_text = optional_ascii(&mapped.private.passkey_text, "Traits.Hint")?;
    if !passkey_text.is_empty()
        && passkey_text
            .parse::<i32>()
            .ok()
            .filter(|value| *value == passkey)
            .is_none()
    {
        return Err(DesktopValidationError::PasskeyMismatch);
    }

    let legacy_prologue_62 = current_task.is_empty()
        && activity == "Loading...."
        && queue.last().is_some_and(|command| {
            command.kind == DesktopQueueKind::Task
                && command.duration_seconds == 2
                && command.caption == "Loading"
        });
    let legacy_quest_placeholder = matches!(quest, DesktopQuestMarker::LegacyPlaceholder { .. });

    Ok(DesktopValidatedSave {
        traits,
        stats,
        equipment,
        inventory,
        spells,
        plots,
        quests,
        current_task,
        quest,
        queue,
        activity,
        bars: DesktopValidatedBars {
            experience: validated_bar(&mapped.bars.experience, "ExpBar")?,
            encumbrance: validated_bar(&mapped.bars.encumbrance, "EncumBar")?,
            plot: validated_bar(&mapped.bars.plot, "PlotBar")?,
            quest: validated_bar(&mapped.bars.quest, "QuestBar")?,
            task: validated_bar(&mapped.bars.task, "TaskBar")?,
        },
        prized_equipment,
        game_style,
        profile: DesktopValidatedProfile {
            motto: optional_ascii(&mapped.profile.motto, "Stats.Hint")?,
            guild: optional_ascii(&mapped.profile.guild, "Label1.Hint")?,
        },
        private: DesktopValidatedPrivateMetadata {
            passkey,
            realm: optional_ascii(&mapped.private.realm, "Spells.Hint")?,
            endpoint: optional_ascii(&mapped.private.endpoint, "Equips.Hint")?,
            account: optional_ascii(&mapped.private.account, "Inventory.Hint")?,
            password: optional_ascii(&mapped.private.password, "Plots.Hint")?,
        },
        adaptations: DesktopAdaptations {
            legacy_prologue_62,
            legacy_quest_placeholder,
            spelling_patch_applied,
        },
    })
}

fn validated_rows(
    rows: &[GameListRow],
    field: &'static str,
) -> Result<Vec<DesktopValidatedRow>, DesktopValidationError> {
    rows.iter()
        .map(|row| {
            Ok(DesktopValidatedRow {
                header: row.header,
                caption: ascii(&row.caption, field)?.to_owned(),
                subitems: row
                    .subitems
                    .iter()
                    .map(|value| ascii(value, field).map(str::to_owned))
                    .collect::<Result<Vec<_>, _>>()?,
            })
        })
        .collect()
}

fn ascii<'a>(bytes: &'a [u8], field: &'static str) -> Result<&'a str, DesktopValidationError> {
    if !bytes.is_ascii() {
        return Err(DesktopValidationError::UnsupportedEncoding(field));
    }
    std::str::from_utf8(bytes).map_err(|_| DesktopValidationError::UnsupportedEncoding(field))
}

fn optional_ascii(
    bytes: &Option<Vec<u8>>,
    field: &'static str,
) -> Result<String, DesktopValidationError> {
    bytes
        .as_deref()
        .map(|bytes| ascii(bytes, field).map(str::to_owned))
        .transpose()
        .map(Option::unwrap_or_default)
}

fn require_layout(
    rows: &[DesktopValidatedRow],
    expected: &[&str],
    field: &'static str,
) -> Result<(), DesktopValidationError> {
    if rows.len() != expected.len()
        || rows
            .iter()
            .map(|row| row.caption.as_str())
            .ne(expected.iter().copied())
    {
        return Err(DesktopValidationError::InvalidLayout(field));
    }
    Ok(())
}

fn require_one_subitem(
    rows: &[DesktopValidatedRow],
    field: &'static str,
) -> Result<(), DesktopValidationError> {
    if rows.iter().any(|row| row.subitems.len() != 1) {
        return Err(DesktopValidationError::InvalidLayout(field));
    }
    Ok(())
}

fn normalize_optional_single_subitem(
    rows: &mut [DesktopValidatedRow],
    field: &'static str,
) -> Result<(), DesktopValidationError> {
    for row in rows {
        match row.subitems.len() {
            0 => row.subitems.push(String::new()),
            1 => {}
            _ => return Err(DesktopValidationError::InvalidLayout(field)),
        }
    }
    Ok(())
}

fn parse_nonnegative(value: &str, field: &'static str) -> Result<u32, DesktopValidationError> {
    value
        .parse::<u32>()
        .map_err(|_| DesktopValidationError::InvalidNumber(field))
}

fn parse_positive(value: &str, field: &'static str) -> Result<u32, DesktopValidationError> {
    let value = parse_nonnegative(value, field)?;
    if value == 0 {
        return Err(DesktopValidationError::InvalidNumber(field));
    }
    Ok(value)
}

fn nonnegative_index(value: i64, field: &'static str) -> Result<usize, DesktopValidationError> {
    usize::try_from(value).map_err(|_| DesktopValidationError::InvalidIndex(field))
}

fn validated_bar(
    bar: &DesktopMappedBar,
    field: &'static str,
) -> Result<DesktopValidatedBar, DesktopValidationError> {
    let position = u64::try_from(bar.position.unwrap_or(0))
        .map_err(|_| DesktopValidationError::InvalidNumber(field))?;
    let maximum = u64::try_from(bar.maximum.ok_or(DesktopValidationError::Missing(field))?)
        .map_err(|_| DesktopValidationError::InvalidNumber(field))?;
    if position > maximum {
        return Err(DesktopValidationError::InvalidNumber(field));
    }
    Ok(DesktopValidatedBar { position, maximum })
}

fn parse_queue_command(command: &str) -> Result<DesktopQueueCommand, DesktopValidationError> {
    let mut parts = command.split('|');
    let kind = match parts.next() {
        Some("task") => DesktopQueueKind::Task,
        Some("plot") => DesktopQueueKind::Plot,
        _ => return Err(DesktopValidationError::InvalidCommand),
    };
    let duration_seconds = parts
        .next()
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|value| *value > 0)
        .ok_or(DesktopValidationError::InvalidCommand)?;
    let caption = parts
        .next()
        .filter(|value| !value.is_empty())
        .ok_or(DesktopValidationError::InvalidCommand)?
        .to_owned();
    if parts.next().is_some() {
        return Err(DesktopValidationError::InvalidCommand);
    }
    Ok(DesktopQueueCommand {
        kind,
        duration_seconds,
        caption,
    })
}

fn valid_current_task(task: &str) -> bool {
    if matches!(task, "" | "load" | "market" | "sell" | "buying" | "heading") {
        return true;
    }
    let mut parts = task.split('|');
    parts.next() == Some("kill")
        && parts.next().is_some_and(|value| !value.is_empty())
        && parts
            .next()
            .and_then(|value| value.parse::<u32>().ok())
            .is_some()
        && parts.next().is_some_and(|value| !value.is_empty())
        && parts.next().is_none()
}

fn is_canonical_roman(value: &str) -> bool {
    if value.is_empty() || !value.bytes().all(|byte| b"IVXLCDMAT".contains(&byte)) {
        return false;
    }
    let mut rest = value;
    let mut total = 0_u32;
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
            total += amount;
            rest = &rest[token.len()..];
            matched = true;
        }
    }
    total > 0 && rest.is_empty() && desktop_roman(total) == value
}

fn desktop_roman(mut value: u32) -> String {
    let mut encoded = String::new();
    for (token, amount) in [
        ("T", 10_000),
        ("MT", 9_000),
        ("A", 5_000),
        ("MA", 4_000),
        ("M", 1_000),
        ("CM", 900),
        ("D", 500),
        ("CD", 400),
        ("C", 100),
        ("XC", 90),
        ("L", 50),
        ("XL", 40),
        ("X", 10),
        ("IX", 9),
        ("V", 5),
        ("IV", 4),
        ("I", 1),
    ] {
        while value >= amount {
            encoded.push_str(token);
            value -= amount;
        }
    }
    encoded
}

pub fn decode_game_list(bytes: &[u8]) -> Result<Vec<GameListRow>, GameListError> {
    let primary = decode_game_list_with_subitem_word(bytes, 3);
    let alternate = decode_game_list_with_subitem_word(bytes, 4);
    match (primary, alternate) {
        (Ok(primary), Ok(alternate)) if primary == alternate => Ok(primary),
        (Ok(_), Ok(_)) => Err(GameListError::SubitemCount),
        (Ok(rows), Err(_)) | (Err(_), Ok(rows)) => Ok(rows),
        (Err(primary), Err(_)) => Err(primary),
    }
}

fn decode_game_list_with_subitem_word(
    bytes: &[u8],
    subitem_word: usize,
) -> Result<Vec<GameListRow>, GameListError> {
    let mut reader = GameListReader { bytes, position: 0 };
    let declared_length = reader.u32()? as usize;
    if declared_length != bytes.len() {
        return Err(GameListError::TotalLength);
    }
    let row_count = reader.u32()? as usize;
    if row_count > MAX_GAME_LIST_ROWS || row_count > reader.remaining() / 21 {
        return Err(GameListError::RowCount);
    }
    let mut rows = Vec::with_capacity(row_count);
    let mut aggregate_rows = row_count;
    for _ in 0..row_count {
        let header = [
            reader.i32()?,
            reader.i32()?,
            reader.i32()?,
            reader.i32()?,
            reader.i32()?,
        ];
        let data_word = if subitem_word == 3 { 4 } else { 3 };
        if header[data_word] != 0 {
            return Err(GameListError::SubitemCount);
        }
        let subitem_count =
            usize::try_from(header[subitem_word]).map_err(|_| GameListError::SubitemCount)?;
        aggregate_rows = aggregate_rows
            .checked_add(subitem_count)
            .ok_or(GameListError::RowLimit)?;
        if aggregate_rows > MAX_GAME_LIST_ROWS || subitem_count > reader.remaining() / 3 {
            return Err(GameListError::RowLimit);
        }
        let caption = reader.short_bytes()?;
        let mut subitems = Vec::with_capacity(subitem_count);
        for _ in 0..subitem_count {
            subitems.push(reader.short_bytes()?);
        }
        rows.push(GameListRow {
            header,
            caption,
            subitems,
        });
    }
    let trailer_bytes = aggregate_rows
        .checked_sub(row_count)
        .and_then(|count| count.checked_mul(2))
        .ok_or(GameListError::RowLimit)?;
    reader.take(trailer_bytes)?;
    if reader.remaining() != 0 {
        return Err(GameListError::TrailingBytes);
    }
    Ok(rows)
}

struct GameListReader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl GameListReader<'_> {
    fn remaining(&self) -> usize {
        self.bytes.len() - self.position
    }

    fn u32(&mut self) -> Result<u32, GameListError> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    fn i32(&mut self) -> Result<i32, GameListError> {
        Ok(i32::from_le_bytes(self.array()?))
    }

    fn short_bytes(&mut self) -> Result<Vec<u8>, GameListError> {
        let length = usize::from(self.byte()?);
        Ok(self.take(length)?.to_vec())
    }

    fn byte(&mut self) -> Result<u8, GameListError> {
        let byte = self
            .bytes
            .get(self.position)
            .copied()
            .ok_or(GameListError::Truncated {
                offset: self.position,
            })?;
        self.position += 1;
        Ok(byte)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], GameListError> {
        Ok(self.take(N)?.try_into().expect("length was checked"))
    }

    fn take(&mut self, length: usize) -> Result<&[u8], GameListError> {
        let end = self
            .position
            .checked_add(length)
            .ok_or(GameListError::Truncated {
                offset: self.position,
            })?;
        let value = self
            .bytes
            .get(self.position..end)
            .ok_or(GameListError::Truncated {
                offset: self.position,
            })?;
        self.position = end;
        Ok(value)
    }
}

struct Parser<'a> {
    bytes: &'a [u8],
    position: usize,
    component_count: usize,
    component_names: HashSet<String>,
}

impl Parser<'_> {
    fn component(
        &mut self,
        depth: usize,
        root: bool,
    ) -> Result<DesktopComponent, DesktopParseError> {
        if depth > MAX_COMPONENT_DEPTH {
            return Err(DesktopParseError::DepthLimit);
        }
        self.component_count += 1;
        if self.component_count > MAX_COMPONENTS {
            return Err(DesktopParseError::ComponentLimit);
        }
        if root {
            let offset = self.position;
            if self.take(4)? != b"TPF0" {
                return Err(DesktopParseError::InvalidSignature { offset });
            }
        }
        self.prefix()?;
        let class_name = self.short_ascii()?;
        let name = self.short_ascii()?;
        if !name.is_empty() && !self.component_names.insert(name.to_ascii_lowercase()) {
            return Err(DesktopParseError::DuplicateComponent);
        }
        let properties = self.properties()?;
        let mut children = Vec::new();
        loop {
            if self.peek()? == 0 {
                self.position += 1;
                break;
            }
            children.push(self.component(depth + 1, false)?);
        }
        Ok(DesktopComponent {
            class_name,
            name,
            properties,
            children,
        })
    }

    fn prefix(&mut self) -> Result<(), DesktopParseError> {
        if self.peek()? & 0xf0 != 0xf0 {
            return Ok(());
        }
        let offset = self.position;
        let flags = self.byte()? & 0x0f;
        if flags & !0x07 != 0 {
            return Err(DesktopParseError::InvalidPrefix { offset });
        }
        if flags & 0x02 != 0 {
            match self.byte()? {
                2 => {
                    self.take(1)?;
                }
                3 => {
                    self.take(2)?;
                }
                4 => {
                    self.take(4)?;
                }
                _ => return Err(DesktopParseError::InvalidPrefix { offset }),
            }
        }
        Ok(())
    }

    fn properties(&mut self) -> Result<Vec<DesktopProperty>, DesktopParseError> {
        let mut properties = Vec::new();
        let mut names = HashSet::new();
        loop {
            let name = self.short_ascii()?;
            if name.is_empty() {
                break;
            }
            if !names.insert(name.to_ascii_lowercase()) {
                return Err(DesktopParseError::DuplicateProperty);
            }
            properties.push(DesktopProperty {
                name,
                value: self.value()?,
            });
        }
        Ok(properties)
    }

    fn value(&mut self) -> Result<DesktopValue, DesktopParseError> {
        let offset = self.position;
        match self.byte()? {
            0 => Ok(DesktopValue::Null),
            1 => {
                let mut values = Vec::new();
                while self.peek()? != 0 {
                    values.push(self.value()?);
                }
                self.position += 1;
                Ok(DesktopValue::List(values))
            }
            2 => Ok(DesktopValue::Integer(i64::from(self.byte()? as i8))),
            3 => Ok(DesktopValue::Integer(i64::from(i16::from_le_bytes(
                self.array()?,
            )))),
            4 => Ok(DesktopValue::Integer(i64::from(i32::from_le_bytes(
                self.array()?,
            )))),
            5 => Ok(DesktopValue::Extended(self.array()?)),
            6 => Ok(DesktopValue::String(self.short_bytes()?)),
            7 => Ok(DesktopValue::Ident(self.short_bytes()?)),
            8 => Ok(DesktopValue::Boolean(false)),
            9 => Ok(DesktopValue::Boolean(true)),
            10 => Ok(DesktopValue::Binary(self.long_bytes(1)?)),
            11 => {
                let mut values = Vec::new();
                loop {
                    let value = self.short_bytes()?;
                    if value.is_empty() {
                        break;
                    }
                    values.push(value);
                }
                Ok(DesktopValue::Set(values))
            }
            12 => Ok(DesktopValue::String(self.long_bytes(1)?)),
            13 => Ok(DesktopValue::Nil),
            14 => Ok(DesktopValue::Collection(self.collection()?)),
            15 => Ok(DesktopValue::Single(self.array()?)),
            16 => Ok(DesktopValue::Currency(self.array()?)),
            17 => Ok(DesktopValue::Date(self.array()?)),
            18 => Ok(DesktopValue::WideString(self.long_bytes(2)?)),
            19 => Ok(DesktopValue::Integer(i64::from_le_bytes(self.array()?))),
            20 => Ok(DesktopValue::Utf8String(self.long_bytes(1)?)),
            21 => Ok(DesktopValue::Double(self.array()?)),
            tag => Err(DesktopParseError::UnsupportedValue { offset, tag }),
        }
    }

    fn collection(&mut self) -> Result<Vec<DesktopCollectionItem>, DesktopParseError> {
        let mut items = Vec::new();
        while self.peek()? != 0 {
            let order = match self.peek()? {
                2..=4 => match self.value()? {
                    DesktopValue::Integer(value) => Some(value),
                    _ => unreachable!(),
                },
                _ => None,
            };
            let offset = self.position;
            if self.byte()? != 1 {
                return Err(DesktopParseError::InvalidValue { offset });
            }
            items.push(DesktopCollectionItem {
                order,
                properties: self.properties()?,
            });
        }
        self.position += 1;
        Ok(items)
    }

    fn short_ascii(&mut self) -> Result<String, DesktopParseError> {
        let offset = self.position;
        let bytes = self.short_bytes()?;
        if !bytes.is_ascii() {
            return Err(DesktopParseError::InvalidValue { offset });
        }
        Ok(String::from_utf8(bytes).expect("ASCII was checked"))
    }

    fn short_bytes(&mut self) -> Result<Vec<u8>, DesktopParseError> {
        let length = usize::from(self.byte()?);
        Ok(self.take(length)?.to_vec())
    }

    fn long_bytes(&mut self, width: usize) -> Result<Vec<u8>, DesktopParseError> {
        let offset = self.position;
        let units = u32::from_le_bytes(self.array()?) as usize;
        let length = units
            .checked_mul(width)
            .ok_or(DesktopParseError::ValueLimit { offset })?;
        if length > MAX_VALUE_BYTES {
            return Err(DesktopParseError::ValueLimit { offset });
        }
        Ok(self.take(length)?.to_vec())
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], DesktopParseError> {
        Ok(self.take(N)?.try_into().expect("length was checked"))
    }

    fn peek(&self) -> Result<u8, DesktopParseError> {
        self.bytes
            .get(self.position)
            .copied()
            .ok_or(DesktopParseError::Truncated {
                offset: self.position,
            })
    }

    fn byte(&mut self) -> Result<u8, DesktopParseError> {
        let value = self.peek()?;
        self.position += 1;
        Ok(value)
    }

    fn take(&mut self, length: usize) -> Result<&[u8], DesktopParseError> {
        let end = self
            .position
            .checked_add(length)
            .ok_or(DesktopParseError::Truncated {
                offset: self.position,
            })?;
        let value = self
            .bytes
            .get(self.position..end)
            .ok_or(DesktopParseError::Truncated {
                offset: self.position,
            })?;
        self.position = end;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        compatibility::{DesktopCanonicalState, DesktopImportMetadata},
        desktop_protocol::{
            DesktopAccountAuthentication, DesktopReportOperation, guild_request, report,
        },
    };

    fn short(value: &str) -> Vec<u8> {
        let mut bytes = vec![value.len() as u8];
        bytes.extend_from_slice(value.as_bytes());
        bytes
    }

    fn component(name: &str, properties: Vec<(&str, Vec<u8>)>, children: Vec<Vec<u8>>) -> Vec<u8> {
        let mut bytes = b"TPF0".to_vec();
        bytes.extend(short("TLabel"));
        bytes.extend(short(name));
        for (name, value) in properties {
            bytes.extend(short(name));
            bytes.extend(value);
        }
        bytes.push(0);
        for child in children {
            bytes.extend_from_slice(&child[4..]);
        }
        bytes.push(0);
        bytes
    }

    fn nested(depth: usize) -> Vec<u8> {
        let mut current = component(&format!("C{depth}"), vec![], vec![]);
        for level in (1..depth).rev() {
            current = component(&format!("C{level}"), vec![], vec![current]);
        }
        current
    }

    fn game_list(rows: &[(&str, &[&str])]) -> Vec<u8> {
        game_list_with_subitem_word(rows, 3)
    }

    fn game_list_with_subitem_word(rows: &[(&str, &[&str])], subitem_word: usize) -> Vec<u8> {
        let mut bytes = vec![0; 4];
        bytes.extend_from_slice(&(rows.len() as u32).to_le_bytes());
        for (caption, subitems) in rows {
            let mut header = [0_i32, -1, -1, 0, 0];
            header[subitem_word] = subitems.len() as i32;
            for value in header {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
            bytes.push(caption.len() as u8);
            bytes.extend_from_slice(caption.as_bytes());
            for subitem in *subitems {
                bytes.push(subitem.len() as u8);
                bytes.extend_from_slice(subitem.as_bytes());
            }
        }
        for (_, subitems) in rows {
            bytes.extend(std::iter::repeat_n(0, subitems.len() * 2));
        }
        let length = bytes.len() as u32;
        bytes[..4].copy_from_slice(&length.to_le_bytes());
        bytes
    }

    fn mapped_component(
        name: &str,
        class_name: &str,
        properties: Vec<(&str, DesktopValue)>,
    ) -> DesktopComponent {
        DesktopComponent {
            class_name: class_name.to_owned(),
            name: name.to_owned(),
            properties: properties
                .into_iter()
                .map(|(name, value)| DesktopProperty {
                    name: name.to_owned(),
                    value,
                })
                .collect(),
            children: Vec::new(),
        }
    }

    fn list_value(rows: &[(&str, &[&str])]) -> DesktopValue {
        DesktopValue::Binary(game_list(rows))
    }

    fn text(value: &str) -> DesktopValue {
        DesktopValue::String(value.as_bytes().to_vec())
    }

    fn mapped_row(caption: &[u8], value: &[u8]) -> GameListRow {
        GameListRow {
            header: [0, 0, 0, 0, 1],
            caption: caption.to_vec(),
            subitems: vec![value.to_vec()],
        }
    }

    fn encoded_document(document: &DesktopDocument) -> Vec<u8> {
        document
            .components
            .iter()
            .flat_map(encoded_component)
            .collect()
    }

    fn encoded_component(component: &DesktopComponent) -> Vec<u8> {
        let mut bytes = b"TPF0".to_vec();
        bytes.extend(short(&component.class_name));
        bytes.extend(short(&component.name));
        for property in &component.properties {
            bytes.extend(short(&property.name));
            bytes.extend(encoded_value(&property.value));
        }
        bytes.extend([0, 0]);
        bytes
    }

    fn encoded_value(value: &DesktopValue) -> Vec<u8> {
        match value {
            DesktopValue::Integer(value) => {
                [vec![4], (*value as i32).to_le_bytes().to_vec()].concat()
            }
            DesktopValue::String(value) => {
                let mut bytes = vec![6, value.len() as u8];
                bytes.extend(value);
                bytes
            }
            DesktopValue::Binary(value) => {
                let mut bytes = vec![10];
                bytes.extend_from_slice(&(value.len() as u32).to_le_bytes());
                bytes.extend(value);
                bytes
            }
            DesktopValue::List(values) => {
                let mut bytes = vec![1];
                for value in values {
                    bytes.extend(encoded_value(value));
                }
                bytes.push(0);
                bytes
            }
            _ => panic!("synthetic desktop builder received an unsupported value"),
        }
    }

    fn mapped_document(final_queue: &str) -> DesktopDocument {
        let one: &[&str] = &["Synthetic"];
        let level: &[&str] = &["2"];
        let rank_one: &[&str] = &["I"];
        let rank_two: &[&str] = &["II"];
        let quantity: &[&str] = &["3"];
        let completed: &[&str] = &[];
        DesktopDocument {
            components: vec![
                mapped_component(
                    "Traits",
                    "TListView",
                    vec![
                        (
                            "Items.Data",
                            list_value(&[
                                ("Name", one),
                                ("Race", one),
                                ("Class", one),
                                ("Level", level),
                            ]),
                        ),
                        ("Tag", DesktopValue::Integer(4_242)),
                        ("Hint", text("4242")),
                    ],
                ),
                mapped_component(
                    "Stats",
                    "TListView",
                    vec![
                        (
                            "Items.Data",
                            list_value(&[
                                ("STR", &["12"]),
                                ("CON", &["11"]),
                                ("DEX", &["10"]),
                                ("INT", &["9"]),
                                ("WIS", &["8"]),
                                ("CHA", &["7"]),
                                ("HP Max", &["20"]),
                                ("MP Max", &["15"]),
                            ]),
                        ),
                        ("Hint", text("Synthetic motto")),
                    ],
                ),
                mapped_component(
                    "Equips",
                    "TListView",
                    vec![
                        (
                            "Items.Data",
                            list_value(&[
                                ("Weapon", &["Stick"]),
                                ("Shield", &["Plate"]),
                                ("Helm", &["Cap"]),
                                ("Hauberk", &["Burlap"]),
                                ("Brassairts", &["Cloth"]),
                                ("Vambraces", &["Cloth"]),
                                ("Gauntlets", &["Cloth"]),
                                ("Gambeson", &["Cloth"]),
                                ("Cuisses", &["Cloth"]),
                                ("Greaves", &["Cloth"]),
                                ("Sollerets", &["Cloth"]),
                            ]),
                        ),
                        ("Tag", DesktopValue::Integer(0)),
                        ("Hint", text("http://synthetic.invalid/?")),
                    ],
                ),
                mapped_component(
                    "Inventory",
                    "TListView",
                    vec![
                        ("Items.Data", list_value(&[("Gold", quantity)])),
                        ("Hint", text("synthetic-account")),
                    ],
                ),
                mapped_component(
                    "Spells",
                    "TListView",
                    vec![
                        (
                            "Items.Data",
                            list_value(&[("Gyp", rank_one), ("Shoelaces", rank_two)]),
                        ),
                        ("Hint", text("Synthetic Realm")),
                    ],
                ),
                mapped_component(
                    "Plots",
                    "TListView",
                    vec![
                        (
                            "Items.Data",
                            list_value(&[("Prologue", completed), ("Act I", completed)]),
                        ),
                        ("Hint", text("synthetic-password")),
                    ],
                ),
                mapped_component(
                    "Quests",
                    "TListView",
                    vec![("Items.Data", list_value(&[("Synthetic quest", completed)]))],
                ),
                mapped_component("fTask", "TLabel", vec![("Caption", text("load"))]),
                mapped_component(
                    "fQuest",
                    "TLabel",
                    vec![
                        ("Caption", text("fQuest")),
                        ("Tag", DesktopValue::Integer(1)),
                    ],
                ),
                mapped_component(
                    "fQueue",
                    "TListBox",
                    vec![(
                        "Items.Strings",
                        DesktopValue::List(vec![
                            text("task|10|Synthetic vision"),
                            text(final_queue),
                        ]),
                    )],
                ),
                mapped_component(
                    "Kill",
                    "TStatusBar",
                    vec![("SimpleText", text("Loading..."))],
                ),
                mapped_component(
                    "ExpBar",
                    "TProgressBar",
                    vec![
                        ("Position", DesktopValue::Integer(12)),
                        ("Max", DesktopValue::Integer(100)),
                        ("Hint", text("stale experience hint")),
                    ],
                ),
                mapped_component(
                    "EncumBar",
                    "TProgressBar",
                    vec![
                        ("Position", DesktopValue::Integer(3)),
                        ("Max", DesktopValue::Integer(50)),
                    ],
                ),
                mapped_component(
                    "PlotBar",
                    "TProgressBar",
                    vec![
                        ("Position", DesktopValue::Integer(0)),
                        ("Max", DesktopValue::Integer(26)),
                        ("Hint", text("complete")),
                    ],
                ),
                mapped_component(
                    "QuestBar",
                    "TProgressBar",
                    vec![
                        ("Position", DesktopValue::Integer(0)),
                        ("Max", DesktopValue::Integer(75)),
                        ("Hint", text("100% complete")),
                    ],
                ),
                mapped_component(
                    "TaskBar",
                    "TProgressBar",
                    vec![
                        ("Position", DesktopValue::Integer(2_000)),
                        ("Max", DesktopValue::Integer(2_000)),
                    ],
                ),
                mapped_component(
                    "InventoryLabelAlsoGameStyle",
                    "TLabel",
                    vec![("Tag", DesktopValue::Integer(3))],
                ),
                mapped_component("Label1", "TLabel", vec![("Hint", text("Synthetic Guild"))]),
                mapped_component(
                    "Timer1",
                    "TTimer",
                    vec![("Tag", DesktopValue::Integer(987_654))],
                ),
            ],
        }
    }

    #[test]
    fn parses_concatenated_components_and_supported_values() {
        let first = component(
            "First",
            vec![
                ("Caption", [vec![6], short("Synthetic")].concat()),
                ("Tag", vec![4, 42, 0, 0, 0]),
                ("Visible", vec![9]),
                (
                    "Data",
                    [vec![10], 3_u32.to_le_bytes().to_vec(), vec![1, 2, 3]].concat(),
                ),
            ],
            vec![],
        );
        let second = component("Second", vec![("Items", vec![1, 2, 7, 0])], vec![]);
        let document = parse_components(&[first, second].concat()).unwrap();
        assert_eq!(document.components.len(), 2);
        assert_eq!(document.components[0].name, "First");
        assert_eq!(
            document.components[0].properties[1].value,
            DesktopValue::Integer(42)
        );
        assert_eq!(
            document.components[1].properties[0].value,
            DesktopValue::List(vec![DesktopValue::Integer(7)])
        );
    }

    #[test]
    fn accepts_declared_component_depth_and_count_limits() {
        assert!(parse_components(&nested(MAX_COMPONENT_DEPTH)).is_ok());
        let roots = (0..MAX_COMPONENTS)
            .flat_map(|index| component(&format!("C{index}"), vec![], vec![]))
            .collect::<Vec<_>>();
        assert!(parse_components(&roots).is_ok());
    }

    #[test]
    fn rejects_values_over_depth_and_component_limits() {
        assert_eq!(
            parse_components(&nested(MAX_COMPONENT_DEPTH + 1)).unwrap_err(),
            DesktopParseError::DepthLimit
        );
        let roots = (0..=MAX_COMPONENTS)
            .flat_map(|index| component(&format!("C{index}"), vec![], vec![]))
            .collect::<Vec<_>>();
        assert_eq!(
            parse_components(&roots).unwrap_err(),
            DesktopParseError::ComponentLimit
        );
    }

    #[test]
    fn accepts_exact_length_limit_and_rejects_one_byte_over() {
        let exact = component(
            "Exact",
            vec![(
                "Data",
                [
                    vec![10],
                    (MAX_VALUE_BYTES as u32).to_le_bytes().to_vec(),
                    vec![0; MAX_VALUE_BYTES],
                ]
                .concat(),
            )],
            vec![],
        );
        assert!(parse_components(&exact).is_ok());

        let over = component(
            "Over",
            vec![(
                "Data",
                [
                    vec![10],
                    ((MAX_VALUE_BYTES + 1) as u32).to_le_bytes().to_vec(),
                ]
                .concat(),
            )],
            vec![],
        );
        assert!(matches!(
            parse_components(&over),
            Err(DesktopParseError::ValueLimit { .. })
        ));
    }

    #[test]
    fn rejects_truncation_unknown_tags_duplicates_and_trailing_bytes() {
        let valid = component("Only", vec![("Tag", vec![2, 1])], vec![]);
        assert!(matches!(
            parse_components(&valid[..valid.len() - 1]),
            Err(DesktopParseError::Truncated { .. })
        ));
        assert!(matches!(
            parse_components(&component("Unknown", vec![("Value", vec![99])], vec![])),
            Err(DesktopParseError::UnsupportedValue { tag: 99, .. })
        ));
        assert_eq!(
            parse_components(&component(
                "DuplicateProperty",
                vec![("Tag", vec![2, 1]), ("tag", vec![2, 2])],
                vec![],
            ))
            .unwrap_err(),
            DesktopParseError::DuplicateProperty
        );
        assert_eq!(
            parse_components(
                &[
                    component("Duplicate", vec![], vec![]),
                    component("duplicate", vec![], vec![]),
                ]
                .concat()
            )
            .unwrap_err(),
            DesktopParseError::DuplicateComponent
        );
        let mut trailing = valid;
        trailing.push(1);
        assert!(matches!(
            parse_components(&trailing),
            Err(DesktopParseError::InvalidSignature { .. })
                | Err(DesktopParseError::Truncated { .. })
        ));
    }

    #[test]
    fn rejects_invalid_prefix_and_collection_framing() {
        let mut prefix = b"TPF0".to_vec();
        prefix.push(0xf8);
        prefix.extend(short("TLabel"));
        prefix.extend(short("Prefix"));
        prefix.extend([0, 0]);
        assert!(matches!(
            parse_components(&prefix),
            Err(DesktopParseError::InvalidPrefix { .. })
        ));
        assert!(matches!(
            parse_components(&component(
                "Collection",
                vec![("Items", vec![14, 9])],
                vec![],
            )),
            Err(DesktopParseError::InvalidValue { .. })
        ));
    }

    #[test]
    fn decodes_ordered_empty_and_populated_game_lists() {
        assert_eq!(decode_game_list(&game_list(&[])).unwrap(), vec![]);
        let empty: &[&str] = &[];
        let spell: &[&str] = &["II", "selected"];
        let rows = decode_game_list(&game_list(&[("Name", empty), ("Spell", spell)])).unwrap();
        assert_eq!(
            rows.iter()
                .map(|row| String::from_utf8(row.caption.clone()).unwrap())
                .collect::<Vec<_>>(),
            ["Name", "Spell"]
        );
        assert_eq!(rows[1].subitems, [b"II".to_vec(), b"selected".to_vec()]);
        assert_eq!(rows[1].header[3], 2);
    }

    #[test]
    fn decodes_observed_game_lists_with_the_alternate_subitem_word() {
        let rows = decode_game_list(&game_list_with_subitem_word(
            &[("Name", &["Disposable"]), ("Plot", &[])],
            4,
        ))
        .unwrap();
        assert_eq!(rows[0].subitems, [b"Disposable".to_vec()]);
        assert!(rows[1].subitems.is_empty());
        assert_eq!(rows[0].header[3], 0);
        assert_eq!(rows[0].header[4], 1);
    }

    #[test]
    fn normalizes_omitted_fresh_equipment_values_but_rejects_multiple_values() {
        let mut rows = vec![
            DesktopValidatedRow {
                header: [0, -1, -1, 0, 0],
                caption: "Shield".to_owned(),
                subitems: Vec::new(),
            },
            DesktopValidatedRow {
                header: [0, -1, -1, 1, 0],
                caption: "Weapon".to_owned(),
                subitems: vec!["Sharp Stick".to_owned()],
            },
        ];
        normalize_optional_single_subitem(&mut rows, "Equips").unwrap();
        assert_eq!(rows[0].subitems, [""]);
        assert_eq!(rows[1].subitems, ["Sharp Stick"]);

        rows[0].subitems = vec!["one".to_owned(), "two".to_owned()];
        assert_eq!(
            normalize_optional_single_subitem(&mut rows, "Equips").unwrap_err(),
            DesktopValidationError::InvalidLayout("Equips")
        );
    }

    #[test]
    fn rejects_inconsistent_lengths_counts_and_premature_exhaustion() {
        let mut inconsistent = game_list(&[("Name", &[])]);
        inconsistent[..4].copy_from_slice(&1_u32.to_le_bytes());
        assert_eq!(
            decode_game_list(&inconsistent).unwrap_err(),
            GameListError::TotalLength
        );

        let mut invalid_rows = game_list(&[]);
        invalid_rows[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(
            decode_game_list(&invalid_rows).unwrap_err(),
            GameListError::RowCount
        );

        let mut negative_subitems = game_list(&[("Name", &[])]);
        negative_subitems[20..24].copy_from_slice(&(-1_i32).to_le_bytes());
        assert_eq!(
            decode_game_list(&negative_subitems).unwrap_err(),
            GameListError::SubitemCount
        );

        let populated = game_list(&[("Spell", &["II"])]);
        let truncated = &populated[..populated.len() - 1];
        assert_eq!(
            decode_game_list(truncated).unwrap_err(),
            GameListError::TotalLength
        );

        let mut declared_truncation = truncated.to_vec();
        let length = declared_truncation.len() as u32;
        declared_truncation[..4].copy_from_slice(&length.to_le_bytes());
        assert!(matches!(
            decode_game_list(&declared_truncation),
            Err(GameListError::Truncated { .. })
        ));
    }

    #[test]
    fn rejects_aggregate_game_list_row_overflow_and_trailing_bytes() {
        let mut excessive = game_list(&[("Name", &[])]);
        excessive[20..24].copy_from_slice(&(MAX_GAME_LIST_ROWS as i32).to_le_bytes());
        assert_eq!(
            decode_game_list(&excessive).unwrap_err(),
            GameListError::RowLimit
        );

        let mut trailing = game_list(&[]);
        trailing.push(0);
        let length = trailing.len() as u32;
        trailing[..4].copy_from_slice(&length.to_le_bytes());
        assert_eq!(
            decode_game_list(&trailing).unwrap_err(),
            GameListError::TrailingBytes
        );
    }

    #[test]
    fn maps_supported_desktop_components_and_private_metadata() {
        let mapped = map_desktop_document(&mapped_document("plot|2|Loading")).unwrap();
        assert_eq!(mapped.traits[0].caption, b"Name");
        assert_eq!(mapped.spells[0].caption, b"Gyp");
        assert_eq!(mapped.spells[1].caption, b"Shoelaces");
        assert_eq!(mapped.current_task, Some(b"load".to_vec()));
        assert_eq!(mapped.quest_marker, Some(b"fQuest".to_vec()));
        assert_eq!(mapped.quest_index, Some(1));
        assert_eq!(mapped.queue.last(), Some(&b"plot|2|Loading".to_vec()));
        assert_eq!(mapped.activity, Some(b"Loading...".to_vec()));
        assert_eq!(mapped.bars.quest.position, Some(0));
        assert_eq!(mapped.bars.quest.maximum, Some(75));
        assert_eq!(mapped.prized_equipment, Some(0));
        assert_eq!(mapped.game_style, Some(3));
        assert_eq!(mapped.profile.motto, Some(b"Synthetic motto".to_vec()));
        assert_eq!(mapped.profile.guild, Some(b"Synthetic Guild".to_vec()));
        assert_eq!(mapped.private.passkey, Some(4_242));
        assert_eq!(mapped.private.passkey_text, Some(b"4242".to_vec()));
        assert_eq!(mapped.private.realm, Some(b"Synthetic Realm".to_vec()));
        assert_eq!(
            mapped.private.endpoint,
            Some(b"http://synthetic.invalid/?".to_vec())
        );
        assert_eq!(mapped.private.account, Some(b"synthetic-account".to_vec()));
        assert_eq!(
            mapped.private.password,
            Some(b"synthetic-password".to_vec())
        );
    }

    #[test]
    fn preserves_both_prologue_queues_without_using_visual_or_timer_hints() {
        let legacy = map_desktop_document(&mapped_document("task|2|Loading")).unwrap();
        let modern = map_desktop_document(&mapped_document("plot|2|Loading")).unwrap();
        assert_eq!(legacy.queue.last(), Some(&b"task|2|Loading".to_vec()));
        assert_eq!(modern.queue.last(), Some(&b"plot|2|Loading".to_vec()));
        assert_eq!(legacy.bars.plot.position, Some(0));
        assert_eq!(legacy.bars.quest.position, Some(0));
        assert_eq!(legacy.bars.task.position, Some(2_000));
        assert_eq!(legacy.bars.task.maximum, Some(2_000));
    }

    #[test]
    fn validates_desktop_state_without_browser_normalization() {
        let mapped = map_desktop_document(&mapped_document("plot|2|Loading")).unwrap();
        let validated = validate_desktop_save(&mapped).unwrap();

        assert_eq!(validated.spells[0].caption, "Gyp");
        assert_eq!(validated.spells[1].caption, "Shoelaces");
        assert_eq!(validated.current_task, "load");
        assert_eq!(validated.queue.last().unwrap().kind, DesktopQueueKind::Plot);
        assert_eq!(validated.game_style, 3);
        assert_eq!(validated.prized_equipment, 0);
        assert_eq!(
            validated.quest,
            DesktopQuestMarker::LegacyPlaceholder { index: 1 }
        );
        assert!(!validated.adaptations.legacy_prologue_62);
        assert!(validated.adaptations.legacy_quest_placeholder);
        assert!(!validated.adaptations.spelling_patch_applied);
    }

    #[test]
    fn validates_supported_state_from_in_memory_binary_input() {
        let bytes = encoded_document(&mapped_document("plot|2|Loading"));
        let document = parse_components(&bytes).unwrap();
        let mapped = map_desktop_document(&document).unwrap();
        let validated = validate_desktop_save(&mapped).unwrap();

        assert_eq!(validated.traits[0].subitems, ["Synthetic"]);
        assert_eq!(validated.queue.len(), 2);
        assert_eq!(validated.bars.task.maximum, 2_000);
        assert_eq!(validated.private.account, "synthetic-account");
    }

    #[test]
    fn applies_only_evidenced_defaults_and_load_adaptations() {
        let mut mapped = map_desktop_document(&mapped_document("task|2|Loading")).unwrap();
        mapped.current_task = None;
        mapped.activity = Some(b"Loading....".to_vec());
        mapped.spells = vec![
            mapped_row(b"Innoculate", b"I"),
            mapped_row(b"Innoculate", b"II"),
            mapped_row(b"Tonsilectomy", b"III"),
        ];
        mapped.quests.clear();
        mapped.bars.plot.position = None;
        mapped.bars.quest.position = None;
        mapped.bars.task.position = None;
        mapped.profile.motto = None;
        mapped.profile.guild = None;

        let validated = validate_desktop_save(&mapped).unwrap();
        assert_eq!(validated.current_task, "");
        assert_eq!(validated.spells[0].caption, "Innoculate");
        assert_eq!(validated.spells[1].caption, "Inoculate");
        assert_eq!(validated.spells[2].caption, "Tonsillectomy");
        assert!(validated.quests.is_empty());
        assert_eq!(validated.bars.plot.position, 0);
        assert_eq!(validated.bars.quest.position, 0);
        assert_eq!(validated.bars.task.position, 0);
        assert_eq!(validated.profile.motto, "");
        assert_eq!(validated.profile.guild, "");
        assert!(validated.adaptations.legacy_prologue_62);
        assert!(validated.adaptations.spelling_patch_applied);
    }

    #[test]
    fn quest_placeholder_study_compares_callbacks_requests_and_preserves_provenance() {
        use crate::{
            compatibility::{DesktopAdaptation, DesktopRandomState},
            desktop_callback::DesktopCallbackCheckpoint,
            desktop_simulation::{
                DesktopReportTrigger, SourceDerivedDesktopHooks, resolve_legacy_quest_marker,
            },
        };

        let study: serde_json::Value = serde_json::from_str(include_str!(
            "../tests/fixtures/desktop-quest-placeholder-study.json"
        ))
        .unwrap();
        assert_eq!(
            study["source"]["commit"],
            crate::desktop_rules::SOURCE_COMMIT
        );
        assert_eq!(
            study["source"]["sha256"],
            crate::desktop_rules::MAIN_PAS_SHA256
        );
        assert_eq!(study["syntheticResults"]["questTransitions"]["seeds"], 64);
        assert_eq!(
            study["syntheticResults"]["questTransitions"]["transitionsPerSeed"],
            3
        );
        assert_eq!(
            study["syntheticResults"]["questTransitions"]["initialAndSubsequentQuestPairs"],
            true
        );
        assert_eq!(study["decision"]["changesEligibility"], false);
        assert_eq!(study["decision"]["authorizesRequests"], false);
        assert_eq!(study["decision"]["activeTimeEvidence"], false);

        let mut mapped = map_desktop_document(&mapped_document("plot|2|Loading")).unwrap();
        mapped.current_task = Some(b"kill|Rat|1|tail".to_vec());
        mapped.queue.clear();
        mapped.private.realm = Some(b"Pemptus".to_vec());
        mapped.private.endpoint = Some(b"http://progressquest.com/pemptus.php?".to_vec());
        mapped.private.account = None;
        mapped.private.password = None;
        let placeholder_save = validate_desktop_save(&mapped).unwrap();
        let index = 1;
        let mut canonical_mapped = mapped.clone();
        canonical_mapped.quest_marker = Some(
            crate::desktop_rules::bundled().tables.monsters[index]
                .as_bytes()
                .to_vec(),
        );
        let canonical_save = validate_desktop_save(&canonical_mapped).unwrap();
        let metadata = DesktopImportMetadata::from_validated(&placeholder_save.adaptations);
        assert_eq!(
            metadata.provenance.adaptations,
            [DesktopAdaptation::LegacyQuestPlaceholder]
        );
        assert!(
            DesktopImportMetadata::from_validated(&canonical_save.adaptations)
                .provenance
                .adaptations
                .is_empty()
        );
        let mut resolved = DesktopCanonicalState::from(&placeholder_save);
        resolve_legacy_quest_marker(&mut resolved).unwrap();
        assert_eq!(resolved, DesktopCanonicalState::from(&canonical_save));
        assert_eq!(
            DesktopImportMetadata::from_validated(&placeholder_save.adaptations),
            metadata
        );

        fn compare_unsigned(left: &DesktopCanonicalState, right: &DesktopCanonicalState) {
            let authentication = DesktopAccountAuthentication::new("", "");
            for (operation, motto) in [
                (DesktopReportOperation::Manual, "Synthetic motto"),
                (DesktopReportOperation::Level, "Synthetic motto"),
                (DesktopReportOperation::Act, "Synthetic motto"),
                (DesktopReportOperation::Motto, "Synthetic set"),
                (DesktopReportOperation::Motto, ""),
            ] {
                let left =
                    report(left, operation, "Pemptus", motto, 4_242, &authentication).unwrap();
                let right =
                    report(right, operation, "Pemptus", motto, 4_242, &authentication).unwrap();
                assert_eq!(left.query_before_validator, right.query_before_validator);
                assert_eq!(left.fields_after_validator, right.fields_after_validator);
                assert!(
                    !left
                        .fields_before_validator
                        .iter()
                        .any(|field| field.name == "p")
                );
                assert!(!left.authentication.credentials_present);
            }
            for guild in ["Synthetic Guild A", "Synthetic Guild B", ""] {
                let left = guild_request(left, "Pemptus", guild, 4_242, &authentication).unwrap();
                let right = guild_request(right, "Pemptus", guild, 4_242, &authentication).unwrap();
                assert_eq!(left.query_before_validator, right.query_before_validator);
                assert!(
                    !left
                        .fields_before_validator
                        .iter()
                        .any(|field| field.name == "p")
                );
                assert!(!left.authentication.credentials_present);
            }
        }

        compare_unsigned(
            &DesktopCanonicalState::from(&placeholder_save),
            &DesktopCanonicalState::from(&canonical_save),
        );
        let mut monster_quest = false;
        let mut non_monster_quest = false;
        for seed in 0..64 {
            for initial in [false, true] {
                let mut left = DesktopCallbackCheckpoint {
                    state: DesktopCanonicalState::from(&placeholder_save),
                    random: DesktopRandomState(seed),
                };
                let mut right = DesktopCallbackCheckpoint {
                    state: DesktopCanonicalState::from(&canonical_save),
                    random: DesktopRandomState(seed),
                };
                if initial {
                    left.state.quests.clear();
                    right.state.quests.clear();
                }
                for _ in 0..3 {
                    for checkpoint in [&mut left, &mut right] {
                        checkpoint.state.current_task = "kill|Rat|1|tail".to_owned();
                        checkpoint.state.queue.clear();
                        checkpoint.state.bars.task.position = checkpoint.state.bars.task.maximum;
                        checkpoint.state.bars.quest.position = checkpoint.state.bars.quest.maximum;
                    }
                    let mut left_hooks = SourceDerivedDesktopHooks::traced();
                    let mut right_hooks = SourceDerivedDesktopHooks::traced();
                    assert_eq!(
                        left.apply_progression_callback(100, &mut left_hooks)
                            .unwrap(),
                        right
                            .apply_progression_callback(100, &mut right_hooks)
                            .unwrap()
                    );
                    assert_eq!(left, right);
                    assert_eq!(left_hooks.reports(), right_hooks.reports());
                    compare_unsigned(&left.state, &right.state);
                    monster_quest |= matches!(left.state.quest, DesktopQuestMarker::Value { .. });
                    non_monster_quest |= matches!(left.state.quest, DesktopQuestMarker::None);
                }
            }
        }
        assert!(monster_quest && non_monster_quest);

        let mut left = DesktopCallbackCheckpoint {
            state: DesktopCanonicalState::from(&placeholder_save),
            random: DesktopRandomState(1),
        };
        left.state.bars.task.position = left.state.bars.task.maximum;
        left.state.bars.experience.position = left.state.bars.experience.maximum;
        left.state.queue = vec![DesktopQueueCommand {
            kind: DesktopQueueKind::Plot,
            duration_seconds: 2,
            caption: "Synthetic act transition".to_owned(),
        }];
        let mut right = left.clone();
        right.state.quest = DesktopCanonicalState::from(&canonical_save).quest;
        let mut left_hooks = SourceDerivedDesktopHooks::traced();
        let mut right_hooks = SourceDerivedDesktopHooks::traced();
        left.apply_progression_callback(100, &mut left_hooks)
            .unwrap();
        right
            .apply_progression_callback(100, &mut right_hooks)
            .unwrap();
        assert_eq!(left, right);
        assert_eq!(left_hooks.reports(), right_hooks.reports());
        assert_eq!(
            left_hooks
                .reports()
                .iter()
                .map(|snapshot| snapshot.trigger)
                .collect::<Vec<_>>(),
            [DesktopReportTrigger::Level, DesktopReportTrigger::Act]
        );
        for (left, right) in left_hooks.reports().iter().zip(right_hooks.reports()) {
            compare_unsigned(&left.state, &right.state);
        }
        crate::desktop_evidence::with_synthetic_desktop_evidence(vec![], || {
            assert!(
                crate::save::inspect_desktop(&placeholder_save)
                    .online_eligibility
                    .iter()
                    .filter(|operation| operation.operation
                        != crate::desktop_eligibility::DesktopOnlineOperation::Guild)
                    .all(|operation| !matches!(
                        operation.decision,
                        crate::desktop_eligibility::DesktopEligibilityDecision::Eligible
                    ))
            )
        });
    }

    #[test]
    fn quest_placeholder_study_boundary_imports_do_not_gain_eligibility() {
        use crate::compatibility::DesktopAdaptation;
        let mut base = map_desktop_document(&mapped_document("plot|2|Loading")).unwrap();
        base.private.realm = Some(b"Pemptus".to_vec());
        base.private.endpoint = Some(b"http://progressquest.com/pemptus.php?".to_vec());
        base.private.account = None;
        base.private.password = None;
        for index in [
            -1,
            crate::desktop_rules::bundled().tables.monsters.len() as i64,
        ] {
            let mut candidate = base.clone();
            candidate.quest_index = Some(index);
            assert!(validate_desktop_save(&candidate).is_err());
        }
        let mut unsupported_task = base.clone();
        unsupported_task.current_task = Some(b"fTask".to_vec());
        assert_eq!(
            validate_desktop_save(&unsupported_task),
            Err(DesktopValidationError::InvalidTask)
        );
        let mut spelling = base.clone();
        spelling.spells[1].caption = b"Innoculate".to_vec();
        let mut prologue = base.clone();
        prologue.current_task = None;
        prologue.activity = Some(b"Loading....".to_vec());
        prologue.queue = vec![b"task|2|Loading".to_vec()];
        for candidate in [&base, &spelling, &prologue] {
            let save = validate_desktop_save(candidate).unwrap();
            let metadata = DesktopImportMetadata::from_validated(&save.adaptations);
            assert!(
                metadata
                    .provenance
                    .adaptations
                    .contains(&DesktopAdaptation::LegacyQuestPlaceholder)
            );
            crate::desktop_evidence::with_synthetic_desktop_evidence(vec![], || {
                assert!(
                    crate::save::inspect_desktop(&save)
                        .online_eligibility
                        .iter()
                        .all(|operation| {
                            matches!(
                                operation.decision,
                                crate::desktop_eligibility::DesktopEligibilityDecision::Eligible
                            ) == (operation.operation
                                == crate::desktop_eligibility::DesktopOnlineOperation::Guild
                                && metadata.provenance.adaptations
                                    == [DesktopAdaptation::LegacyQuestPlaceholder])
                        })
                )
            });
        }
        assert!(
            validate_desktop_save(&spelling)
                .unwrap()
                .adaptations
                .spelling_patch_applied
        );
        assert!(
            validate_desktop_save(&prologue)
                .unwrap()
                .adaptations
                .legacy_prologue_62
        );
    }

    #[test]
    fn spelling_patch_converges_to_canonical_state_and_protocol_with_distinct_provenance() {
        let mut pre_correction = map_desktop_document(&mapped_document("plot|2|Loading")).unwrap();
        pre_correction.quest_marker = None;
        pre_correction.quest_index = None;
        pre_correction.spells = vec![
            mapped_row(b"Gyp", b"I"),
            mapped_row(b"Innoculate", b"II"),
            mapped_row(b"Tonsilectomy", b"III"),
        ];
        let mut already_canonical = pre_correction.clone();
        already_canonical.spells = vec![
            mapped_row(b"Gyp", b"I"),
            mapped_row(b"Inoculate", b"II"),
            mapped_row(b"Tonsillectomy", b"III"),
        ];

        let pre_correction = validate_desktop_save(&pre_correction).unwrap();
        let already_canonical = validate_desktop_save(&already_canonical).unwrap();
        let pre_state = DesktopCanonicalState::from(&pre_correction);
        let canonical_state = DesktopCanonicalState::from(&already_canonical);
        assert_eq!(pre_state, canonical_state);

        let pre_metadata = DesktopImportMetadata::from_validated(&pre_correction.adaptations);
        let canonical_metadata =
            DesktopImportMetadata::from_validated(&already_canonical.adaptations);
        assert_eq!(
            pre_metadata.provenance.adaptations,
            [crate::compatibility::DesktopAdaptation::LoadSpellingPatch]
        );
        assert!(canonical_metadata.provenance.adaptations.is_empty());

        let authentication =
            DesktopAccountAuthentication::new("synthetic-account", "synthetic-password");
        for operation in [
            DesktopReportOperation::Manual,
            DesktopReportOperation::Level,
            DesktopReportOperation::Act,
            DesktopReportOperation::Motto,
        ] {
            assert_eq!(
                report(
                    &pre_state,
                    operation,
                    "Synthetic Realm",
                    "Synthetic motto",
                    4_242,
                    &authentication,
                )
                .unwrap(),
                report(
                    &canonical_state,
                    operation,
                    "Synthetic Realm",
                    "Synthetic motto",
                    4_242,
                    &authentication,
                )
                .unwrap()
            );
        }
        assert_eq!(
            guild_request(
                &pre_state,
                "Synthetic Realm",
                "Synthetic Guild",
                4_242,
                &authentication,
            )
            .unwrap(),
            guild_request(
                &canonical_state,
                "Synthetic Realm",
                "Synthetic Guild",
                4_242,
                &authentication,
            )
            .unwrap()
        );
    }

    #[test]
    fn rejects_unsafe_text_numbers_ranks_and_indexes() {
        let base = map_desktop_document(&mapped_document("plot|2|Loading")).unwrap();

        let mut non_ascii = base.clone();
        non_ascii.inventory[0].caption = vec![0xff];
        assert_eq!(
            validate_desktop_save(&non_ascii).unwrap_err(),
            DesktopValidationError::UnsupportedEncoding("Inventory")
        );

        let mut numeric = base.clone();
        numeric.stats[0].subitems[0] = b"-1".to_vec();
        assert_eq!(
            validate_desktop_save(&numeric).unwrap_err(),
            DesktopValidationError::InvalidNumber("Stats")
        );

        let mut rank = base.clone();
        rank.spells[0].subitems[0] = b"IIII".to_vec();
        assert_eq!(
            validate_desktop_save(&rank).unwrap_err(),
            DesktopValidationError::InvalidRank
        );

        let mut prized = base.clone();
        prized.prized_equipment = Some(prized.equipment.len() as i64);
        assert_eq!(
            validate_desktop_save(&prized).unwrap_err(),
            DesktopValidationError::InvalidIndex("Equips.Tag")
        );

        let mut quest = base.clone();
        quest.quest_index = Some(crate::desktop_rules::bundled().tables.monsters.len() as i64);
        assert_eq!(
            validate_desktop_save(&quest).unwrap_err(),
            DesktopValidationError::InvalidIndex("fQuest.Tag")
        );

        let mut style = base.clone();
        style.game_style = Some(2);
        assert_eq!(
            validate_desktop_save(&style).unwrap_err(),
            DesktopValidationError::InvalidNumber("GameStyle.Tag")
        );
    }

    #[test]
    fn rejects_unsupported_commands_tasks_progress_and_passkeys() {
        let base = map_desktop_document(&mapped_document("plot|2|Loading")).unwrap();

        let mut command = base.clone();
        command.queue[0] = b"wait|2|Nope".to_vec();
        assert_eq!(
            validate_desktop_save(&command).unwrap_err(),
            DesktopValidationError::InvalidCommand
        );

        let mut task = base.clone();
        task.current_task = Some(b"browser-task".to_vec());
        assert_eq!(
            validate_desktop_save(&task).unwrap_err(),
            DesktopValidationError::InvalidTask
        );

        let mut progress = base.clone();
        progress.bars.task.position = Some(2_001);
        assert_eq!(
            validate_desktop_save(&progress).unwrap_err(),
            DesktopValidationError::InvalidNumber("TaskBar")
        );

        let mut passkey = base.clone();
        passkey.private.passkey_text = Some(b"4243".to_vec());
        assert_eq!(
            validate_desktop_save(&passkey).unwrap_err(),
            DesktopValidationError::PasskeyMismatch
        );
    }

    #[test]
    fn rejects_missing_wrong_class_and_wrong_property_type_components() {
        let mut missing = mapped_document("plot|2|Loading");
        missing
            .components
            .retain(|component| component.name != "Traits");
        assert_eq!(
            map_desktop_document(&missing).unwrap_err(),
            DesktopMappingError::MissingComponent("Traits")
        );

        let mut wrong_class = mapped_document("plot|2|Loading");
        wrong_class
            .components
            .iter_mut()
            .find(|component| component.name == "Traits")
            .unwrap()
            .class_name = "TLabel".to_owned();
        assert_eq!(
            map_desktop_document(&wrong_class).unwrap_err(),
            DesktopMappingError::UnsupportedClass {
                component: "Traits",
                expected: "TListView",
            }
        );

        let mut wrong_property = mapped_document("plot|2|Loading");
        wrong_property
            .components
            .iter_mut()
            .find(|component| component.name == "TaskBar")
            .unwrap()
            .properties
            .iter_mut()
            .find(|property| property.name == "Position")
            .unwrap()
            .value = text("not numeric");
        assert_eq!(
            map_desktop_document(&wrong_property).unwrap_err(),
            DesktopMappingError::PropertyType {
                component: "TaskBar",
                property: "Position",
            }
        );
    }
}
