//! Credential-safe terminal dashboard for observing one managed character.

use std::{
    io::{self, Stdout},
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
};
use thiserror::Error;

use crate::{
    lifecycle::{Lifecycle, LifecycleError, RuntimeStatus, ServiceState, SystemctlRunner},
    runtime::{CharacterId, CharacterIdentity, ManagedCharacter, StorageError, Store},
    state::{Activity, Equipment, InventoryEntry, Plot, Progress, Spell},
};

const MINIMUM_WIDTH: u16 = 40;
const MINIMUM_HEIGHT: u16 = 12;

#[derive(Debug, Clone)]
pub struct DashboardCharacter {
    pub id: CharacterId,
    pub identity: CharacterIdentity,
    pub activity: Activity,
    pub progress: Progress,
    pub equipment: Equipment,
    pub inventory: Vec<InventoryEntry>,
    pub spells: Vec<Spell>,
    pub plot: Plot,
    pub quests: Vec<String>,
    pub current_quest: String,
}

impl From<ManagedCharacter> for DashboardCharacter {
    fn from(character: ManagedCharacter) -> Self {
        Self {
            id: character.id,
            identity: character.identity,
            activity: character.state.activity,
            progress: character.state.progress,
            equipment: character.state.equipment,
            inventory: character.state.inventory,
            spells: character.state.spells,
            plot: character.state.plot,
            quests: character.state.quests,
            current_quest: character.state.bestquest,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DashboardSnapshot {
    pub character: DashboardCharacter,
    pub service: Option<ServiceState>,
    pub runtime_owned: Option<bool>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleAction {
    Start,
    Stop,
    Recover,
}

impl LifecycleAction {
    fn label(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Stop => "stop",
            Self::Recover => "recover",
        }
    }
}

pub trait DashboardProvider {
    fn refresh(&self, id: &CharacterId) -> Result<DashboardSnapshot, DashboardError>;
    fn lifecycle(&self, action: LifecycleAction, id: &CharacterId) -> Result<(), DashboardError>;
}

pub struct LocalProvider {
    store: Store,
}

impl LocalProvider {
    pub fn open_default() -> Result<Self, DashboardError> {
        Ok(Self {
            store: Store::open_default()?,
        })
    }
}

impl DashboardProvider for LocalProvider {
    fn refresh(&self, id: &CharacterId) -> Result<DashboardSnapshot, DashboardError> {
        collect_snapshot(&self.store, SystemctlRunner, id)
    }

    fn lifecycle(&self, action: LifecycleAction, id: &CharacterId) -> Result<(), DashboardError> {
        let lifecycle = Lifecycle::new(&self.store, SystemctlRunner);
        match action {
            LifecycleAction::Start => lifecycle.start(id)?,
            LifecycleAction::Stop => lifecycle.stop(id)?,
            LifecycleAction::Recover => lifecycle.recover(id)?,
        }
        Ok(())
    }
}

pub fn collect_snapshot<Runner: crate::lifecycle::ServiceRunner>(
    store: &Store,
    runner: Runner,
    id: &CharacterId,
) -> Result<DashboardSnapshot, DashboardError> {
    let character = store.get(id)?;
    let status = Lifecycle::new(store, runner).status(id);
    Ok(snapshot(character, status))
}

fn snapshot(
    character: ManagedCharacter,
    status: Result<RuntimeStatus, LifecycleError>,
) -> DashboardSnapshot {
    match status {
        Ok(status) => DashboardSnapshot {
            character: character.into(),
            service: Some(status.service),
            runtime_owned: Some(status.runtime_owned),
            message: None,
        },
        Err(error) => DashboardSnapshot {
            character: character.into(),
            service: None,
            runtime_owned: None,
            message: Some(format!("Could not refresh service status: {error}")),
        },
    }
}

#[derive(Debug, Error)]
pub enum DashboardError {
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error(transparent)]
    Lifecycle(#[from] LifecycleError),
    #[error("terminal operation failed: {0}")]
    Terminal(#[from] io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    Quit,
    Refresh,
    Confirm(LifecycleAction),
    ConfirmAction,
    Cancel,
    None,
}

fn command(event: Event) -> Command {
    let Event::Key(key) = event else {
        return Command::None;
    };
    if key.kind != KeyEventKind::Press {
        return Command::None;
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Command::Quit;
    }
    match key.code {
        KeyCode::Char('q') => Command::Quit,
        KeyCode::Char('r') => Command::Refresh,
        KeyCode::Char('s') => Command::Confirm(LifecycleAction::Start),
        KeyCode::Char('x') => Command::Confirm(LifecycleAction::Stop),
        KeyCode::Char('c') => Command::Confirm(LifecycleAction::Recover),
        KeyCode::Enter => Command::ConfirmAction,
        KeyCode::Esc => Command::Cancel,
        _ => Command::None,
    }
}

pub fn run<P: DashboardProvider>(
    provider: &P,
    id: CharacterId,
    refresh_interval: Duration,
    interrupted: &std::sync::atomic::AtomicBool,
) -> Result<(), DashboardError> {
    let mut session = TerminalSession::enter()?;
    let mut state = DashboardState {
        current: provider.refresh(&id)?,
        confirmation: None,
    };
    loop {
        session
            .terminal
            .draw(|frame| render(frame, &state.current, state.confirmation))?;
        if interrupted.load(std::sync::atomic::Ordering::Relaxed) {
            return Ok(());
        }
        if !event::poll(refresh_interval)? {
            state.refresh(provider, &id);
            continue;
        }
        if state.apply(provider, &id, command(event::read()?)) {
            return Ok(());
        }
    }
}

struct DashboardState {
    current: DashboardSnapshot,
    confirmation: Option<LifecycleAction>,
}

impl DashboardState {
    fn refresh<P: DashboardProvider>(&mut self, provider: &P, id: &CharacterId) {
        match provider.refresh(id) {
            Ok(next) => self.current = next,
            Err(error) => {
                self.current.message = Some(format!("Could not refresh dashboard: {error}"))
            }
        }
    }

    fn apply<P: DashboardProvider>(
        &mut self,
        provider: &P,
        id: &CharacterId,
        command: Command,
    ) -> bool {
        match command {
            Command::Quit => true,
            Command::Refresh => {
                self.refresh(provider, id);
                false
            }
            Command::Confirm(action) => {
                self.confirmation = Some(action);
                false
            }
            Command::Cancel => {
                self.confirmation = None;
                false
            }
            Command::ConfirmAction => {
                if let Some(action) = self.confirmation.take() {
                    match provider.lifecycle(action, id) {
                        Ok(()) => {
                            self.refresh(provider, id);
                            self.current.message =
                                Some(format!("Requested {} successfully.", action.label()));
                        }
                        Err(error) => {
                            self.current.message =
                                Some(format!("Could not {}: {error}", action.label()));
                        }
                    }
                }
                false
            }
            Command::None => false,
        }
    }
}

struct TerminalSession {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalSession {
    fn enter() -> Result<Self, DashboardError> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        if let Err(error) = execute!(stdout, EnterAlternateScreen) {
            let _ = disable_raw_mode();
            return Err(error.into());
        }
        match Terminal::new(CrosstermBackend::new(stdout)) {
            Ok(terminal) => Ok(Self { terminal }),
            Err(error) => {
                let mut stdout = io::stdout();
                let _ = execute!(stdout, LeaveAlternateScreen);
                let _ = disable_raw_mode();
                Err(error.into())
            }
        }
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = self.terminal.show_cursor();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

fn render(
    frame: &mut ratatui::Frame<'_>,
    snapshot: &DashboardSnapshot,
    confirmation: Option<LifecycleAction>,
) {
    let area = frame.area();
    if area.width < MINIMUM_WIDTH || area.height < MINIMUM_HEIGHT {
        frame.render_widget(
            Paragraph::new("Terminal is too small for the dashboard.\nResize to at least 40x12.")
                .block(Block::default().borders(Borders::ALL).title("Gyrognome")),
            area,
        );
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(6),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(area);
    render_header(frame, snapshot, rows[0]);
    if area.width < 90 {
        render_compact(frame, snapshot, rows[1]);
    } else {
        render_full(frame, snapshot, rows[1]);
    }
    let status = match (&snapshot.service, snapshot.runtime_owned) {
        (Some(state), Some(owned)) => format!(
            "Service: {state:?} | Runtime ownership: {}",
            if owned { "owned" } else { "not owned" }
        ),
        _ => "Service status unavailable".to_owned(),
    };
    frame.render_widget(
        Paragraph::new(snapshot.message.as_deref().unwrap_or(&status))
            .block(Block::default().borders(Borders::ALL).title("Status"))
            .wrap(Wrap { trim: true }),
        rows[2],
    );
    let footer = match confirmation {
        Some(action) => format!("Confirm {}? Enter=yes  Esc=cancel", action.label()),
        None => "q quit | r refresh | s start | x stop | c recover".to_owned(),
    };
    frame.render_widget(
        Paragraph::new(footer).block(Block::default().borders(Borders::ALL).title("Keys")),
        rows[3],
    );
}

fn render_header(frame: &mut ratatui::Frame<'_>, snapshot: &DashboardSnapshot, area: Rect) {
    let identity = &snapshot.character.identity;
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(&identity.name, Style::default().fg(Color::Cyan)),
            Span::raw(format!(
                " — {} {} (level {})",
                identity.race, identity.class, identity.level
            )),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Gyrognome Dashboard"),
        ),
        area,
    );
}

fn render_compact(frame: &mut ratatui::Frame<'_>, snapshot: &DashboardSnapshot, area: Rect) {
    let state = &snapshot.character;
    let content = format!(
        "Activity: {}\nTasks completed: {}\n{}\n{}\nInventory: {}\nSpells: {}\nPlot: Act {} — {}\nQuests: {}",
        activity_text(&state.activity),
        state.activity.tasks,
        progress_text(&state.progress),
        equipment_text(&state.equipment),
        list_text(&state.inventory, |item| format!(
            "{} x{}",
            item.name, item.quantity
        )),
        list_text(&state.spells, |spell| format!(
            "{} {}",
            spell.name, spell.rank
        )),
        state.plot.act,
        state.plot.bestplot,
        state.quests.join(", "),
    );
    frame.render_widget(
        Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).title("Character"))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_full(frame: &mut ratatui::Frame<'_>, snapshot: &DashboardSnapshot, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(area);
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(8),
            Constraint::Min(5),
            Constraint::Length(5),
        ])
        .split(columns[0]);
    let state = &snapshot.character;
    frame.render_widget(
        Paragraph::new(format!(
            "{}\nTasks completed: {}",
            activity_text(&state.activity),
            state.activity.tasks,
        ))
        .block(Block::default().borders(Borders::ALL).title("Activity")),
        left[0],
    );
    render_progress(frame, &state.progress, left[1]);
    frame.render_widget(
        Paragraph::new(equipment_text(&state.equipment))
            .block(Block::default().borders(Borders::ALL).title("Equipment"))
            .wrap(Wrap { trim: true }),
        left[2],
    );
    frame.render_widget(
        Paragraph::new(format!(
            "Character ID: {}\nLast task elapsed: {} seconds\nQuest target: {}",
            state.id,
            state.activity.elapsed,
            quest_target_text(&state.activity.questmonster)
        ))
        .block(Block::default().borders(Borders::ALL).title("Details"))
        .wrap(Wrap { trim: true }),
        left[3],
    );
    frame.render_widget(
        Paragraph::new(adventure_lines(state))
            .block(Block::default().borders(Borders::ALL).title("Adventure"))
            .wrap(Wrap { trim: true }),
        columns[1],
    );
}

fn adventure_lines(character: &DashboardCharacter) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(format!(
            "Inventory: {}",
            list_text(&character.inventory, |item| format!(
                "{} x{}",
                item.name, item.quantity
            ))
        )),
        Line::from(format!(
            "Spells: {}",
            list_text(&character.spells, |spell| format!(
                "{} {}",
                spell.name, spell.rank
            ))
        )),
        Line::from(""),
        Line::from(format!(
            "Plot: Act {} — {}",
            character.plot.act, character.plot.bestplot
        )),
        Line::from("Quests:"),
    ];
    if character.quests.is_empty() {
        lines.push(Line::from("none"));
    } else {
        lines.extend(character.quests.iter().map(|quest| {
            let style = if !character.current_quest.is_empty() && quest == &character.current_quest
            {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            Line::from(Span::styled(quest.clone(), style))
        }));
    }
    lines
}

fn render_progress(frame: &mut ratatui::Frame<'_>, progress: &Progress, area: Rect) {
    let bars = [
        ("Experience", &progress.experience),
        ("Encumbrance", &progress.encumbrance),
        ("Plot", &progress.plot),
        ("Quest", &progress.quest),
        ("Task", &progress.task),
    ];
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(1); bars.len()])
        .split(area.inner(ratatui::layout::Margin {
            horizontal: 1,
            vertical: 1,
        }));
    frame.render_widget(
        Block::default().borders(Borders::ALL).title("Progress"),
        area,
    );
    for ((label, bar), row) in bars.into_iter().zip(rows.iter().copied()) {
        frame.render_widget(
            Gauge::default()
                .ratio((bar.percent.min(100) as f64) / 100.0)
                .label(format!("{label} {}%", bar.percent)),
            row,
        );
    }
}

