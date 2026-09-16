//! Alternate-screen terminal flow for offline character creation.

use std::{
    io,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use thiserror::Error;

use crate::{
    dashboard::TerminalSession,
    newguy::{self, BaseStats, NewGuyError, OsRandom, RandomSource, Selection},
    rng::{Alea, AleaState},
    ruleset::{self, Ruleset},
    state::Character,
};

#[derive(Debug, Error)]
pub enum WizardError {
    #[error(transparent)]
    Generation(#[from] NewGuyError),
    #[error(transparent)]
    Dashboard(#[from] crate::dashboard::DashboardError),
    #[error("terminal operation failed: {0}")]
    Terminal(#[from] io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    Continue,
    Confirm,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Name,
    Race,
    Class,
    Stats,
}

impl Focus {
    fn next(self) -> Self {
        match self {
            Self::Name => Self::Race,
            Self::Race => Self::Class,
            Self::Class => Self::Stats,
            Self::Stats => Self::Name,
        }
    }

    fn previous(self) -> Self {
        match self {
            Self::Name => Self::Stats,
            Self::Race => Self::Name,
            Self::Class => Self::Race,
            Self::Stats => Self::Class,
        }
    }
}

struct RollSnapshot {
    base_stats: BaseStats,
    seed: AleaState,
}

impl RandomSource for Alea {
    fn next_u32(&mut self) -> Result<u32, NewGuyError> {
        Ok(self.uint32() as u32)
    }
}

struct Wizard {
    selection: Selection,
    character: Character,
    base_stats: BaseStats,
    roll_seed: AleaState,
    history: Vec<RollSnapshot>,
    focus: Focus,
    message: Option<String>,
    random: Alea,
    ruleset: Ruleset,
}

impl Wizard {
    fn new(ruleset: Ruleset, mut source: impl RandomSource) -> Result<Self, WizardError> {
        let mut random = Alea::from_state(AleaState([
            source.next_u32()? as f64 / (u32::MAX as f64 + 1.0),
            source.next_u32()? as f64 / (u32::MAX as f64 + 1.0),
            source.next_u32()? as f64 / (u32::MAX as f64 + 1.0),
            1.0,
        ]));
        let selection = newguy::random_selection(&ruleset, &mut random)?;
        let roll_seed = random.state();
        let base_stats = newguy::roll_stats(&mut random)?;
        let character = newguy::generate_from_roll(&selection, &ruleset, base_stats, &mut random)?;
        Ok(Self {
            selection,
            character,
            base_stats,
            roll_seed,
            history: Vec::new(),
            focus: Focus::Name,
            message: None,
            random,
            ruleset,
        })
    }

    fn apply(&mut self, event: Event) -> Result<Action, WizardError> {
        let Event::Key(key) = event else {
            return Ok(Action::Continue);
        };
        if key.kind != KeyEventKind::Press {
            return Ok(Action::Continue);
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Ok(Action::Cancel);
        }
        match key.code {
            KeyCode::Esc => Ok(Action::Cancel),
            KeyCode::Enter => self.confirm(),
            KeyCode::Up => {
                self.focus = self.focus.previous();
                Ok(Action::Continue)
            }
            KeyCode::Down => {
                self.focus = self.focus.next();
                Ok(Action::Continue)
            }
            key => self.apply_focused(key),
        }
    }

    fn apply_focused(&mut self, key: KeyCode) -> Result<Action, WizardError> {
        match self.focus {
            Focus::Name => match key {
                KeyCode::Char('?') => {
                    self.selection.name = newguy::random_name(&self.ruleset, &mut self.random)?;
                    self.refresh()?;
                }
                KeyCode::Backspace => {
                    self.selection.name.pop();
                    self.message = None;
                    self.refresh()?;
                }
                KeyCode::Char(character)
                    if !character.is_control()
                        && self.selection.name.chars().count() < newguy::NAME_MAX_LENGTH =>
                {
                    self.selection.name.push(character);
                    self.message = None;
                    self.refresh()?;
                }
                _ => {}
            },
            Focus::Race => match key {
                KeyCode::Left => {
                    self.selection.race = previous_trait(self.ruleset.races, &self.selection.race);
                    self.refresh()?;
                }
                KeyCode::Right => {
                    self.selection.race = next_trait(self.ruleset.races, &self.selection.race);
                    self.refresh()?;
                }
                _ => {}
            },
            Focus::Class => match key {
                KeyCode::Left => {
                    self.selection.class =
                        previous_trait(self.ruleset.klasses, &self.selection.class);
                    self.refresh()?;
                }
                KeyCode::Right => {
                    self.selection.class = next_trait(self.ruleset.klasses, &self.selection.class);
                    self.refresh()?;
                }
                _ => {}
            },
            Focus::Stats => match key {
                KeyCode::Char('r') => {
                    self.history.push(RollSnapshot {
                        base_stats: self.base_stats,
                        seed: self.roll_seed,
                    });
                    self.roll_seed = self.random.state();
                    self.roll()?;
                }
                KeyCode::Char('u') => {
                    if let Some(previous) = self.history.pop() {
                        self.random = Alea::from_state(previous.seed);
                        self.roll_seed = previous.seed;
                        self.roll()?;
                        debug_assert_eq!(self.base_stats, previous.base_stats);
                    }
                }
                _ => {}
            },
        }
        Ok(Action::Continue)
    }

    fn refresh(&mut self) -> Result<(), WizardError> {
        newguy::apply_stats_unchecked(
            &mut self.character,
            &self.selection,
            &self.ruleset,
            self.base_stats,
        )?;
        Ok(())
    }

    fn roll(&mut self) -> Result<(), WizardError> {
        self.base_stats = newguy::roll_stats(&mut self.random)?;
        self.character = newguy::generate_from_roll(
            &self.selection,
            &self.ruleset,
            self.base_stats,
            &mut self.random,
        )?;
        Ok(())
    }

    fn confirm(&mut self) -> Result<Action, WizardError> {
        match newguy::validate_name(&self.selection.name) {
            Ok(()) => Ok(Action::Confirm),
            Err(error) => {
                self.message = Some(error.to_string());
                Ok(Action::Continue)
            }
        }
    }
}

pub fn run(interrupted: &AtomicBool) -> Result<Option<Character>, WizardError> {
    let mut wizard = Wizard::new(ruleset::BUNDLED, OsRandom::open()?)?;
    let mut session = TerminalSession::enter()?;
    loop {
        session.terminal.draw(|frame| render(frame, &wizard))?;
        if interrupted.load(Ordering::Relaxed) {
            return Ok(None);
        }
        if event::poll(Duration::from_millis(100))? {
            match wizard.apply(event::read()?)? {
                Action::Continue => {}
                Action::Confirm => return Ok(Some(wizard.character)),
                Action::Cancel => return Ok(None),
            }
        }
    }
}

fn render(frame: &mut ratatui::Frame<'_>, wizard: &Wizard) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(frame.area());
    frame.render_widget(
        Paragraph::new("New Guy").block(Block::default().borders(Borders::ALL).title("Gyrognome")),
        rows[0],
    );
    let stats = &wizard.character.stats;
    let total = wizard.base_stats.into_iter().map(u64::from).sum::<u64>();
    let marker = |focus| if wizard.focus == focus { ">" } else { " " };
    let stat_lines = vec![
        Line::from(format!(
            "{} Name: {}",
            marker(Focus::Name),
            wizard.selection.name
        )),
        Line::from(format!(
            "{} Race: {}",
            marker(Focus::Race),
            wizard.selection.race
        )),
        Line::from(format!(
            "{} Class: {}",
            marker(Focus::Class),
            wizard.selection.class
        )),
        Line::from(vec![Span::raw(format!(
            "{} Stats: STR {}  CON {}  DEX {}  INT {}  WIS {}  CHA {}",
            marker(Focus::Stats),
            stats.strength,
            stats.constitution,
            stats.dexterity,
            stats.intelligence,
            stats.wisdom,
            stats.charisma,
        ))]),
        Line::from(format!(
            "  HP Max {}  MP Max {}",
            stats.hit_points_max, stats.mana_points_max
        )),
        Line::from(vec![
            Span::raw("  Total: "),
            Span::styled(total.to_string(), Style::default().fg(total_color(total))),
        ]),
    ];
    frame.render_widget(
        Paragraph::new(stat_lines).block(Block::default().borders(Borders::ALL).title("Character")),
        rows[1],
    );
    frame.render_widget(
        Paragraph::new(match &wizard.message {
            Some(message) => format!("Error: {message} | Enter Sold! | Esc cancel"),
            None => format!(
                "{} | Enter Sold! | Esc cancel",
                focused_controls(wizard.focus, !wizard.history.is_empty())
            ),
        })
        .block(Block::default().borders(Borders::ALL).title("Controls")),
        rows[2],
    );
}

fn focused_controls(focus: Focus, can_unroll: bool) -> &'static str {
    match (focus, can_unroll) {
        (Focus::Name, _) => "Up/Down select row | type name | ? Random Name",
        (Focus::Race, _) => "Up/Down select row | Left/Right select race",
        (Focus::Class, _) => "Up/Down select row | Left/Right select class",
        (Focus::Stats, true) => "Up/Down select row | r Roll | u Unroll",
        (Focus::Stats, false) => "Up/Down select row | r Roll",
    }
}

fn total_color(total: u64) -> Color {
    match total {
        ..46 => Color::DarkGray,
        46..=54 => Color::Gray,
        55..=72 => Color::White,
        73..=80 => Color::Yellow,
        _ => Color::Red,
    }
}

fn next_trait(values: &[&str], current: &str) -> String {
    let index = values
        .iter()
        .position(|value| value.split('|').next() == Some(current))
        .unwrap_or(0);
    values[(index + 1) % values.len()]
        .split('|')
        .next()
        .unwrap_or_default()
        .to_owned()
}

fn previous_trait(values: &[&str], current: &str) -> String {
    let index = values
        .iter()
        .position(|value| value.split('|').next() == Some(current))
        .unwrap_or(0);
    values[(index + values.len() - 1) % values.len()]
        .split('|')
        .next()
        .unwrap_or_default()
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Numbers(u32);
    impl RandomSource for Numbers {
        fn next_u32(&mut self) -> Result<u32, NewGuyError> {
            let value = self.0;
            self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            Ok(value)
        }
    }

    fn key(code: KeyCode) -> Event {
        Event::Key(crossterm::event::KeyEvent::from(code))
    }

    #[test]
    fn focused_events_keep_action_key_characters_in_names() {
        let mut wizard = Wizard::new(ruleset::BUNDLED, Numbers(1)).unwrap();
        let original = wizard.selection.clone();
        wizard.apply(key(KeyCode::Char('r'))).unwrap();
        wizard.apply(key(KeyCode::Char('e'))).unwrap();
        assert!(wizard.selection.name.ends_with("re"));
        assert_eq!(wizard.selection.race, original.race);
        assert_eq!(wizard.selection.class, original.class);

        wizard.apply(key(KeyCode::Down)).unwrap();
        let race = wizard.selection.race.clone();
        wizard.apply(key(KeyCode::Right)).unwrap();
        assert_ne!(wizard.selection.race, race);

        wizard.apply(key(KeyCode::Down)).unwrap();
        let class = wizard.selection.class.clone();
        wizard.apply(key(KeyCode::Right)).unwrap();
        assert_ne!(wizard.selection.class, class);
    }

    #[test]
    fn rolls_and_unrolls_all_prior_stat_rolls() {
        let mut wizard = Wizard::new(ruleset::BUNDLED, Numbers(1)).unwrap();
        wizard.apply(key(KeyCode::Down)).unwrap();
        wizard.apply(key(KeyCode::Down)).unwrap();
        wizard.apply(key(KeyCode::Down)).unwrap();
        let initial = wizard.base_stats;
        wizard.apply(key(KeyCode::Char('r'))).unwrap();
        let first = wizard.base_stats;
        wizard.apply(key(KeyCode::Char('r'))).unwrap();
        let second = wizard.base_stats;
        assert_ne!(second, first);
        wizard.apply(key(KeyCode::Char('u'))).unwrap();
        assert_eq!(wizard.base_stats, first);
        wizard.apply(key(KeyCode::Char('r'))).unwrap();
        assert_eq!(wizard.base_stats, second);
        wizard.apply(key(KeyCode::Char('u'))).unwrap();
        assert_eq!(wizard.base_stats, first);
        wizard.apply(key(KeyCode::Char('u'))).unwrap();
        assert_eq!(wizard.base_stats, initial);
    }

    #[test]
    fn matches_desktop_total_quality_boundaries() {
        assert_eq!(total_color(45), Color::DarkGray);
        assert_eq!(total_color(46), Color::Gray);
        assert_eq!(total_color(54), Color::Gray);
        assert_eq!(total_color(55), Color::White);
        assert_eq!(total_color(72), Color::White);
        assert_eq!(total_color(73), Color::Yellow);
        assert_eq!(total_color(80), Color::Yellow);
        assert_eq!(total_color(81), Color::Red);
    }

    #[test]
    fn confirms_and_cancels() {
        let mut wizard = Wizard::new(ruleset::BUNDLED, Numbers(1)).unwrap();
        assert_eq!(wizard.apply(key(KeyCode::Enter)).unwrap(), Action::Confirm);
        assert_eq!(wizard.apply(key(KeyCode::Esc)).unwrap(), Action::Cancel);
    }

    #[test]
    fn blocks_sold_for_an_invalid_name_without_closing_the_wizard() {
        let mut wizard = Wizard::new(ruleset::BUNDLED, Numbers(1)).unwrap();
        wizard.selection.name = "   ".to_owned();
        wizard.refresh().unwrap();
        assert_eq!(wizard.apply(key(KeyCode::Enter)).unwrap(), Action::Continue);
        assert_eq!(
            wizard.message.as_deref(),
            Some("character name must contain at least one non-whitespace character")
        );
    }
}
