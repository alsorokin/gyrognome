use std::io::Write;

use flate2::{Compression, write::ZlibEncoder};
use thiserror::Error;

use crate::{
    compatibility::DesktopCanonicalState,
    desktop_save::{
        DesktopQuestMarker, DesktopQueueKind, DesktopValidatedPrivateMetadata, DesktopValidatedRow,
    },
};

/// Components of the desktop 6.4.4 main form in `Components[]` order. The
/// client reads one component per slot, so every slot must be present.
const COMPONENTS: &[(&str, &str)] = &[
    ("TPanel", "Panel1"),
    ("TLabel", "Label1"),
    ("TLabel", "Label6"),
    ("TLabel", "Label4"),
    ("TListView", "Traits"),
    ("TListView", "Stats"),
    ("TProgressBar", "ExpBar"),
    ("TListView", "Spells"),
    ("TPanel", "Cheats"),
    ("TButton", "CashIn"),
    ("TButton", "Button1"),
    ("TButton", "FinishQuest"),
    ("TButton", "Button3"),
    ("TButton", "CheatPlot"),
    ("TPanel", "Panel3"),
    ("TLabel", "Label3"),
    ("TLabel", "Label2"),
    ("TProgressBar", "QuestBar"),
    ("TListView", "Plots"),
    ("TProgressBar", "PlotBar"),
    ("TListView", "Quests"),
    ("TPanel", "Panel2"),
    ("TLabel", "InventoryLabelAlsoGameStyle"),
    ("TLabel", "Label7"),
    ("TLabel", "Label8"),
    ("TListView", "Inventory"),
    ("TProgressBar", "EncumBar"),
    ("TListView", "Equips"),
    ("TPanel", "vars"),
    ("TLabel", "fTask"),
    ("TLabel", "fQuest"),
    ("TListBox", "fQueue"),
    ("TPanel", "Panel4"),
    ("TStatusBar", "Kill"),
    ("TProgressBar", "TaskBar"),
    ("TTimer", "Timer1"),
    ("TImageList", "ImageList1"),
];

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DesktopExportError {
    #[error("{0} contains text that a desktop save cannot hold")]
    Text(&'static str),
    #[error("{0} is too large for a desktop save")]
    TooLarge(&'static str),
    #[error("desktop compression failed")]
    Compression,
}

enum Value {
    Int(i64),
    Str(String),
    Bin(Vec<u8>),
    Strings(Vec<String>),
}

/// Encodes canonical desktop state and credentials as a `.pq` save.
pub fn encode_desktop_save(
    state: &DesktopCanonicalState,
    private: &DesktopValidatedPrivateMetadata,
) -> Result<Vec<u8>, DesktopExportError> {
    let mut stream = Vec::new();
    for (class, name) in COMPONENTS {
        let properties = properties(name, state, private)?;
        stream.extend_from_slice(b"TPF0");
        push_short(&mut stream, class, "component")?;
        push_short(&mut stream, name, "component")?;
        for (property, value) in properties {
            push_short(&mut stream, property, "property")?;
            push_value(&mut stream, &value, property)?;
        }
        stream.extend_from_slice(&[0, 0]);
    }
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(&stream)
        .map_err(|_| DesktopExportError::Compression)?;
    encoder
        .finish()
        .map_err(|_| DesktopExportError::Compression)
}

fn properties(
    name: &str,
    state: &DesktopCanonicalState,
    private: &DesktopValidatedPrivateMetadata,
) -> Result<Vec<(&'static str, Value)>, DesktopExportError> {
    let text = |value: &str| Value::Str(value.to_owned());
    let bar = |bar: &crate::desktop_save::DesktopValidatedBar| -> Result<_, DesktopExportError> {
        let int = |value: u64| {
            i64::try_from(value).map_err(|_| DesktopExportError::TooLarge("progress bar"))
        };
        let mut properties = vec![
            ("Min", Value::Int(0)),
            ("Max", Value::Int(int(bar.maximum)?)),
            ("Position", Value::Int(int(bar.position)?)),
        ];
        let remaining = bar.maximum.saturating_sub(bar.position);
        let hint = match name {
            "ExpBar" => Some(format!("{remaining} XP needed for next level")),
            "PlotBar" => Some(format!("{} remaining", rough_time(remaining))),
            "QuestBar" if bar.maximum > 0 => {
                Some(format!("{}% complete", 100 * bar.position / bar.maximum))
            }
            "EncumBar" => Some(format!("{}/{} cubits", bar.position, bar.maximum)),
            _ => None,
        };
        if let Some(hint) = hint {
            properties.push(("Hint", Value::Str(hint)));
        }
        Ok(properties)
    };
    Ok(match name {
        "Label1" => vec![("Hint", text(&state.profile.guild))],
        "Traits" => vec![
            ("Tag", Value::Int(i64::from(private.passkey))),
            (
                "Hint",
                Value::Str(if private.passkey == 0 {
                    String::new()
                } else {
                    private.passkey.to_string()
                }),
            ),
            ("Items.Data", Value::Bin(encode_rows(&state.traits)?)),
        ],
        "Stats" => vec![
            ("Hint", text(&state.profile.motto)),
            ("Items.Data", Value::Bin(encode_rows(&state.stats)?)),
        ],
        "Equips" => vec![
            ("Tag", Value::Int(to_i64(state.prized_equipment)?)),
            ("Hint", text(&private.endpoint)),
            ("Items.Data", Value::Bin(encode_rows(&state.equipment)?)),
        ],
        "Inventory" => vec![
            ("Hint", text(&private.account)),
            ("Items.Data", Value::Bin(encode_rows(&state.inventory)?)),
        ],
        "Spells" => vec![
            ("Hint", text(&private.realm)),
            ("Items.Data", Value::Bin(encode_rows(&state.spells)?)),
        ],
        "Plots" => vec![
            ("Hint", text(&private.password)),
            ("Items.Data", Value::Bin(encode_rows(&state.plots)?)),
        ],
        "Quests" => vec![("Items.Data", Value::Bin(encode_rows(&state.quests)?))],
        "ExpBar" => bar(&state.bars.experience)?,
        "EncumBar" => bar(&state.bars.encumbrance)?,
        "PlotBar" => bar(&state.bars.plot)?,
        "QuestBar" => bar(&state.bars.quest)?,
        "TaskBar" => bar(&state.bars.task)?,
        "InventoryLabelAlsoGameStyle" => vec![("Tag", Value::Int(i64::from(state.game_style)))],
        "fTask" => vec![("Caption", text(&state.current_task))],
        "fQuest" => match &state.quest {
            DesktopQuestMarker::None => vec![("Caption", text(""))],
            DesktopQuestMarker::LegacyPlaceholder { index } => vec![
                ("Caption", text("fQuest")),
                ("Tag", Value::Int(to_i64(*index)?)),
            ],
            DesktopQuestMarker::Value { marker, index } => vec![
                ("Caption", text(marker)),
                ("Tag", Value::Int(to_i64(*index)?)),
            ],
        },
        "fQueue" => vec![(
            "Items.Strings",
            Value::Strings(
                state
                    .queue
                    .iter()
                    .map(|command| {
                        let kind = match command.kind {
                            DesktopQueueKind::Task => "task",
                            DesktopQueueKind::Plot => "plot",
                        };
                        format!("{kind}|{}|{}", command.duration_seconds, command.caption)
                    })
                    .collect(),
            ),
        )],
        "Kill" => vec![("SimpleText", text(&state.activity))],
        _ => Vec::new(),
    })
}

fn to_i64(value: usize) -> Result<i64, DesktopExportError> {
    i64::try_from(value).map_err(|_| DesktopExportError::TooLarge("index"))
}

fn rough_time(seconds: u64) -> String {
    if seconds < 120 {
        format!("{seconds} seconds")
    } else if seconds < 60 * 120 {
        format!("{} minutes", seconds / 60)
    } else if seconds < 3600 * 48 {
        format!("{} hours", seconds / 3600)
    } else {
        format!("{} days", seconds / 86400)
    }
}

fn encode_rows(rows: &[DesktopValidatedRow]) -> Result<Vec<u8>, DesktopExportError> {
    let mut body = Vec::new();
    body.extend_from_slice(&u32_bytes(rows.len(), "list")?);
    let mut subitems = 0_usize;
    for row in rows {
        // pq.exe reads the subitem count from word 3. Rows created by the
        // simulation carry it in word 4 with a different prefix.
        let mut header = if row.header[3] == 0 && row.header[4] != 0 {
            [0, -1, -1, 0, 0]
        } else {
            row.header
        };
        header[3] = i32::try_from(row.subitems.len())
            .map_err(|_| DesktopExportError::TooLarge("list row"))?;
        header[4] = 0;
        for value in header {
            body.extend_from_slice(&value.to_le_bytes());
        }
        push_short(&mut body, &row.caption, "list text")?;
        for item in &row.subitems {
            push_short(&mut body, item, "list text")?;
        }
        subitems += row.subitems.len();
    }
    body.extend(std::iter::repeat_n(0xff, subitems * 2));
    let mut bytes = u32_bytes(body.len() + 4, "list")?.to_vec();
    bytes.extend(body);
    Ok(bytes)
}

fn u32_bytes(value: usize, what: &'static str) -> Result<[u8; 4], DesktopExportError> {
    Ok(u32::try_from(value)
        .map_err(|_| DesktopExportError::TooLarge(what))?
        .to_le_bytes())
}

fn push_short(
    out: &mut Vec<u8>,
    value: &str,
    what: &'static str,
) -> Result<(), DesktopExportError> {
    if !value.is_ascii() {
        return Err(DesktopExportError::Text(what));
    }
    let length = u8::try_from(value.len()).map_err(|_| DesktopExportError::TooLarge(what))?;
    out.push(length);
    out.extend_from_slice(value.as_bytes());
    Ok(())
}

fn push_string_value(
    out: &mut Vec<u8>,
    value: &str,
    what: &'static str,
) -> Result<(), DesktopExportError> {
    if !value.is_ascii() {
        return Err(DesktopExportError::Text(what));
    }
    if value.len() <= 255 {
        out.push(6);
        out.push(value.len() as u8);
    } else {
        out.push(12);
        out.extend_from_slice(&u32_bytes(value.len(), what)?);
    }
    out.extend_from_slice(value.as_bytes());
    Ok(())
}

fn push_value(
    out: &mut Vec<u8>,
    value: &Value,
    what: &'static str,
) -> Result<(), DesktopExportError> {
    match value {
        Value::Int(value) => {
            if let Ok(value) = i8::try_from(*value) {
                out.push(2);
                out.push(value as u8);
            } else if let Ok(value) = i16::try_from(*value) {
                out.push(3);
                out.extend_from_slice(&value.to_le_bytes());
            } else if let Ok(value) = i32::try_from(*value) {
                out.push(4);
                out.extend_from_slice(&value.to_le_bytes());
            } else {
                out.push(19);
                out.extend_from_slice(&value.to_le_bytes());
            }
        }
        Value::Str(value) => push_string_value(out, value, what)?,
        Value::Bin(bytes) => {
            out.push(10);
            out.extend_from_slice(&u32_bytes(bytes.len(), what)?);
            out.extend_from_slice(bytes);
        }
        Value::Strings(values) => {
            out.push(1);
            for value in values {
                push_string_value(out, value, what)?;
            }
            out.push(0);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        compatibility::DesktopCanonicalState,
        desktop_save::{
            DesktopAdaptations, DesktopQueueCommand, DesktopValidatedBar, DesktopValidatedBars,
            DesktopValidatedProfile, DesktopValidatedSave,
        },
        save::{self, ImportedSave},
    };

    fn rows(values: &[(&str, &str)]) -> Vec<DesktopValidatedRow> {
        values
            .iter()
            .map(|(caption, value)| DesktopValidatedRow {
                header: [0, -1, -1, 1, 0],
                caption: (*caption).to_owned(),
                subitems: vec![(*value).to_owned()],
            })
            .collect()
    }

    fn bar(position: u64, maximum: u64) -> DesktopValidatedBar {
        DesktopValidatedBar { position, maximum }
    }

    fn fixture() -> DesktopValidatedSave {
        DesktopValidatedSave {
            traits: rows(&[
                ("Name", "Export Hero"),
                ("Race", "Gyrognome"),
                ("Class", "Robot Monk"),
                ("Level", "7"),
            ]),
            stats: rows(&[
                ("STR", "12"),
                ("CON", "11"),
                ("DEX", "10"),
                ("INT", "9"),
                ("WIS", "8"),
                ("CHA", "7"),
                ("HP Max", "20"),
                ("MP Max", "15"),
            ]),
            equipment: rows(&[
                ("Weapon", "Stick"),
                ("Shield", ""),
                ("Helm", "Cap"),
                ("Hauberk", ""),
                ("Brassairts", ""),
                ("Vambraces", ""),
                ("Gauntlets", ""),
                ("Gambeson", ""),
                ("Cuisses", ""),
                ("Greaves", ""),
                ("Sollerets", ""),
            ]),
            inventory: rows(&[("Gold", "300"), ("Rat tail", "2")]),
            spells: rows(&[("Gyp", "II"), ("Shoelaces", "I")]),
            plots: vec![
                DesktopValidatedRow {
                    header: [0, 1, -1, 0, 0],
                    caption: "Prologue".to_owned(),
                    subitems: Vec::new(),
                },
                DesktopValidatedRow {
                    header: [0, 0, -1, 0, 0],
                    caption: "Act I".to_owned(),
                    subitems: Vec::new(),
                },
            ],
            quests: vec![DesktopValidatedRow {
                header: [0, 0, -1, 0, 0],
                caption: "Exterminate the Rats".to_owned(),
                subitems: Vec::new(),
            }],
            current_task: "kill|Rat|1|tail".to_owned(),
            quest: DesktopQuestMarker::Value {
                marker: "3|Rat".to_owned(),
                index: 4,
            },
            queue: vec![
                DesktopQueueCommand {
                    kind: DesktopQueueKind::Task,
                    duration_seconds: 2,
                    caption: "Continue".to_owned(),
                },
                DesktopQueueCommand {
                    kind: DesktopQueueKind::Plot,
                    duration_seconds: 10,
                    caption: "Act II".to_owned(),
                },
            ],
            activity: "Executing a Rat...".to_owned(),
            bars: DesktopValidatedBars {
                experience: bar(24, 1_269),
                encumbrance: bar(4, 23),
                plot: bar(30, 21_600),
                quest: bar(18, 149),
                task: bar(4_232, 70_000),
            },
            prized_equipment: 2,
            game_style: 3,
            profile: DesktopValidatedProfile {
                motto: "Synthetic motto".to_owned(),
                guild: "Synthetic Guild".to_owned(),
            },
            private: DesktopValidatedPrivateMetadata {
                passkey: 4_242,
                realm: "Synthetic Realm".to_owned(),
                endpoint: "https://synthetic.invalid/".to_owned(),
                account: "synthetic-account".to_owned(),
                password: "synthetic-password".to_owned(),
            },
            adaptations: DesktopAdaptations {
                legacy_prologue_62: false,
                legacy_quest_placeholder: false,
                spelling_patch_applied: false,
            },
        }
    }

    fn round_trip(save: &DesktopValidatedSave) -> DesktopValidatedSave {
        let bytes = encode_desktop_save(&DesktopCanonicalState::from(save), &save.private).unwrap();
        match save::import_supported_bytes(&bytes).unwrap() {
            ImportedSave::Desktop(imported) => imported,
            ImportedSave::Browser(_) => panic!("expected a desktop save"),
        }
    }

    #[test]
    fn round_trips_state_and_credentials() {
        let save = fixture();
        let imported = round_trip(&save);
        assert_eq!(
            DesktopCanonicalState::from(&imported),
            DesktopCanonicalState::from(&save)
        );
        assert_eq!(imported.private, save.private);
    }

    #[test]
    fn round_trips_empty_optional_state() {
        let mut save = fixture();
        save.spells.clear();
        save.plots.clear();
        save.quests.clear();
        save.queue.clear();
        save.quest = DesktopQuestMarker::None;
        save.current_task.clear();
        save.profile.motto.clear();
        save.profile.guild.clear();
        save.private = DesktopValidatedPrivateMetadata {
            passkey: 0,
            realm: String::new(),
            endpoint: String::new(),
            account: String::new(),
            password: String::new(),
        };
        let imported = round_trip(&save);
        assert_eq!(
            DesktopCanonicalState::from(&imported),
            DesktopCanonicalState::from(&save)
        );
        assert_eq!(imported.private, save.private);
    }

    #[test]
    fn round_trips_legacy_quest_placeholder_and_alternate_header_layout() {
        let mut save = fixture();
        save.quest = DesktopQuestMarker::LegacyPlaceholder { index: 5 };
        save.plots = vec![DesktopValidatedRow {
            header: [0, -1, -1, 0, 1],
            caption: "Act I".to_owned(),
            subitems: vec!["0".to_owned()],
        }];
        let imported = round_trip(&save);
        assert_eq!(imported.quest, save.quest);
        assert_eq!(imported.plots[0].subitems, save.plots[0].subitems);
    }

    #[test]
    fn rejects_non_ascii_text() {
        let mut save = fixture();
        save.profile.motto = "caf\u{e9}".to_owned();
        assert!(matches!(
            encode_desktop_save(&DesktopCanonicalState::from(&save), &save.private),
            Err(DesktopExportError::Text(_))
        ));
    }
}