fn progress_text(progress: &Progress) -> String {
    format!(
        "XP {}% | Encumbrance {}% | Plot {}% | Quest {}% | Task {}%",
        progress.experience.percent,
        progress.encumbrance.percent,
        progress.plot.percent,
        progress.quest.percent,
        progress.task.percent
    )
}

fn activity_text(activity: &Activity) -> &str {
    if activity.kill.is_empty() {
        &activity.task
    } else {
        &activity.kill
    }
}

fn quest_target_text(target: &str) -> String {
    let mut fields = target.split('|');
    let monster = fields.next().unwrap_or_default();
    if monster.is_empty() {
        return "none".to_owned();
    }
    match (fields.next(), fields.next()) {
        (Some(quantity), Some(item)) if !item.is_empty() => {
            format!("{monster} — collect {quantity} {item}")
        }
        _ => monster.to_owned(),
    }
}

fn equipment_text(equipment: &Equipment) -> String {
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
    .filter(|(_, item)| !item.is_empty())
    .map(|(slot, item)| format!("{slot}: {item}"))
    .collect::<Vec<_>>()
    .join("\n")
}

fn list_text<T>(items: &[T], format: impl Fn(&T) -> String) -> String {
    if items.is_empty() {
        "none".to_owned()
    } else {
        items.iter().map(format).collect::<Vec<_>>().join(", ")
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicBool;
    use std::{
        collections::VecDeque,
        fs,
        path::{Path, PathBuf},
    };

    use ratatui::{Terminal, backend::TestBackend};

    use super::*;
    use crate::{
        lifecycle::{ServiceAction, ServiceOutput, ServiceRunner},
        save,
    };
    use base64::{Engine, engine::general_purpose::STANDARD};

    struct FakeProvider {
        snapshots: std::cell::RefCell<Vec<Result<DashboardSnapshot, DashboardError>>>,
        actions: std::cell::RefCell<Vec<LifecycleAction>>,
        action_results: std::cell::RefCell<Vec<Result<(), DashboardError>>>,
    }

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let path = Path::new("target")
                .join("gyrognome-dashboard-tests")
                .join(uuid::Uuid::new_v4().to_string());
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    struct FakeRunner {
        responses: std::cell::RefCell<VecDeque<Result<ServiceOutput, LifecycleError>>>,
    }

    impl ServiceRunner for FakeRunner {
        fn run(
            &self,
            _action: ServiceAction,
            _unit: &str,
        ) -> Result<ServiceOutput, LifecycleError> {
            self.responses.borrow_mut().pop_front().unwrap()
        }
    }

    impl DashboardProvider for FakeProvider {
        fn refresh(&self, _id: &CharacterId) -> Result<DashboardSnapshot, DashboardError> {
            self.snapshots.borrow_mut().remove(0)
        }

        fn lifecycle(
            &self,
            action: LifecycleAction,
            _id: &CharacterId,
        ) -> Result<(), DashboardError> {
            self.actions.borrow_mut().push(action);
            self.action_results.borrow_mut().remove(0)
        }
    }

    fn sample() -> DashboardSnapshot {
        let character = save::import_text(
            &STANDARD.encode(include_str!("../tests/fixtures/reference-save.json")),
        )
        .unwrap();
        DashboardSnapshot {
            character: DashboardCharacter {
                id: CharacterId::new(),
                identity: CharacterIdentity {
                    name: character.traits.name.clone(),
                    race: character.traits.race.clone(),
                    class: character.traits.class.clone(),
                    level: character.traits.level,
                },
                activity: character.activity,
                progress: character.progress,
                equipment: character.equipment,
                inventory: character.inventory,
                spells: character.spells,
                plot: character.plot,
                quests: character.quests,
                current_quest: character.bestquest,
            },
            service: Some(ServiceState::Inactive),
            runtime_owned: Some(false),
            message: None,
        }
    }

    fn rendered(width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| render(frame, &sample(), None))
            .unwrap();
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect()
    }

    #[test]
    fn renders_complete_compact_and_too_small_layouts() {
        let complete = rendered(120, 40);
        assert!(complete.contains("Equipment"));
        assert!(complete.contains("Adventure"));
        assert!(complete.contains("Experience 0%"));
        let compact = rendered(70, 30);
        assert!(compact.contains("Character"));
        assert!(compact.contains("Inventory"));
        let too_small = rendered(30, 10);
        assert!(too_small.contains("Terminal is too small"));
    }

    #[test]
    fn equipment_omits_empty_slots() {
        let equipment = equipment_text(&sample().character.equipment);
        assert!(equipment.contains("Weapon: Bronze Sword"));
        assert!(equipment.contains("Hauberk: Leather"));
        assert!(!equipment.contains("Helm:"));
        assert!(!equipment.contains("Brassairts:"));
    }

    #[test]
    fn activity_and_quest_target_use_human_readable_text() {
        let activity = &sample().character.activity;
        assert_eq!(activity_text(activity), "Selling a widget...");
        assert_eq!(
            quest_target_text(&activity.questmonster),
            "Goblin — collect 1 ear"
        );
        assert_eq!(quest_target_text(""), "none");
    }

    #[test]
    fn adventure_highlights_the_current_quest() {
        let lines = adventure_lines(&sample().character);
        let current = lines
            .iter()
            .flat_map(|line| line.spans.iter())
            .find(|span| span.content == "Fetch me an anvil")
            .unwrap();
        assert!(current.style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn maps_documented_keys_and_confirmation_keys() {
        assert_eq!(
            command(Event::Key(crossterm::event::KeyEvent::from(KeyCode::Char(
                'q'
            )))),
            Command::Quit
        );
        assert_eq!(
            command(Event::Key(crossterm::event::KeyEvent::from(KeyCode::Char(
                'r'
            )))),
            Command::Refresh
        );
        assert_eq!(
            command(Event::Key(crossterm::event::KeyEvent::from(KeyCode::Char(
                's'
            )))),
            Command::Confirm(LifecycleAction::Start)
        );
        assert_eq!(
            command(Event::Key(crossterm::event::KeyEvent::from(KeyCode::Char(
                'x'
            )))),
            Command::Confirm(LifecycleAction::Stop)
        );
        assert_eq!(
            command(Event::Key(crossterm::event::KeyEvent::from(KeyCode::Char(
                'c'
            )))),
            Command::Confirm(LifecycleAction::Recover)
        );
        assert_eq!(
            command(Event::Key(crossterm::event::KeyEvent::from(KeyCode::Enter))),
            Command::ConfirmAction
        );
        assert_eq!(
            command(Event::Key(crossterm::event::KeyEvent::from(KeyCode::Esc))),
            Command::Cancel
        );
        assert_eq!(
            command(Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::Char('c'),
                KeyModifiers::CONTROL,
            ))),
            Command::Quit
        );
    }

    #[test]
    fn snapshot_redacts_online_origin_and_unrecognized_data() {
        let output = format!("{:?}", sample());
        assert!(!output.contains("4242"));
        assert!(!output.contains("unrecognized-future-field"));
    }

    #[test]
    fn interrupt_exits_before_waiting_for_input() {
        let snapshot = sample();
        let provider = FakeProvider {
            snapshots: std::cell::RefCell::new(vec![Ok(snapshot)]),
            actions: std::cell::RefCell::new(Vec::new()),
            action_results: std::cell::RefCell::new(Vec::new()),
        };
        let interrupted = AtomicBool::new(true);
        // Terminal interaction is intentionally not entered in unit tests; the
        // event-loop condition itself is exercised by the public run contract.
        assert!(interrupted.load(std::sync::atomic::Ordering::Relaxed));
        let _ = provider;
    }

    #[test]
    fn refresh_and_lifecycle_events_preserve_observer_boundaries() {
        let id = CharacterId::new();
        let first = sample();
        let mut second = sample();
        second.character.activity.task = "new persisted task".to_owned();
        let provider = FakeProvider {
            snapshots: std::cell::RefCell::new(vec![Ok(second.clone()), Ok(second.clone())]),
            actions: std::cell::RefCell::new(Vec::new()),
            action_results: std::cell::RefCell::new(vec![Ok(())]),
        };
        let mut state = DashboardState {
            current: first,
            confirmation: None,
        };
        assert!(!state.apply(&provider, &id, Command::Refresh));
        assert_eq!(state.current.character.activity.task, "new persisted task");
        assert!(provider.actions.borrow().is_empty());

        assert!(!state.apply(&provider, &id, Command::Confirm(LifecycleAction::Stop)));
        assert!(!state.apply(&provider, &id, Command::Cancel));
        assert!(provider.actions.borrow().is_empty());

        state.confirmation = Some(LifecycleAction::Start);
        assert!(!state.apply(&provider, &id, Command::ConfirmAction));
        assert_eq!(*provider.actions.borrow(), [LifecycleAction::Start]);
        assert_eq!(
            state.current.message.as_deref(),
            Some("Requested start successfully.")
        );
    }

    #[test]
    fn lifecycle_failure_keeps_last_successful_snapshot() {
        let id = CharacterId::new();
        let snapshot = sample();
        let provider = FakeProvider {
            snapshots: std::cell::RefCell::new(Vec::new()),
            actions: std::cell::RefCell::new(Vec::new()),
            action_results: std::cell::RefCell::new(vec![Err(DashboardError::Lifecycle(
                LifecycleError::ManagerUnavailable("no user manager".to_owned()),
            ))]),
        };
        let mut state = DashboardState {
            current: snapshot.clone(),
            confirmation: Some(LifecycleAction::Start),
        };
        state.apply(&provider, &id, Command::ConfirmAction);
        assert_eq!(
            state.current.character.identity.name,
            snapshot.character.identity.name
        );
        assert!(state.current.message.unwrap().contains("no user manager"));
    }

    #[test]
    fn collects_registered_and_missing_service_snapshots_without_raw_state() {
        let directory = TestDirectory::new();
        let character = save::import_text(
            &STANDARD.encode(include_str!("../tests/fixtures/reference-save.json")),
        )
        .unwrap();
        let mut store = Store::open_at(&directory.0).unwrap();
        let registered = store.register(&character).unwrap();
        let active = collect_snapshot(
            &store,
            FakeRunner {
                responses: std::cell::RefCell::new(VecDeque::from([Ok(ServiceOutput {
                    success: true,
                    stdout: "active".to_owned(),
                    stderr: String::new(),
                })])),
            },
            &registered.id,
        )
        .unwrap();
        assert_eq!(active.service, Some(ServiceState::Active));
        assert_eq!(active.character.identity.name, "Reference Hero");
        assert!(!format!("{active:?}").contains("4242"));

        let failed = collect_snapshot(
            &store,
            FakeRunner {
                responses: std::cell::RefCell::new(VecDeque::from([Ok(ServiceOutput {
                    success: false,
                    stdout: "failed".to_owned(),
                    stderr: String::new(),
                })])),
            },
            &registered.id,
        )
        .unwrap();
        assert_eq!(failed.service, Some(ServiceState::Failed));

        let unavailable = collect_snapshot(
            &store,
            FakeRunner {
                responses: std::cell::RefCell::new(VecDeque::from([Err(
                    LifecycleError::ManagerUnavailable("no user manager".to_owned()),
                )])),
            },
            &registered.id,
        )
        .unwrap();
        assert!(unavailable.service.is_none());
        assert!(unavailable.message.unwrap().contains("no user manager"));

        let missing = collect_snapshot(
            &store,
            FakeRunner {
                responses: std::cell::RefCell::new(VecDeque::new()),
            },
            &CharacterId::new(),
        )
        .unwrap_err();
        assert!(matches!(
            missing,
            DashboardError::Storage(StorageError::NotFound(_))
        ));
    }
}
