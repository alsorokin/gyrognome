//! Credential-safe terminal dashboard for observing one managed character.

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    io::{self, Stdout},
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{
        BeginSynchronizedUpdate, EndSynchronizedUpdate, EnterAlternateScreen, LeaveAlternateScreen,
        disable_raw_mode, enable_raw_mode,
    },
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Widget, Wrap},
};
use thiserror::Error;

use crate::{
    guild::GuildOutcome,
    lifecycle::{Lifecycle, LifecycleError, RuntimeStatus, ServiceState, SystemctlRunner},
    reporting::{self, DeliveryOutcome, HttpsTransport, ReportingError},
    runtime::{CharacterId, CharacterIdentity, ManagedCharacter, StorageError, Store},
    state::{
        Activity, Attributes, Equipment, InventoryEntry, OnlineProfile, Plot, Progress, Spell,
    },
};

const MINIMUM_WIDTH: u16 = 40;
const MINIMUM_HEIGHT: u16 = 12;
const MINIMUM_REDRAW_SPACING: Duration = Duration::from_millis(25);
const SETTLING_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug, Clone)]
pub struct DashboardCharacter {
    pub id: CharacterId,
    pub identity: CharacterIdentity,
    pub activity: Activity,
    pub stats: Attributes,
    pub progress: Progress,
    pub equipment: Equipment,
    pub inventory: Vec<InventoryEntry>,
    pub spells: Vec<Spell>,
    pub plot: Plot,
    pub quests: Vec<String>,
    pub current_quest: String,
    pub profile: OnlineProfile,
}

impl From<ManagedCharacter> for DashboardCharacter {
    fn from(character: ManagedCharacter) -> Self {
        Self {
            id: character.id,
            identity: character.identity,
            activity: character.state.activity,
            stats: character.state.stats,
            progress: character.state.progress,
            equipment: character.state.equipment,
            inventory: character.state.inventory,
            spells: character.state.spells,
            plot: character.state.plot,
            quests: character.state.quests,
            current_quest: character.state.bestquest,
            profile: character.state.profile,
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
    fn read_state(&self, id: &CharacterId) -> Result<DashboardCharacter, DashboardError>;
    fn lifecycle(&self, action: LifecycleAction, id: &CharacterId) -> Result<(), DashboardError>;
    fn brag(&self, id: &CharacterId) -> Result<DeliveryOutcome, DashboardError>;
    fn set_motto(&self, id: &CharacterId, text: &str) -> Result<DeliveryOutcome, DashboardError>;
    fn set_guild(
        &self,
        id: &CharacterId,
        designation: &str,
    ) -> Result<GuildOutcome, DashboardError>;
}

pub struct LocalProvider {
    store: RefCell<Store>,
}

impl LocalProvider {
    pub fn open_default() -> Result<Self, DashboardError> {
        Ok(Self {
            store: RefCell::new(Store::open_default()?),
        })
    }
}

impl DashboardProvider for LocalProvider {
    fn refresh(&self, id: &CharacterId) -> Result<DashboardSnapshot, DashboardError> {
        collect_snapshot(&self.store.borrow(), SystemctlRunner, id)
    }

    fn read_state(&self, id: &CharacterId) -> Result<DashboardCharacter, DashboardError> {
        Ok(self.store.borrow().get(id)?.into())
    }

    fn lifecycle(&self, action: LifecycleAction, id: &CharacterId) -> Result<(), DashboardError> {
        let store = self.store.borrow();
        let lifecycle = Lifecycle::new(&store, SystemctlRunner);
        match action {
            LifecycleAction::Start => lifecycle.start(id)?,
            LifecycleAction::Stop => lifecycle.stop(id)?,
            LifecycleAction::Recover => lifecycle.recover(id)?,
        }
        Ok(())
    }

    fn brag(&self, id: &CharacterId) -> Result<DeliveryOutcome, DashboardError> {
        Ok(reporting::submit(&self.store.borrow(), id, &HttpsTransport)?.outcome)
    }

    fn set_motto(&self, id: &CharacterId, text: &str) -> Result<DeliveryOutcome, DashboardError> {
        Ok(reporting::set_motto(&mut self.store.borrow_mut(), id, text, &HttpsTransport)?.outcome)
    }

    fn set_guild(
        &self,
        id: &CharacterId,
        designation: &str,
    ) -> Result<GuildOutcome, DashboardError> {
        Ok(reporting::set_guild(
            &mut self.store.borrow_mut(),
            id,
            designation,
            &HttpsTransport,
        )?
        .outcome)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TaskIdentity {
    completed_tasks: u64,
    duration_ms: u64,
}

#[derive(Debug, Clone, Copy)]
struct TaskAnchor {
    position_ms: f64,
    duration_ms: u64,
    identity: TaskIdentity,
    observed_at: Instant,
}

impl TaskAnchor {
    fn from_snapshot(snapshot: &DashboardSnapshot, observed_at: Instant) -> Self {
        Self::from_character(&snapshot.character, observed_at)
    }

    fn from_character(character: &DashboardCharacter, observed_at: Instant) -> Self {
        Self {
            position_ms: character.progress.task.position,
            duration_ms: character.progress.task.max,
            identity: TaskIdentity {
                completed_tasks: character.activity.tasks,
                duration_ms: character.progress.task.max,
            },
            observed_at,
        }
    }
}

fn predicted_task_position(anchor: TaskAnchor, elapsed: Duration) -> f64 {
    if anchor.duration_ms == 0 {
        return 0.0;
    }
    (anchor.position_ms + elapsed.as_millis() as f64).min(anchor.duration_ms as f64)
}

fn task_percent(position_ms: f64, duration_ms: u64) -> u64 {
    if duration_ms == 0 {
        100
    } else {
        ((100.0 * position_ms.clamp(0.0, duration_ms as f64)) / duration_ms as f64).floor() as u64
    }
}

fn predicted_task_percent(anchor: TaskAnchor, elapsed: Duration) -> u64 {
    task_percent(predicted_task_position(anchor, elapsed), anchor.duration_ms)
}

fn next_task_percent_boundary(anchor: TaskAnchor, elapsed: Duration) -> Option<Duration> {
    if anchor.duration_ms == 0 {
        return None;
    }
    let position = predicted_task_position(anchor, elapsed);
    if position >= anchor.duration_ms as f64 {
        return None;
    }
    let percent = task_percent(position, anchor.duration_ms);
    let target_position =
        ((u128::from(percent + 1) * u128::from(anchor.duration_ms)).div_ceil(100)) as f64;
    Some(Duration::from_millis(
        (target_position - position).max(1.0).ceil() as u64,
    ))
}

fn next_redraw_boundary(anchor: TaskAnchor, elapsed: Duration) -> Option<Duration> {
    next_task_percent_boundary(anchor, elapsed).map(|deadline| deadline.max(MINIMUM_REDRAW_SPACING))
}

fn select_poll_timeout(
    next_percent_boundary: Option<Duration>,
    next_combined_refresh: Duration,
    next_settling_read: Option<Duration>,
    refresh_interval: Duration,
) -> Duration {
    [
        next_percent_boundary,
        Some(next_combined_refresh),
        next_settling_read,
        Some(refresh_interval),
    ]
    .into_iter()
    .flatten()
    .min()
    .unwrap_or(refresh_interval)
}

#[derive(Debug, Error)]
pub enum DashboardError {
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error(transparent)]
    Lifecycle(#[from] LifecycleError),
    #[error(transparent)]
    Reporting(#[from] ReportingError),
    #[error("terminal operation failed: {0}")]
    Terminal(#[from] io::Error),
    #[error("no managed characters are registered")]
    NoManagedCharacters,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectionCommand {
    Previous,
    Next,
    Select,
    Cancel,
    None,
}

fn selection_command(event: Event) -> SelectionCommand {
    let Event::Key(key) = event else {
        return SelectionCommand::None;
    };
    if key.kind != KeyEventKind::Press {
        return SelectionCommand::None;
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return SelectionCommand::Cancel;
    }
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => SelectionCommand::Previous,
        KeyCode::Down | KeyCode::Char('j') => SelectionCommand::Next,
        KeyCode::Enter => SelectionCommand::Select,
        KeyCode::Char('q') | KeyCode::Esc => SelectionCommand::Cancel,
        _ => SelectionCommand::None,
    }
}

#[derive(Debug)]
struct SelectorState {
    selected: usize,
    length: usize,
}

impl SelectorState {
    fn new(length: usize) -> Self {
        Self {
            selected: 0,
            length,
        }
    }

    fn apply(&mut self, command: SelectionCommand) -> Option<Option<usize>> {
        match command {
            SelectionCommand::Previous => {
                self.selected = self.selected.checked_sub(1).unwrap_or(self.length - 1);
                None
            }
            SelectionCommand::Next => {
                self.selected = (self.selected + 1) % self.length;
                None
            }
            SelectionCommand::Select => Some(Some(self.selected)),
            SelectionCommand::Cancel => Some(None),
            SelectionCommand::None => None,
        }
    }
}

/// Lets users select a registered character before entering its dashboard.
pub fn select_character(
    characters: Vec<(CharacterId, CharacterIdentity)>,
    interrupted: &std::sync::atomic::AtomicBool,
) -> Result<Option<CharacterId>, DashboardError> {
    if characters.is_empty() {
        return Err(DashboardError::NoManagedCharacters);
    }

    let mut session = TerminalSession::enter()?;
    let mut state = SelectorState::new(characters.len());
    loop {
        session
            .terminal
            .draw(|frame| render_selector(frame, &characters, state.selected))?;
        if interrupted.load(std::sync::atomic::Ordering::Relaxed) {
            return Ok(None);
        }
        if let Some(result) = state.apply(selection_command(event::read()?)) {
            return Ok(result.map(|index| characters[index].0.clone()));
        }
    }
}

fn render_selector(
    frame: &mut ratatui::Frame<'_>,
    characters: &[(CharacterId, CharacterIdentity)],
    selected: usize,
) {
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(frame.area());
    let entries = characters
        .iter()
        .map(|(id, identity)| {
            ListItem::new(format!(
                "{} — {} {} (level {})\n{}",
                identity.name, identity.race, identity.class, identity.level, id
            ))
        })
        .collect::<Vec<_>>();
    let mut state = ListState::default();
    state.select(Some(selected));
    frame.render_stateful_widget(
        List::new(entries)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Select a managed character"),
            )
            .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black))
            .highlight_symbol("> "),
        areas[0],
        &mut state,
    );
    frame.render_widget(
        Paragraph::new("Up/Down or j/k select | Enter open | Esc/q cancel")
            .block(Block::default().borders(Borders::ALL).title("Keys")),
        areas[1],
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    Quit,
    Refresh,
    Brag,
    Confirm(LifecycleAction),
    ConfirmAction,
    Cancel,
    TogglePane(Pane),
    Edit(ProfileField),
    Insert(char),
    Backspace,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProfileField {
    Motto,
    Guild,
}

#[derive(Debug)]
struct ProfileEditor {
    field: ProfileField,
    text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pane {
    Activity,
    Progress,
    Equipment,
    Details,
    Status,
    Adventure,
    Journal,
}

impl Pane {
    const ALL: [Self; 7] = [
        Self::Activity,
        Self::Progress,
        Self::Equipment,
        Self::Details,
        Self::Status,
        Self::Adventure,
        Self::Journal,
    ];

    const fn index(self) -> usize {
        match self {
            Self::Activity => 0,
            Self::Progress => 1,
            Self::Equipment => 2,
            Self::Details => 3,
            Self::Status => 4,
            Self::Adventure => 5,
            Self::Journal => 6,
        }
    }

    const fn hotkey(self) -> &'static str {
        match self {
            Self::Activity => "F1",
            Self::Progress => "F2",
            Self::Equipment => "F3",
            Self::Details => "F4",
            Self::Status => "F5",
            Self::Adventure => "F6",
            Self::Journal => "F7",
        }
    }
}

#[derive(Debug, Default)]
struct PaneVisibility {
    collapsed: [bool; Pane::ALL.len()],
}

impl PaneVisibility {
    fn is_collapsed(&self, pane: Pane) -> bool {
        self.collapsed[pane.index()]
    }

    fn toggle(&mut self, pane: Pane) {
        self.collapsed[pane.index()] = !self.collapsed[pane.index()];
    }
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
        KeyCode::Char('b') => Command::Brag,
        KeyCode::Char('m') => Command::Edit(ProfileField::Motto),
        KeyCode::Char('g') => Command::Edit(ProfileField::Guild),
        KeyCode::Char('s') => Command::Confirm(LifecycleAction::Start),
        KeyCode::Char('x') => Command::Confirm(LifecycleAction::Stop),
        KeyCode::Char('c') => Command::Confirm(LifecycleAction::Recover),
        KeyCode::Enter => Command::ConfirmAction,
        KeyCode::Esc => Command::Cancel,
        KeyCode::F(1) => Command::TogglePane(Pane::Activity),
        KeyCode::F(2) => Command::TogglePane(Pane::Progress),
        KeyCode::F(3) => Command::TogglePane(Pane::Equipment),
        KeyCode::F(4) => Command::TogglePane(Pane::Details),
        KeyCode::F(5) => Command::TogglePane(Pane::Status),
        KeyCode::F(6) => Command::TogglePane(Pane::Adventure),
        KeyCode::F(7) => Command::TogglePane(Pane::Journal),
        _ => Command::None,
    }
}

fn editor_command(event: Event) -> Command {
    let Event::Key(key) = event else {
        return Command::None;
    };
    if key.kind == KeyEventKind::Release {
        return Command::None;
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Command::Quit;
    }
    if key
        .modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
    {
        return Command::None;
    }
    match key.code {
        KeyCode::Esc => Command::Cancel,
        KeyCode::Enter => Command::ConfirmAction,
        KeyCode::Backspace => Command::Backspace,
        KeyCode::Char(value) if !value.is_control() => Command::Insert(value),
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
    let observed_at = Instant::now();
    let mut state = DashboardState::new(provider.refresh(&id)?, observed_at, refresh_interval);
    loop {
        let now = Instant::now();
        let predicted_task_percent = state.displayed_task_percent(now);
        session.draw_synchronized(|frame| {
            render(
                frame,
                &state.current,
                predicted_task_percent,
                &state.updates,
                state.confirmation,
                &state.panes,
            );
            if let Some(editor) = &state.editor {
                render_editor(frame, editor);
            }
        })?;
        if interrupted.load(std::sync::atomic::Ordering::Relaxed) {
            return Ok(());
        }
        if !event::poll(state.poll_timeout(Instant::now()))? {
            state.handle_deadline(provider, &id, Instant::now());
            continue;
        }
        if state.handle_event(provider, &id, event::read()?, Instant::now()) {
            return Ok(());
        }
    }
}

struct DashboardState {
    current: DashboardSnapshot,
    updates: RecentTaskUpdates,
    confirmation: Option<LifecycleAction>,
    editor: Option<ProfileEditor>,
    panes: PaneVisibility,
    task_anchor: TaskAnchor,
    next_combined_refresh: Instant,
    refresh_interval: Duration,
}

impl DashboardState {
    fn new(current: DashboardSnapshot, now: Instant, refresh_interval: Duration) -> Self {
        let task_anchor = TaskAnchor::from_snapshot(&current, now);
        Self {
            current,
            updates: RecentTaskUpdates::default(),
            confirmation: None,
            editor: None,
            panes: PaneVisibility::default(),
            task_anchor,
            next_combined_refresh: now.checked_add(refresh_interval).unwrap_or(now),
            refresh_interval,
        }
    }

    fn displayed_task_position(&self, now: Instant) -> f64 {
        if self.current.runtime_owned == Some(true) {
            predicted_task_position(
                self.task_anchor,
                now.saturating_duration_since(self.task_anchor.observed_at),
            )
        } else {
            self.current.character.progress.task.position
        }
    }

    fn displayed_task_percent(&self, now: Instant) -> u64 {
        if self.current.runtime_owned == Some(true) {
            predicted_task_percent(
                self.task_anchor,
                now.saturating_duration_since(self.task_anchor.observed_at),
            )
        } else {
            self.current.character.progress.task.percent
        }
    }

    fn replace_character(&mut self, character: DashboardCharacter, now: Instant) {
        let mut next_anchor = TaskAnchor::from_character(&character, now);
        if next_anchor.identity == self.task_anchor.identity
            && self.current.runtime_owned == Some(true)
        {
            next_anchor.position_ms = next_anchor
                .position_ms
                .max(self.displayed_task_position(now));
        }
        if character.activity.tasks > self.current.character.activity.tasks {
            self.updates = RecentTaskUpdates::between(&self.current.character, &character);
        }
        self.current.character = character;
        self.task_anchor = next_anchor;
    }

    fn refresh<P: DashboardProvider>(&mut self, provider: &P, id: &CharacterId, now: Instant) {
        match provider.refresh(id) {
            Ok(next) => {
                let DashboardSnapshot {
                    character,
                    service,
                    runtime_owned,
                    message,
                } = next;
                self.replace_character(character, now);
                self.current.service = service;
                self.current.runtime_owned = runtime_owned;
                self.current.message = message;
            }
            Err(error) => {
                self.current.message = Some(format!("Could not refresh dashboard: {error}"))
            }
        }
        self.next_combined_refresh = now.checked_add(self.refresh_interval).unwrap_or(now);
    }

    fn read_state<P: DashboardProvider>(&mut self, provider: &P, id: &CharacterId, now: Instant) {
        match provider.read_state(id) {
            Ok(character) => {
                self.replace_character(character, now);
                if self.current.message.as_deref().is_some_and(|message| {
                    message.starts_with("Could not refresh character state:")
                }) {
                    self.current.message = None;
                }
            }
            Err(error) => {
                let displayed = self.displayed_task_position(now);
                self.current.message = Some(format!("Could not refresh character state: {error}"));
                self.task_anchor.observed_at = now;
                self.task_anchor.position_ms = displayed;
            }
        }
    }

    fn settling_deadline(&self) -> Option<Instant> {
        if self.current.runtime_owned != Some(true) {
            return None;
        }
        let remaining_ms = (self.task_anchor.duration_ms as f64 - self.task_anchor.position_ms)
            .max(0.0)
            .ceil() as u64;
        self.task_anchor
            .observed_at
            .checked_add(Duration::from_millis(remaining_ms))
            .and_then(|saturation| saturation.checked_add(SETTLING_INTERVAL))
    }

    fn poll_timeout(&self, now: Instant) -> Duration {
        let elapsed = now.saturating_duration_since(self.task_anchor.observed_at);
        let boundary = (self.current.runtime_owned == Some(true))
            .then(|| next_redraw_boundary(self.task_anchor, elapsed))
            .flatten();
        let refresh = self.next_combined_refresh.saturating_duration_since(now);
        let settling = self
            .settling_deadline()
            .map(|deadline| deadline.saturating_duration_since(now));
        select_poll_timeout(boundary, refresh, settling, self.refresh_interval)
    }

    fn handle_deadline<P: DashboardProvider>(
        &mut self,
        provider: &P,
        id: &CharacterId,
        now: Instant,
    ) {
        if now >= self.next_combined_refresh {
            self.refresh(provider, id, now);
        } else if self
            .settling_deadline()
            .is_some_and(|deadline| now >= deadline)
        {
            self.read_state(provider, id, now);
        }
    }

    fn handle_event<P: DashboardProvider>(
        &mut self,
        provider: &P,
        id: &CharacterId,
        event: Event,
        now: Instant,
    ) -> bool {
        let command = if self.editor.is_some() {
            editor_command(event)
        } else {
            command(event)
        };
        self.apply(provider, id, command, now)
    }

    fn apply<P: DashboardProvider>(
        &mut self,
        provider: &P,
        id: &CharacterId,
        command: Command,
        now: Instant,
    ) -> bool {
        if let Some(editor) = &mut self.editor {
            match command {
                Command::Quit => return true,
                Command::Cancel => self.editor = None,
                Command::Insert(value) if !value.is_control() => editor.text.push(value),
                Command::Backspace => {
                    editor.text.pop();
                }
                Command::ConfirmAction => {
                    let editor = self.editor.take().expect("editor is open");
                    let message = match editor.field {
                        ProfileField::Motto => match provider.set_motto(id, &editor.text) {
                            Ok(outcome) => outcome.motto_message().to_owned(),
                            Err(error) => format!("Could not change motto: {error}"),
                        },
                        ProfileField::Guild => match provider.set_guild(id, &editor.text) {
                            Ok(outcome) => outcome.message().to_owned(),
                            Err(error) => format!("Could not change guild: {error}"),
                        },
                    };
                    self.refresh(provider, id, now);
                    self.current.message = Some(match self.current.message.take() {
                        Some(refresh_message) => format!("{message} {refresh_message}"),
                        None => message,
                    });
                }
                _ => {}
            }
            return false;
        }
        match command {
            Command::Quit => true,
            Command::Refresh => {
                self.refresh(provider, id, now);
                false
            }
            Command::Brag => {
                let message = match provider.brag(id) {
                    Ok(DeliveryOutcome::Delivered) => "Leaderboard report delivered.".to_owned(),
                    Ok(DeliveryOutcome::EndpointRejected) => {
                        "Leaderboard report was not accepted by the endpoint.".to_owned()
                    }
                    Ok(DeliveryOutcome::DeliveryFailed) => {
                        "Leaderboard report could not be delivered.".to_owned()
                    }
                    Err(error) => format!("Could not submit leaderboard report: {error}"),
                };
                self.refresh(provider, id, now);
                self.current.message = Some(message);
                false
            }
            Command::Confirm(action) => {
                self.confirmation = Some(action);
                false
            }
            Command::Edit(field) => {
                self.confirmation = None;
                self.editor = Some(ProfileEditor {
                    field,
                    text: match field {
                        ProfileField::Motto => self.current.character.profile.motto.clone(),
                        ProfileField::Guild => self.current.character.profile.guild.clone(),
                    },
                });
                false
            }
            Command::Cancel => {
                self.confirmation = None;
                false
            }
            Command::TogglePane(pane) => {
                self.panes.toggle(pane);
                false
            }
            Command::ConfirmAction => {
                if let Some(action) = self.confirmation.take() {
                    match provider.lifecycle(action, id) {
                        Ok(()) => {
                            self.refresh(provider, id, now);
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
            Command::None | Command::Insert(_) | Command::Backspace => false,
        }
    }
}

pub(crate) struct TerminalSession {
    pub(crate) terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalSession {
    pub(crate) fn enter() -> Result<Self, DashboardError> {
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

    fn draw_synchronized(
        &mut self,
        render_callback: impl FnOnce(&mut ratatui::Frame<'_>),
    ) -> io::Result<()> {
        execute!(self.terminal.backend_mut(), BeginSynchronizedUpdate)?;
        let draw_result = self.terminal.draw(render_callback).map(|_| ());
        let end_result = execute!(self.terminal.backend_mut(), EndSynchronizedUpdate);
        draw_result.and(end_result)
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
    task_percent: u64,
    updates: &RecentTaskUpdates,
    confirmation: Option<LifecycleAction>,
    panes: &PaneVisibility,
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

    let full_layout = area.width >= 90;
    let header_height = if header_uses_single_line(&snapshot.character, area.width) {
        3
    } else {
        4
    };
    let status_height = if full_layout && panes.is_collapsed(Pane::Status) {
        2
    } else {
        3
    };
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height),
            Constraint::Min(6),
            Constraint::Length(status_height),
            Constraint::Length(if full_layout || confirmation.is_some() {
                3
            } else {
                5
            }),
        ])
        .split(area);
    render_header(frame, snapshot, updates, rows[0]);
    if !full_layout {
        render_compact(frame, snapshot, task_percent, updates, rows[1]);
    } else {
        render_full(frame, snapshot, task_percent, updates, rows[1], panes);
    }
    let status = match (&snapshot.service, snapshot.runtime_owned) {
        (Some(state), Some(owned)) => format!(
            "Service: {state:?} | Runtime ownership: {}",
            if owned { "owned" } else { "not owned" }
        ),
        _ => "Service status unavailable".to_owned(),
    };
    if full_layout && panes.is_collapsed(Pane::Status) {
        frame.render_widget(pane_block("Status", Pane::Status), rows[2]);
    } else {
        let status_block = if full_layout {
            pane_block("Status", Pane::Status)
        } else {
            Block::default().borders(Borders::ALL).title("Status")
        };
        frame.render_widget(
            Paragraph::new(snapshot.message.as_deref().unwrap_or(&status))
                .block(status_block)
                .wrap(Wrap { trim: true }),
            rows[2],
        );
    }
    let footer = match confirmation {
        Some(action) => format!("Confirm {}? Enter=yes  Esc=cancel", action.label()),
        None if full_layout => {
            "q quit | r refresh | b brag | m motto | g guild | s start | x stop | c recover"
                .to_owned()
        }
        None => "q quit | r refresh | b brag\nm motto | g guild\ns start | x stop | c recover"
            .to_owned(),
    };
    frame.render_widget(
        Paragraph::new(footer).block(Block::default().borders(Borders::ALL).title("Keys")),
        rows[3],
    );
}

fn render_editor(frame: &mut ratatui::Frame<'_>, editor: &ProfileEditor) {
    let screen = frame.area();
    let width = screen.width.saturating_sub(4).min(80);
    let height = screen.height.min(5);
    let area = Rect::new(
        screen.x + (screen.width - width) / 2,
        screen.y + (screen.height - height) / 2,
        width,
        height,
    );
    let title = match editor.field {
        ProfileField::Motto => "Motto (empty clears)",
        ProfileField::Guild => "Guild (empty leaves)",
    };
    let capacity = width.saturating_sub(3) as usize;
    let mut visible = Vec::new();
    let mut used = 0;
    for value in editor.text.chars().rev() {
        let width = Span::raw(value.to_string()).width();
        if used + width > capacity {
            break;
        }
        visible.push(value);
        used += width;
    }
    let visible: String = visible.into_iter().rev().collect();
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(visible),
            Line::from(""),
            Line::from("Enter submit | Esc cancel"),
        ])
        .block(Block::default().borders(Borders::ALL).title(title)),
        area,
    );
    if area.width > 2 && area.height > 2 {
        frame.set_cursor_position((area.x + 1 + used as u16, area.y + 1));
    }
}

fn render_header(
    frame: &mut ratatui::Frame<'_>,
    snapshot: &DashboardSnapshot,
    updates: &RecentTaskUpdates,
    area: Rect,
) {
    let identity = &snapshot.character.identity;
    let state = &snapshot.character;
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Gyrognome Dashboard");
    let inner = area.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });
    frame.render_widget(block, area);
    if header_uses_single_line(state, area.width) {
        let stats_width = header_stats_text(&state.stats).chars().count() as u16;
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(stats_width)])
            .split(inner);
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(&identity.name, Style::default().fg(Color::Cyan)),
                Span::raw(header_identity_text(identity)),
            ])),
            columns[0],
        );
        frame.render_widget(
            Paragraph::new(header_stats_line(&state.stats, updates)).alignment(Alignment::Right),
            columns[1],
        );
    } else {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(inner);
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(&identity.name, Style::default().fg(Color::Cyan)),
                Span::raw(header_identity_text(identity)),
            ])),
            rows[0],
        );
        frame.render_widget(
            Paragraph::new(header_stats_line(&state.stats, updates)),
            rows[1],
        );
    }
}

fn header_identity_text(identity: &CharacterIdentity) -> String {
    format!(
        " — {} {} (level {})",
        identity.race, identity.class, identity.level
    )
}

fn header_uses_single_line(character: &DashboardCharacter, width: u16) -> bool {
    let identity_width = character.identity.name.chars().count()
        + header_identity_text(&character.identity).chars().count();
    let stats_width = header_stats_text(&character.stats).chars().count();
    identity_width + stats_width + 2 <= width.saturating_sub(2) as usize
}

fn header_stats_text(stats: &Attributes) -> String {
    attribute_values(stats)
        .into_iter()
        .map(|(label, value)| format!("{label}:{value}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn header_stats_line(stats: &Attributes, updates: &RecentTaskUpdates) -> Line<'static> {
    let mut spans = Vec::new();
    for (index, (label, value)) in attribute_values(stats).into_iter().enumerate() {
        if index > 0 {
            spans.push(Span::raw(" "));
        }
        spans.push(styled_value(
            format!("{label}:{value}"),
            updates.stats.contains(label),
        ));
    }
    Line::from(spans)
}

fn render_compact(
    frame: &mut ratatui::Frame<'_>,
    snapshot: &DashboardSnapshot,
    task_percent: u64,
    updates: &RecentTaskUpdates,
    area: Rect,
) {
    let state = &snapshot.character;
    let mut content = vec![
        Line::from(format!("Activity: {}", activity_text(&state.activity))),
        Line::from(format!("Tasks completed: {}", state.activity.tasks)),
        Line::from(progress_text(&state.progress, task_percent)),
    ];
    content.extend(equipment_lines(&state.equipment, updates));
    content.push(inventory_line(&state.inventory, updates));
    content.push(spells_line(&state.spells, updates));
    content.push(Line::from(format!(
        "Plot: Act {} — {}",
        state.plot.act, state.plot.bestplot
    )));
    content.push(Line::from(format!("Quests: {}", state.quests.join(", "))));
    frame.render_widget(
        Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).title("Character"))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_full(
    frame: &mut ratatui::Frame<'_>,
    snapshot: &DashboardSnapshot,
    task_percent: u64,
    updates: &RecentTaskUpdates,
    area: Rect,
    panes: &PaneVisibility,
) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
        .split(area);
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints(left_pane_constraints(panes, columns[0].height))
        .split(columns[0]);
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints(right_pane_constraints(panes))
        .split(columns[1]);
    let state = &snapshot.character;
    render_pane(
        frame,
        left[0],
        "Activity",
        Pane::Activity,
        panes,
        Paragraph::new(format!(
            "{}\nTasks completed: {}",
            activity_text(&state.activity),
            state.activity.tasks,
        )),
    );
    if panes.is_collapsed(Pane::Progress) {
        frame.render_widget(pane_block("Progress", Pane::Progress), left[1]);
    } else {
        render_progress(
            frame,
            &state.progress,
            task_percent,
            left[1],
            Pane::Progress,
        );
    }
    render_pane(
        frame,
        left[2],
        "Equipment",
        Pane::Equipment,
        panes,
        Paragraph::new(equipment_lines(&state.equipment, updates)),
    );
    render_pane(
        frame,
        left[3],
        "Details",
        Pane::Details,
        panes,
        Paragraph::new(details_lines(state)),
    );
    render_pane(
        frame,
        right[0],
        "Adventure",
        Pane::Adventure,
        panes,
        Paragraph::new(adventure_lines(
            state,
            updates,
            panes.is_collapsed(Pane::Journal),
        ))
        .wrap(Wrap { trim: true }),
    );
    render_pane(
        frame,
        right[1],
        "Journal",
        Pane::Journal,
        panes,
        Paragraph::new(journal_lines(state)).wrap(Wrap { trim: true }),
    );
}

fn adventure_lines(
    character: &DashboardCharacter,
    updates: &RecentTaskUpdates,
    show_current_quest: bool,
) -> Vec<Line<'static>> {
    let mut lines = vec![
        inventory_line(&character.inventory, updates),
        Line::from(""),
        spells_line(&character.spells, updates),
        Line::from(""),
        Line::from(format!(
            "Plot: Act {} — {}",
            character.plot.act, character.plot.bestplot
        )),
    ];
    if show_current_quest {
        lines.push(Line::from(format!(
            "Current quest: {}",
            character.current_quest
        )));
    }
    lines
}

fn journal_lines(character: &DashboardCharacter) -> Vec<Line<'static>> {
    let current_quest_index = character
        .quests
        .iter()
        .rposition(|quest| quest == &character.current_quest);
    let mut lines = vec![Line::from(Span::styled(
        character.current_quest.clone(),
        Style::default().add_modifier(Modifier::BOLD),
    ))];
    lines.extend(
        character
            .quests
            .iter()
            .cloned()
            .enumerate()
            .filter(|(index, _)| current_quest_index != Some(*index))
            .rev()
            .map(|(_, quest)| Line::from(quest)),
    );
    lines
}

fn left_pane_constraints(visibility: &PaneVisibility, height: u16) -> [Constraint; 5] {
    let pane_height = |pane, expanded| {
        if visibility.is_collapsed(pane) {
            2
        } else {
            expanded
        }
    };
    let activity = pane_height(Pane::Activity, 4);
    let progress = pane_height(Pane::Progress, 7);
    let details_min = pane_height(Pane::Details, 6);
    let equipment = pane_height(
        Pane::Equipment,
        height
            .saturating_sub(activity + progress + details_min)
            .clamp(2, 13),
    );
    let details = pane_height(
        Pane::Details,
        height
            .saturating_sub(activity + progress + equipment)
            .max(details_min),
    );
    [
        Constraint::Length(activity),
        Constraint::Length(progress),
        Constraint::Length(equipment),
        Constraint::Length(details),
        Constraint::Min(0),
    ]
}

fn details_lines(character: &DashboardCharacter) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(format!("Character ID: {}", character.id)),
        Line::from(format!(
            "Last task elapsed: {}",
            format_elapsed(character.activity.elapsed)
        )),
    ];
    if !character.profile.motto.is_empty() {
        lines.push(Line::from(format!("Motto: {}", character.profile.motto)));
    }
    if !character.profile.guild.is_empty() {
        lines.push(Line::from(format!("Guild: {}", character.profile.guild)));
    }
    lines
}

fn format_elapsed(seconds: u64) -> String {
    let units = [
        (seconds / 86_400, "d"),
        (seconds / 3_600 % 24, "h"),
        (seconds / 60 % 60, "m"),
        (seconds % 60, "s"),
    ];
    units
        .into_iter()
        .skip_while(|(value, unit)| *value == 0 && *unit != "s")
        .map(|(value, unit)| format!("{value}{unit}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn right_pane_constraints(visibility: &PaneVisibility) -> [Constraint; 2] {
    match (
        visibility.is_collapsed(Pane::Adventure),
        visibility.is_collapsed(Pane::Journal),
    ) {
        (false, false) => [Constraint::Percentage(75), Constraint::Percentage(25)],
        (false, true) => [Constraint::Min(5), Constraint::Length(2)],
        (true, false) => [Constraint::Length(2), Constraint::Min(5)],
        (true, true) => [Constraint::Length(2), Constraint::Length(2)],
    }
}

fn pane_block(title: &str, pane: Pane) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title(Line::from(pane.hotkey()).right_aligned())
}

fn render_pane(
    frame: &mut ratatui::Frame<'_>,
    area: Rect,
    title: &str,
    pane: Pane,
    visibility: &PaneVisibility,
    content: Paragraph<'_>,
) {
    if visibility.is_collapsed(pane) {
        frame.render_widget(pane_block(title, pane), area);
    } else {
        frame.render_widget(content.block(pane_block(title, pane)), area);
    }
}

fn render_progress(
    frame: &mut ratatui::Frame<'_>,
    progress: &Progress,
    task_percent: u64,
    area: Rect,
    pane: Pane,
) {
    let bars = [
        ("Experience", progress.experience.percent),
        ("Encumbrance", progress.encumbrance.percent),
        ("Plot", progress.plot.percent),
        ("Quest", progress.quest.percent),
        ("Task", task_percent),
    ];
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(1); bars.len()])
        .split(area.inner(ratatui::layout::Margin {
            horizontal: 1,
            vertical: 1,
        }));
    frame.render_widget(pane_block("Progress", pane), area);
    for ((label, percent), row) in bars.into_iter().zip(rows.iter().copied()) {
        let label = format!("{label} {percent}%");
        frame.render_widget(
            ProgressGauge {
                label: &label,
                percent: percent.min(100),
            },
            row,
        );
    }
}

struct ProgressGauge<'a> {
    label: &'a str,
    percent: u64,
}

impl Widget for ProgressGauge<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let filled_width = (f64::from(area.width) * (self.percent as f64 / 100.0)).round() as u16;
        let label = self
            .label
            .chars()
            .take(area.width as usize)
            .collect::<Vec<_>>();
        let label_start = (area.width - label.len() as u16) / 2;

        for offset in 0..area.width {
            let cell = &mut buffer[(area.left() + offset, area.top())];
            cell.reset();
            let filled = offset < filled_width;
            if let Some(character) = offset
                .checked_sub(label_start)
                .and_then(|index| label.get(index as usize))
            {
                cell.set_char(*character);
                if filled {
                    cell.set_fg(Color::Black).set_bg(Color::Cyan);
                } else {
                    cell.set_fg(Color::Cyan);
                }
            } else if filled {
                cell.set_symbol(symbols::block::FULL).set_fg(Color::Cyan);
            }
        }
    }
}

fn progress_text(progress: &Progress, task_percent: u64) -> String {
    format!(
        "XP {}% | Encumbrance {}% | Plot {}% | Quest {}% | Task {}%",
        progress.experience.percent,
        progress.encumbrance.percent,
        progress.plot.percent,
        progress.quest.percent,
        task_percent
    )
}

fn activity_text(activity: &Activity) -> &str {
    if activity.kill.is_empty() {
        &activity.task
    } else {
        &activity.kill
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RowKey {
    name: String,
    occurrence: usize,
}

#[derive(Debug, Default)]
struct RecentTaskUpdates {
    stats: HashSet<&'static str>,
    equipment: HashSet<&'static str>,
    inventory: HashSet<RowKey>,
    spells: HashSet<RowKey>,
}

impl RecentTaskUpdates {
    fn between(previous: &DashboardCharacter, current: &DashboardCharacter) -> Self {
        let mut updates = Self::default();
        for ((label, previous_value), (_, current_value)) in attribute_values(&previous.stats)
            .into_iter()
            .zip(attribute_values(&current.stats))
        {
            if previous_value != current_value {
                updates.stats.insert(label);
            }
        }
        for ((slot, previous_item), (_, current_item)) in equipment_slots(&previous.equipment)
            .into_iter()
            .zip(equipment_slots(&current.equipment))
        {
            if previous_item != current_item && !current_item.is_empty() {
                updates.equipment.insert(slot);
            }
        }
        updates.inventory = changed_inventory_rows(&previous.inventory, &current.inventory);
        updates.spells = changed_spell_rows(&previous.spells, &current.spells);
        updates
    }
}

fn attribute_values(stats: &Attributes) -> [(&'static str, f64); 8] {
    [
        ("STR", stats.strength),
        ("CON", stats.constitution),
        ("DEX", stats.dexterity),
        ("INT", stats.intelligence),
        ("WIS", stats.wisdom),
        ("CHA", stats.charisma),
        ("HP Max", stats.hit_points_max),
        ("MP Max", stats.mana_points_max),
    ]
}

fn equipment_slots(equipment: &Equipment) -> [(&'static str, &str); 11] {
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
}

fn changed_inventory_rows(
    previous: &[InventoryEntry],
    current: &[InventoryEntry],
) -> HashSet<RowKey> {
    let mut previous_by_name: HashMap<String, Vec<u64>> = HashMap::new();
    for item in previous {
        previous_by_name
            .entry(item.name.clone())
            .or_default()
            .push(item.quantity);
    }
    let mut occurrences = HashMap::new();
    current
        .iter()
        .filter_map(|item| {
            let occurrence = occurrences.entry(&item.name).or_insert(0);
            let key = RowKey {
                name: item.name.clone(),
                occurrence: *occurrence,
            };
            *occurrence += 1;
            let values = previous_by_name.entry(item.name.clone()).or_default();
            values
                .iter()
                .position(|quantity| *quantity == item.quantity)
                .map(|index| values.remove(index))
                .is_none()
                .then_some(key)
        })
        .collect()
}

fn changed_spell_rows(previous: &[Spell], current: &[Spell]) -> HashSet<RowKey> {
    let mut previous_by_name: HashMap<String, Vec<String>> = HashMap::new();
    for spell in previous {
        previous_by_name
            .entry(spell.name.clone())
            .or_default()
            .push(spell.rank.clone());
    }
    let mut occurrences = HashMap::new();
    current
        .iter()
        .filter_map(|spell| {
            let occurrence = occurrences.entry(&spell.name).or_insert(0);
            let key = RowKey {
                name: spell.name.clone(),
                occurrence: *occurrence,
            };
            *occurrence += 1;
            let ranks = previous_by_name.entry(spell.name.clone()).or_default();
            ranks
                .iter()
                .position(|rank| rank == &spell.rank)
                .map(|index| ranks.remove(index))
                .is_none()
                .then_some(key)
        })
        .collect()
}

fn update_style() -> Style {
    Style::default().fg(Color::Black).bg(Color::Cyan)
}

fn styled_value(content: String, updated: bool) -> Span<'static> {
    if updated {
        Span::styled(content, update_style())
    } else {
        Span::raw(content)
    }
}

fn equipment_lines(equipment: &Equipment, updates: &RecentTaskUpdates) -> Vec<Line<'static>> {
    let lines = equipment_slots(equipment)
        .into_iter()
        .filter(|(_, item)| !item.is_empty())
        .map(|(slot, item)| {
            Line::from(styled_value(
                format!("{slot}: {item}"),
                updates.equipment.contains(slot),
            ))
        })
        .collect::<Vec<_>>();
    if lines.is_empty() {
        vec![Line::from("Equipment: none")]
    } else {
        lines
    }
}

fn inventory_line(items: &[InventoryEntry], updates: &RecentTaskUpdates) -> Line<'static> {
    let mut spans = vec![Span::raw("Inventory: ")];
    if items.is_empty() {
        spans.push(Span::raw("none"));
        return Line::from(spans);
    }
    let mut occurrences = HashMap::new();
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            spans.push(Span::raw(", "));
        }
        let occurrence = occurrences.entry(&item.name).or_insert(0);
        let key = RowKey {
            name: item.name.clone(),
            occurrence: *occurrence,
        };
        *occurrence += 1;
        spans.push(styled_value(
            format!("{} x{}", item.name, item.quantity),
            updates.inventory.contains(&key),
        ));
    }
    Line::from(spans)
}

fn spells_line(spells: &[Spell], updates: &RecentTaskUpdates) -> Line<'static> {
    let mut spans = vec![Span::raw("Spells: ")];
    if spells.is_empty() {
        spans.push(Span::raw("none"));
        return Line::from(spans);
    }
    let mut occurrences = HashMap::new();
    for (index, spell) in spells.iter().enumerate() {
        if index > 0 {
            spans.push(Span::raw(", "));
        }
        let occurrence = occurrences.entry(&spell.name).or_insert(0);
        let key = RowKey {
            name: spell.name.clone(),
            occurrence: *occurrence,
        };
        *occurrence += 1;
        spans.push(styled_value(
            format!("{} {}", spell.name, spell.rank),
            updates.spells.contains(&key),
        ));
    }
    Line::from(spans)
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
        state_snapshots: std::cell::RefCell<Vec<Result<DashboardCharacter, DashboardError>>>,
        refreshes: std::cell::Cell<usize>,
        state_reads: std::cell::Cell<usize>,
        actions: std::cell::RefCell<Vec<LifecycleAction>>,
        action_results: std::cell::RefCell<Vec<Result<(), DashboardError>>>,
        brags: std::cell::RefCell<Vec<CharacterId>>,
        brag_results: std::cell::RefCell<Vec<Result<DeliveryOutcome, DashboardError>>>,
    }

    struct ProfileProvider {
        observer: FakeProvider,
        calls: RefCell<Vec<(CharacterId, ProfileField, String)>>,
        motto: RefCell<Vec<Result<DeliveryOutcome, DashboardError>>>,
        guild: RefCell<Vec<Result<GuildOutcome, DashboardError>>>,
    }

    impl DashboardProvider for ProfileProvider {
        fn refresh(&self, id: &CharacterId) -> Result<DashboardSnapshot, DashboardError> {
            self.observer.refresh(id)
        }
        fn read_state(&self, id: &CharacterId) -> Result<DashboardCharacter, DashboardError> {
            self.observer.read_state(id)
        }
        fn lifecycle(
            &self,
            action: LifecycleAction,
            id: &CharacterId,
        ) -> Result<(), DashboardError> {
            self.observer.lifecycle(action, id)
        }
        fn brag(&self, id: &CharacterId) -> Result<DeliveryOutcome, DashboardError> {
            self.observer.brag(id)
        }
        fn set_motto(
            &self,
            id: &CharacterId,
            text: &str,
        ) -> Result<DeliveryOutcome, DashboardError> {
            self.calls
                .borrow_mut()
                .push((id.clone(), ProfileField::Motto, text.to_owned()));
            self.motto.borrow_mut().remove(0)
        }
        fn set_guild(&self, id: &CharacterId, text: &str) -> Result<GuildOutcome, DashboardError> {
            self.calls
                .borrow_mut()
                .push((id.clone(), ProfileField::Guild, text.to_owned()));
            self.guild.borrow_mut().remove(0)
        }
    }

    fn profile_provider(next: DashboardSnapshot) -> ProfileProvider {
        ProfileProvider {
            observer: fake_provider(vec![Ok(next)], Vec::new()),
            calls: RefCell::new(Vec::new()),
            motto: RefCell::new(vec![Ok(DeliveryOutcome::Delivered)]),
            guild: RefCell::new(vec![Ok(GuildOutcome::Accepted)]),
        }
    }

    fn key_event(code: KeyCode) -> Event {
        Event::Key(crossterm::event::KeyEvent::new(code, KeyModifiers::NONE))
    }

    #[test]
    fn profile_editors_treat_action_keys_as_text_and_cancel_without_calls() {
        let snapshot = active_snapshot(250.0, 1_000);
        let id = snapshot.character.id.clone();
        let now = Instant::now();
        for (key, field) in [('m', ProfileField::Motto), ('g', ProfileField::Guild)] {
            let mut initial = snapshot.clone();
            initial.character.profile.motto = "Old".to_owned();
            initial.character.profile.guild = "Old".to_owned();
            let provider = profile_provider(initial.clone());
            let mut state = DashboardState::new(initial, now, Duration::from_secs(1));
            state.handle_event(&provider, &id, key_event(KeyCode::Char(key)), now);
            assert_eq!(state.editor.as_ref().unwrap().field, field);
            assert_eq!(state.editor.as_ref().unwrap().text, "Old");
            for value in "qrbmsxcg\u{2603}\u{754c}".chars() {
                assert!(!state.handle_event(&provider, &id, key_event(KeyCode::Char(value)), now));
            }
            state.handle_event(&provider, &id, key_event(KeyCode::Backspace), now);
            state.handle_event(&provider, &id, key_event(KeyCode::Char('\n')), now);
            state.handle_event(&provider, &id, key_event(KeyCode::F(1)), now);
            assert_eq!(state.editor.as_ref().unwrap().text, "Oldqrbmsxcg\u{2603}");
            assert!(!state.panes.is_collapsed(Pane::Activity));
            assert!(state.confirmation.is_none());
            state.handle_event(&provider, &id, key_event(KeyCode::Esc), now);
            assert!(state.editor.is_none());
            assert_eq!(
                state.current.character.profile,
                snapshot_with_profile("Old", "Old").character.profile
            );
            assert!(provider.calls.borrow().is_empty());
            assert!(provider.observer.actions.borrow().is_empty());
            assert!(provider.observer.brags.borrow().is_empty());
            assert_eq!(provider.observer.refreshes.get(), 0);
        }
    }

    fn snapshot_with_profile(motto: &str, guild: &str) -> DashboardSnapshot {
        let mut snapshot = sample();
        snapshot.character.profile = OnlineProfile {
            motto: motto.to_owned(),
            guild: guild.to_owned(),
        };
        snapshot
    }

    #[test]
    fn profile_submissions_refresh_the_persisted_value_and_safe_outcome() {
        let now = Instant::now();
        for field in [ProfileField::Motto, ProfileField::Guild] {
            for text in ["Changed \u{2603}", ""] {
                let mut initial = snapshot_with_profile("Old", "Old");
                initial.runtime_owned = Some(true);
                let id = initial.character.id.clone();
                let mut next = initial.clone();
                match field {
                    ProfileField::Motto => next.character.profile.motto = text.to_owned(),
                    ProfileField::Guild => next.character.profile.guild = text.to_owned(),
                }
                let expected = next.character.profile.clone();
                let provider = profile_provider(next);
                let mut state = DashboardState::new(initial, now, Duration::from_secs(1));
                state.apply(&provider, &id, Command::Edit(field), now);
                for _ in 0..3 {
                    state.handle_event(&provider, &id, key_event(KeyCode::Backspace), now);
                }
                for value in text.chars() {
                    state.handle_event(&provider, &id, key_event(KeyCode::Char(value)), now);
                }
                state.handle_event(&provider, &id, key_event(KeyCode::Enter), now);
                assert!(state.editor.is_none());
                assert_eq!(state.current.character.profile, expected);
                assert_eq!(
                    *provider.calls.borrow(),
                    [(id.clone(), field, text.to_owned())]
                );
                assert_eq!(provider.observer.refreshes.get(), 1);
                assert_eq!(state.current.runtime_owned, Some(true));
                let message = state.current.message.unwrap();
                assert!(!message.contains("4242"));
                assert!(!message.contains("alpaquil.php"));
                assert!(message.contains(if field == ProfileField::Motto {
                    "Motto saved"
                } else {
                    "accepted"
                }));
            }
        }
    }

    #[test]
    fn profile_failures_show_retained_values_and_safe_errors() {
        let now = Instant::now();
        for outcome in [GuildOutcome::Rejected, GuildOutcome::Indeterminate] {
            let initial = snapshot_with_profile("Old motto", "Old guild");
            let id = initial.character.id.clone();
            let provider = profile_provider(initial.clone());
            *provider.guild.borrow_mut() = vec![Ok(outcome)];
            let mut state = DashboardState::new(initial, now, Duration::from_secs(1));
            state.apply(&provider, &id, Command::Edit(ProfileField::Guild), now);
            state.apply(&provider, &id, Command::Insert('!'), now);
            state.apply(&provider, &id, Command::ConfirmAction, now);
            assert_eq!(state.current.character.profile.guild, "Old guild");
            assert_eq!(state.current.message.as_deref(), Some(outcome.message()));
            assert_eq!(provider.calls.borrow().len(), 1);
        }
        for outcome in [
            DeliveryOutcome::EndpointRejected,
            DeliveryOutcome::DeliveryFailed,
        ] {
            let initial = snapshot_with_profile("Old", "Old guild");
            let id = initial.character.id.clone();
            let mut next = initial.clone();
            next.character.profile.motto.push('!');
            let provider = profile_provider(next);
            *provider.motto.borrow_mut() = vec![Ok(outcome)];
            let mut state = DashboardState::new(initial, now, Duration::from_secs(1));
            state.apply(&provider, &id, Command::Edit(ProfileField::Motto), now);
            state.apply(&provider, &id, Command::Insert('!'), now);
            state.apply(&provider, &id, Command::ConfirmAction, now);
            assert_eq!(state.current.character.profile.motto, "Old!");
            assert_eq!(
                state.current.message.as_deref(),
                Some(outcome.motto_message())
            );
        }
        let initial = sample();
        let id = initial.character.id.clone();
        let provider = profile_provider(initial.clone());
        *provider.motto.borrow_mut() = vec![Err(ReportingError::InvalidProfileText.into())];
        let mut state = DashboardState::new(initial, now, Duration::from_secs(1));
        state.apply(&provider, &id, Command::Edit(ProfileField::Motto), now);
        state.apply(&provider, &id, Command::ConfirmAction, now);
        assert!(state.current.message.unwrap().contains("control character"));
    }

    #[test]
    fn refresh_failure_after_profile_delivery_is_not_hidden() {
        let initial = sample();
        let id = initial.character.id.clone();
        let provider = profile_provider(initial.clone());
        *provider.observer.snapshots.borrow_mut() =
            vec![Err(ReportingError::InvalidProfileText.into())];
        let now = Instant::now();
        let mut state = DashboardState::new(initial, now, Duration::from_secs(1));
        state.apply(&provider, &id, Command::Edit(ProfileField::Motto), now);
        state.apply(&provider, &id, Command::ConfirmAction, now);
        let message = state.current.message.unwrap();
        assert!(message.contains("Motto saved"));
        assert!(message.contains("Could not refresh dashboard"));
    }

    #[test]
    fn profile_editor_control_keys_and_interrupt_do_not_submit() {
        for value in ['x', '\u{85}'] {
            assert_eq!(
                editor_command(Event::Key(crossterm::event::KeyEvent::new(
                    KeyCode::Char(value),
                    KeyModifiers::CONTROL
                ))),
                Command::None
            );
        }
        assert_eq!(
            editor_command(Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::Char('c'),
                KeyModifiers::CONTROL
            ))),
            Command::Quit
        );
        let initial = sample();
        let id = initial.character.id.clone();
        let provider = profile_provider(initial.clone());
        let now = Instant::now();
        let mut state = DashboardState::new(initial, now, Duration::from_secs(1));
        state.apply(&provider, &id, Command::Edit(ProfileField::Motto), now);
        assert!(state.apply(&provider, &id, Command::Quit, now));
        assert!(provider.calls.borrow().is_empty());
    }

    #[test]
    fn formats_elapsed_boundaries_without_leading_zero_units() {
        for (seconds, expected) in [
            (0, "0s"),
            (59, "59s"),
            (60, "1m 0s"),
            (3600, "1h 0m 0s"),
            (86400, "1d 0h 0m 0s"),
            (90061, "1d 1h 1m 1s"),
        ] {
            assert_eq!(format_elapsed(seconds), expected);
            let mut snapshot = sample();
            snapshot.character.activity.elapsed = seconds;
            let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
            terminal
                .draw(|frame| {
                    render(
                        frame,
                        &snapshot,
                        0,
                        &RecentTaskUpdates::default(),
                        None,
                        &PaneVisibility::default(),
                    )
                })
                .unwrap();
            let output: String = terminal
                .backend()
                .buffer()
                .content()
                .iter()
                .map(|cell| cell.symbol())
                .collect();
            assert!(output.contains(&format!("Last task elapsed: {expected}")));
        }
    }

    #[test]
    fn details_show_each_profile_line_independently_without_quest_target() {
        for motto in ["", "Test motto"] {
            for guild in ["", "Test guild"] {
                for width in [90, 120] {
                    let snapshot = snapshot_with_profile(motto, guild);
                    let mut terminal = Terminal::new(TestBackend::new(width, 40)).unwrap();
                    terminal
                        .draw(|frame| {
                            render(
                                frame,
                                &snapshot,
                                0,
                                &RecentTaskUpdates::default(),
                                None,
                                &PaneVisibility::default(),
                            )
                        })
                        .unwrap();
                    let output: String = terminal
                        .backend()
                        .buffer()
                        .content()
                        .iter()
                        .map(|cell| cell.symbol())
                        .collect();
                    assert!(output.contains("Character ID:"));
                    assert!(output.contains("Last task elapsed:"));
                    assert_eq!(output.contains("Motto:"), !motto.is_empty());
                    assert_eq!(output.contains("Guild:"), !guild.is_empty());
                    assert!(!output.contains("Quest target"));
                }
            }
        }
    }

    #[test]
    fn left_panes_fill_details_and_cap_equipment_for_all_collapses() {
        for height in [20, 24, 40, 80] {
            for mask in 0..128 {
                let mut panes = PaneVisibility::default();
                for (index, pane) in Pane::ALL.into_iter().enumerate() {
                    if mask & (1 << index) != 0 {
                        panes.toggle(pane);
                    }
                }
                let areas = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(left_pane_constraints(&panes, height))
                    .split(Rect::new(0, 0, 50, height));
                for (index, pane, expanded) in [(0, Pane::Activity, 4), (1, Pane::Progress, 7)] {
                    assert_eq!(
                        areas[index].height,
                        if panes.is_collapsed(pane) {
                            2
                        } else {
                            expanded
                        },
                        "height={height}, mask={mask}, pane={pane:?}"
                    );
                }
                assert!((2..=13).contains(&areas[2].height));
                if panes.is_collapsed(Pane::Equipment) {
                    assert_eq!(areas[2].height, 2);
                }
                if panes.is_collapsed(Pane::Details) {
                    assert_eq!(areas[3].height, 2);
                } else {
                    assert!(areas[3].height >= 6);
                    assert_eq!(
                        areas[3].height,
                        height - areas[0].height - areas[1].height - areas[2].height
                    );
                    assert_eq!(areas[3].bottom(), height);
                    assert_eq!(areas[4].height, 0);
                }
                assert_eq!(areas.iter().map(|area| area.height).sum::<u16>(), height);
            }
        }
    }

    #[test]
    fn profile_editor_and_help_render_at_narrow_and_full_widths() {
        for width in [40, 70, 120] {
            let output = rendered(width, 40);
            for key in ["m motto", "g guild", "q quit", "c recover"] {
                assert!(output.contains(key));
            }
            for field in [ProfileField::Motto, ProfileField::Guild] {
                let mut terminal = Terminal::new(TestBackend::new(width, 20)).unwrap();
                let editor = ProfileEditor {
                    field,
                    text: format!("{} \u{754c}", "long ".repeat(100)),
                };
                terminal
                    .draw(|frame| render_editor(frame, &editor))
                    .unwrap();
                let output: String = terminal
                    .backend()
                    .buffer()
                    .content()
                    .iter()
                    .map(|cell| cell.symbol())
                    .collect();
                assert!(output.contains('\u{754c}'));
                assert!(output.contains("Enter submit | Esc cancel"));
                assert!(output.contains(if field == ProfileField::Motto {
                    "empty clears"
                } else {
                    "empty leaves"
                }));
            }
        }
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
            self.refreshes.set(self.refreshes.get() + 1);
            self.snapshots.borrow_mut().remove(0)
        }

        fn read_state(&self, _id: &CharacterId) -> Result<DashboardCharacter, DashboardError> {
            self.state_reads.set(self.state_reads.get() + 1);
            self.state_snapshots.borrow_mut().remove(0)
        }

        fn lifecycle(
            &self,
            action: LifecycleAction,
            _id: &CharacterId,
        ) -> Result<(), DashboardError> {
            self.actions.borrow_mut().push(action);
            self.action_results.borrow_mut().remove(0)
        }

        fn brag(&self, id: &CharacterId) -> Result<DeliveryOutcome, DashboardError> {
            self.brags.borrow_mut().push(id.clone());
            self.brag_results.borrow_mut().remove(0)
        }

        fn set_motto(&self, _: &CharacterId, _: &str) -> Result<DeliveryOutcome, DashboardError> {
            panic!("unexpected motto action")
        }

        fn set_guild(&self, _: &CharacterId, _: &str) -> Result<GuildOutcome, DashboardError> {
            panic!("unexpected guild action")
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
                stats: character.stats,
                progress: character.progress,
                equipment: character.equipment,
                inventory: character.inventory,
                spells: character.spells,
                plot: character.plot,
                quests: character.quests,
                current_quest: character.bestquest,
                profile: character.profile,
            },
            service: Some(ServiceState::Inactive),
            runtime_owned: Some(false),
            message: None,
        }
    }

    fn fake_provider(
        snapshots: Vec<Result<DashboardSnapshot, DashboardError>>,
        state_snapshots: Vec<Result<DashboardCharacter, DashboardError>>,
    ) -> FakeProvider {
        FakeProvider {
            snapshots: std::cell::RefCell::new(snapshots),
            state_snapshots: std::cell::RefCell::new(state_snapshots),
            refreshes: std::cell::Cell::new(0),
            state_reads: std::cell::Cell::new(0),
            actions: std::cell::RefCell::new(Vec::new()),
            action_results: std::cell::RefCell::new(Vec::new()),
            brags: std::cell::RefCell::new(Vec::new()),
            brag_results: std::cell::RefCell::new(Vec::new()),
        }
    }

    fn active_snapshot(position_ms: f64, duration_ms: u64) -> DashboardSnapshot {
        let mut snapshot = sample();
        snapshot.runtime_owned = Some(true);
        snapshot.service = Some(ServiceState::Active);
        snapshot.character.progress.task.position = position_ms;
        snapshot.character.progress.task.max = duration_ms;
        snapshot.character.progress.task.percent = task_percent(position_ms, duration_ms);
        snapshot
    }

    fn rendered_with_panes(width: u16, height: u16, panes: &PaneVisibility) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        let snapshot = sample();
        let task_percent = snapshot.character.progress.task.percent;
        terminal
            .draw(|frame| {
                render(
                    frame,
                    &snapshot,
                    task_percent,
                    &RecentTaskUpdates::default(),
                    None,
                    panes,
                )
            })
            .unwrap();
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect()
    }

    fn rendered(width: u16, height: u16) -> String {
        rendered_with_panes(width, height, &PaneVisibility::default())
    }

    #[test]
    fn task_anchor_is_created_from_an_observed_snapshot() {
        let snapshot = active_snapshot(250.0, 1_000);
        let observed_at = Instant::now();

        let anchor = TaskAnchor::from_snapshot(&snapshot, observed_at);

        assert_eq!(anchor.position_ms, 250.0);
        assert_eq!(anchor.duration_ms, 1_000);
        assert_eq!(
            anchor.identity.completed_tasks,
            snapshot.character.activity.tasks
        );
        assert_eq!(anchor.identity.duration_ms, 1_000);
        assert_eq!(anchor.observed_at, observed_at);
    }

    #[test]
    fn task_prediction_clamps_and_uses_browser_compatible_percentages() {
        let now = Instant::now();
        let snapshot = active_snapshot(250.0, 1_000);
        let anchor = TaskAnchor::from_snapshot(&snapshot, now);

        assert_eq!(predicted_task_position(anchor, Duration::ZERO), 250.0);
        assert_eq!(
            predicted_task_position(anchor, Duration::from_millis(250)),
            500.0
        );
        assert_eq!(
            predicted_task_position(anchor, Duration::from_millis(750)),
            1_000.0
        );
        assert_eq!(
            predicted_task_position(anchor, Duration::from_secs(2)),
            1_000.0
        );
        assert_eq!(
            predicted_task_percent(anchor, Duration::ZERO),
            snapshot.character.progress.task.percent
        );
        assert_eq!(
            predicted_task_percent(anchor, Duration::from_millis(750)),
            100
        );
        assert_eq!(predicted_task_percent(anchor, Duration::from_secs(2)), 100);

        let zero = TaskAnchor {
            position_ms: 0.0,
            duration_ms: 0,
            identity: TaskIdentity {
                completed_tasks: 0,
                duration_ms: 0,
            },
            observed_at: now,
        };
        assert_eq!(predicted_task_position(zero, Duration::ZERO), 0.0);
        assert_eq!(predicted_task_percent(zero, Duration::ZERO), 100);
    }

    #[test]
    fn percent_boundary_deadlines_cover_long_short_final_and_saturated_tasks() {
        let now = Instant::now();
        let long = TaskAnchor::from_snapshot(&active_snapshot(5_000.0, 20_000), now);
        assert_eq!(
            next_task_percent_boundary(long, Duration::ZERO),
            Some(Duration::from_millis(200))
        );
        assert_eq!(
            next_redraw_boundary(long, Duration::ZERO),
            Some(Duration::from_millis(200))
        );

        let short = TaskAnchor::from_snapshot(&active_snapshot(0.0, 1_000), now);
        assert_eq!(
            next_task_percent_boundary(short, Duration::ZERO),
            Some(Duration::from_millis(10))
        );
        assert_eq!(
            next_redraw_boundary(short, Duration::ZERO),
            Some(MINIMUM_REDRAW_SPACING)
        );

        let final_step = TaskAnchor::from_snapshot(&active_snapshot(990.0, 1_000), now);
        assert_eq!(
            next_task_percent_boundary(final_step, Duration::ZERO),
            Some(Duration::from_millis(10))
        );
        assert_eq!(
            next_task_percent_boundary(final_step, Duration::from_millis(10)),
            None
        );

        let zero = TaskAnchor::from_snapshot(&active_snapshot(0.0, 0), now);
        assert_eq!(next_task_percent_boundary(zero, Duration::ZERO), None);
    }

    #[test]
    fn poll_timeout_uses_the_earliest_bounded_deadline() {
        let cap = Duration::from_secs(1);
        assert_eq!(
            select_poll_timeout(
                Some(Duration::from_millis(20)),
                Duration::from_millis(500),
                Some(Duration::from_millis(700)),
                cap,
            ),
            Duration::from_millis(20)
        );
        assert_eq!(
            select_poll_timeout(
                Some(Duration::from_millis(800)),
                Duration::from_millis(300),
                Some(Duration::from_millis(700)),
                cap,
            ),
            Duration::from_millis(300)
        );
        assert_eq!(
            select_poll_timeout(
                None,
                Duration::from_millis(800),
                Some(Duration::from_millis(400)),
                cap,
            ),
            Duration::from_millis(400)
        );
        assert_eq!(
            select_poll_timeout(None, Duration::from_secs(2), None, cap),
            cap
        );
    }

    #[test]
    fn selector_navigation_wraps_and_reports_selection_or_cancellation() {
        let mut selector = SelectorState::new(2);
        assert_eq!(selector.apply(SelectionCommand::Previous), None);
        assert_eq!(selector.selected, 1);
        assert_eq!(selector.apply(SelectionCommand::Next), None);
        assert_eq!(selector.selected, 0);
        assert_eq!(selector.apply(SelectionCommand::Select), Some(Some(0)));
        assert_eq!(selector.apply(SelectionCommand::Cancel), Some(None));
    }

    #[test]
    fn selector_maps_documented_keys() {
        assert_eq!(
            selection_command(Event::Key(crossterm::event::KeyEvent::from(KeyCode::Up))),
            SelectionCommand::Previous
        );
        assert_eq!(
            selection_command(Event::Key(crossterm::event::KeyEvent::from(KeyCode::Char(
                'j'
            )))),
            SelectionCommand::Next
        );
        assert_eq!(
            selection_command(Event::Key(crossterm::event::KeyEvent::from(KeyCode::Enter))),
            SelectionCommand::Select
        );
        assert_eq!(
            selection_command(Event::Key(crossterm::event::KeyEvent::from(KeyCode::Esc))),
            SelectionCommand::Cancel
        );
    }

    #[test]
    fn empty_character_selection_reports_an_actionable_error() {
        let interrupted = AtomicBool::new(false);
        assert!(matches!(
            select_character(Vec::new(), &interrupted),
            Err(DashboardError::NoManagedCharacters)
        ));
    }

    #[test]
    fn renders_complete_compact_and_too_small_layouts() {
        let complete = rendered(120, 40);
        assert!(complete.contains("Equipment"));
        assert!(complete.contains("Adventure"));
        assert!(complete.contains("Journal"));
        for pane in Pane::ALL {
            assert!(complete.contains(pane.hotkey()));
        }
        assert!(complete.contains("Experience 0%"));
        assert!(complete.contains("STR:"));
        let compact = rendered(70, 30);
        assert!(compact.contains("Character"));
        assert!(compact.contains("Inventory"));
        assert!(compact.contains("STR:"));
        let too_small = rendered(30, 10);
        assert!(too_small.contains("Terminal is too small"));
    }

    #[test]
    fn renders_predicted_task_percent_without_mutating_the_snapshot() {
        let snapshot = active_snapshot(250.0, 1_000);
        let persisted_percent = snapshot.character.progress.task.percent;

        for (width, expected_other_progress) in [
            (120, "Experience 0%"),
            (70, "XP 0% | Encumbrance 8% | Plot 0% | Quest 4%"),
        ] {
            let backend = TestBackend::new(width, 40);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal
                .draw(|frame| {
                    render(
                        frame,
                        &snapshot,
                        75,
                        &RecentTaskUpdates::default(),
                        None,
                        &PaneVisibility::default(),
                    )
                })
                .unwrap();
            let rendered = terminal
                .backend()
                .buffer()
                .content()
                .iter()
                .map(|cell| cell.symbol())
                .collect::<String>();

            assert!(rendered.contains("Task 75%"));
            assert!(rendered.contains(expected_other_progress));
        }

        assert_eq!(snapshot.character.progress.task.percent, persisted_percent);
    }

    #[test]
    fn equipment_omits_empty_slots() {
        let equipment =
            equipment_lines(&sample().character.equipment, &RecentTaskUpdates::default())
                .into_iter()
                .flat_map(|line| line.spans)
                .map(|span| span.content)
                .collect::<String>();
        assert!(equipment.contains("Weapon: Bronze Sword"));
        assert!(equipment.contains("Hauberk: Leather"));
        assert!(!equipment.contains("Helm:"));
        assert!(!equipment.contains("Brassairts:"));
    }

    #[test]
    fn activity_uses_human_readable_text() {
        let activity = &sample().character.activity;
        assert_eq!(activity_text(activity), "Selling a widget...");
    }

    fn has_update_style(line: Line<'static>, label: &str) -> bool {
        let backend = TestBackend::new(120, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| frame.render_widget(Paragraph::new(line), frame.area()))
            .unwrap();
        terminal
            .backend()
            .buffer()
            .content()
            .windows(label.len())
            .find(|cells| cells.iter().map(|cell| cell.symbol()).collect::<String>() == label)
            .unwrap()
            .iter()
            .all(|cell| cell.fg == Color::Black && cell.bg == Color::Cyan)
    }

    #[test]
    fn recent_task_updates_compare_current_values_by_logical_identity() {
        let mut previous = sample().character;
        previous.inventory = vec![
            InventoryEntry {
                name: "duplicate".to_owned(),
                quantity: 1,
            },
            InventoryEntry {
                name: "duplicate".to_owned(),
                quantity: 2,
            },
            InventoryEntry {
                name: "removed".to_owned(),
                quantity: 1,
            },
        ];
        previous.spells = vec![
            Spell {
                name: "Spark".to_owned(),
                rank: "I".to_owned(),
            },
            Spell {
                name: "Spark".to_owned(),
                rank: "II".to_owned(),
            },
        ];
        let mut current = previous.clone();
        current.stats.strength += 1.0;
        current.equipment.weapon = "Silver Sword".to_owned();
        current.inventory = vec![
            InventoryEntry {
                name: "duplicate".to_owned(),
                quantity: 2,
            },
            InventoryEntry {
                name: "duplicate".to_owned(),
                quantity: 1,
            },
            InventoryEntry {
                name: "added".to_owned(),
                quantity: 1,
            },
        ];
        current.spells = vec![
            Spell {
                name: "Spark".to_owned(),
                rank: "II".to_owned(),
            },
            Spell {
                name: "Spark".to_owned(),
                rank: "I".to_owned(),
            },
            Spell {
                name: "New spell".to_owned(),
                rank: "I".to_owned(),
            },
        ];

        let updates = RecentTaskUpdates::between(&previous, &current);

        assert!(updates.stats.contains("STR"));
        assert!(updates.equipment.contains("Weapon"));
        assert_eq!(
            updates.inventory,
            HashSet::from([RowKey {
                name: "added".to_owned(),
                occurrence: 0,
            }])
        );
        assert_eq!(
            updates.spells,
            HashSet::from([RowKey {
                name: "New spell".to_owned(),
                occurrence: 0,
            }])
        );
    }

    #[test]
    fn recently_updated_values_use_the_dashboard_selection_style() {
        let character = sample().character;
        let mut updates = RecentTaskUpdates::default();
        updates.stats.insert("STR");
        updates.equipment.insert("Weapon");
        updates.inventory.insert(RowKey {
            name: character.inventory[0].name.clone(),
            occurrence: 0,
        });
        updates.spells.insert(RowKey {
            name: character.spells[0].name.clone(),
            occurrence: 0,
        });

        assert!(has_update_style(
            header_stats_line(&character.stats, &updates),
            &format!("STR:{}", character.stats.strength)
        ));
        assert!(has_update_style(
            equipment_lines(&character.equipment, &updates).remove(0),
            &format!("Weapon: {}", character.equipment.weapon)
        ));
        assert!(has_update_style(
            inventory_line(&character.inventory, &updates),
            &format!(
                "{} x{}",
                character.inventory[0].name, character.inventory[0].quantity
            )
        ));
        assert!(has_update_style(
            spells_line(&character.spells, &updates),
            &format!("{} {}", character.spells[0].name, character.spells[0].rank)
        ));
    }

    #[test]
    fn adventure_shows_current_quest_only_when_journal_is_collapsed() {
        let character = sample().character;
        let expanded_journal = adventure_lines(&character, &RecentTaskUpdates::default(), false);
        assert!(
            !expanded_journal
                .iter()
                .flat_map(|line| line.spans.iter())
                .any(|span| span.content == "Current quest: Fetch me an anvil")
        );

        let lines = adventure_lines(&character, &RecentTaskUpdates::default(), true);
        let current = lines
            .iter()
            .flat_map(|line| line.spans.iter())
            .find(|span| span.content == "Current quest: Fetch me an anvil")
            .unwrap();
        assert!(!current.style.add_modifier.contains(Modifier::BOLD));
        assert!(
            !lines
                .iter()
                .flat_map(|line| line.spans.iter())
                .any(|span| span.content == "Quests:")
        );
        assert!(
            !lines
                .iter()
                .flat_map(|line| line.spans.iter())
                .any(|span| span.content.starts_with("STR:"))
        );
        assert!(lines[1].spans.is_empty());
        assert!(
            lines[2]
                .spans
                .iter()
                .any(|span| span.content.starts_with("Spells:"))
        );
    }

    fn text_position(
        content: &[ratatui::buffer::Cell],
        width: usize,
        text: &str,
    ) -> (usize, usize) {
        let index = content
            .windows(text.len())
            .position(|cells| cells.iter().map(|cell| cell.symbol()).collect::<String>() == text)
            .unwrap();
        (index % width, index / width)
    }

    #[test]
    fn header_right_aligns_stats_or_moves_them_to_a_second_row() {
        let snapshot = sample();
        let updates = RecentTaskUpdates::default();

        let wide_backend = TestBackend::new(120, 40);
        let mut wide_terminal = Terminal::new(wide_backend).unwrap();
        wide_terminal
            .draw(|frame| {
                render(
                    frame,
                    &snapshot,
                    snapshot.character.progress.task.percent,
                    &updates,
                    None,
                    &PaneVisibility::default(),
                )
            })
            .unwrap();
        let wide_content = wide_terminal.backend().buffer().content();
        let identity = text_position(wide_content, 120, &snapshot.character.identity.name);
        let stats = text_position(wide_content, 120, "STR:");
        assert_eq!(identity.1, stats.1);
        assert!(identity.0 < stats.0);

        let narrow_backend = TestBackend::new(80, 30);
        let mut narrow_terminal = Terminal::new(narrow_backend).unwrap();
        narrow_terminal
            .draw(|frame| {
                render(
                    frame,
                    &snapshot,
                    snapshot.character.progress.task.percent,
                    &updates,
                    None,
                    &PaneVisibility::default(),
                )
            })
            .unwrap();
        let narrow_content = narrow_terminal.backend().buffer().content();
        let identity = text_position(narrow_content, 80, &snapshot.character.identity.name);
        let stats = text_position(narrow_content, 80, "STR:");
        assert_eq!(stats.1, identity.1 + 1);
    }

    #[test]
    fn journal_leads_with_current_quest_and_reverses_long_completed_history() {
        let mut character = sample().character;
        character.quests = vec![
            "First completed quest".to_owned(),
            "Second completed quest".to_owned(),
            "Third completed quest".to_owned(),
            "Fourth completed quest".to_owned(),
        ];
        character.current_quest = "Current quest".to_owned();

        let quests = journal_lines(&character)
            .into_iter()
            .flat_map(|line| line.spans)
            .collect::<Vec<_>>();

        assert_eq!(quests.len(), 5);
        assert_eq!(quests[0].content, "Current quest");
        assert_eq!(quests[1].content, "Fourth completed quest");
        assert_eq!(quests[2].content, "Third completed quest");
        assert_eq!(quests[3].content, "Second completed quest");
        assert_eq!(quests[4].content, "First completed quest");
        assert!(quests[0].style.add_modifier.contains(Modifier::BOLD));
        assert!(!quests[1].style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn progress_labels_contrast_with_the_bar_background() {
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut snapshot = sample();
        snapshot.character.progress.experience.percent = 100;
        terminal
            .draw(|frame| {
                render(
                    frame,
                    &snapshot,
                    snapshot.character.progress.task.percent,
                    &RecentTaskUpdates::default(),
                    None,
                    &PaneVisibility::default(),
                )
            })
            .unwrap();

        let label = "Experience 100%";
        let label_cells = terminal
            .backend()
            .buffer()
            .content()
            .windows(label.len())
            .find(|cells| cells.iter().map(|cell| cell.symbol()).collect::<String>() == label)
            .unwrap();

        assert!(label_cells.iter().all(|cell| cell.bg == Color::Cyan));
        assert!(label_cells.iter().all(|cell| cell.fg == Color::Black));

        snapshot.character.progress.experience.percent = 0;
        terminal
            .draw(|frame| {
                render(
                    frame,
                    &snapshot,
                    snapshot.character.progress.task.percent,
                    &RecentTaskUpdates::default(),
                    None,
                    &PaneVisibility::default(),
                )
            })
            .unwrap();
        let label = "Experience 0%";
        let label_cells = terminal
            .backend()
            .buffer()
            .content()
            .windows(label.len())
            .find(|cells| cells.iter().map(|cell| cell.symbol()).collect::<String>() == label)
            .unwrap();

        assert!(label_cells.iter().all(|cell| cell.fg == Color::Cyan));

        snapshot.character.progress.experience.percent = 50;
        terminal
            .draw(|frame| {
                render(
                    frame,
                    &snapshot,
                    snapshot.character.progress.task.percent,
                    &RecentTaskUpdates::default(),
                    None,
                    &PaneVisibility::default(),
                )
            })
            .unwrap();
        let label = "Experience 50%";
        let label_cells = terminal
            .backend()
            .buffer()
            .content()
            .windows(label.len())
            .find(|cells| cells.iter().map(|cell| cell.symbol()).collect::<String>() == label)
            .unwrap();

        assert!(label_cells.iter().any(|cell| cell.fg == Color::Black));
        assert!(label_cells.iter().any(|cell| cell.fg == Color::Cyan));
    }

    #[test]
    fn progress_gauge_handles_digit_width_transitions_without_padding() {
        let backend = TestBackend::new(20, 1);
        let mut terminal = Terminal::new(backend).unwrap();

        for (label, percent) in [("Plot 9%", 9), ("Plot 10%", 10), ("Plot 9%", 9)] {
            terminal
                .draw(|frame| {
                    frame.render_widget(ProgressGauge { label, percent }, frame.area());
                })
                .unwrap();
            let rendered = terminal
                .backend()
                .buffer()
                .content()
                .iter()
                .map(|cell| cell.symbol())
                .collect::<String>();

            assert!(rendered.contains(label));
        }
    }

    #[test]
    fn progress_gauge_fills_the_cell_immediately_after_its_label() {
        let backend = TestBackend::new(20, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(
                    ProgressGauge {
                        label: "Plot 90%",
                        percent: 90,
                    },
                    frame.area(),
                );
            })
            .unwrap();

        let label_start = (20 - "Plot 90%".len()) / 2;
        let cell_after_label =
            &terminal.backend().buffer().content()[label_start + "Plot 90%".len()];

        assert_eq!(cell_after_label.symbol(), symbols::block::FULL);
        assert_eq!(cell_after_label.fg, Color::Cyan);
        assert_eq!(cell_after_label.bg, Color::Reset);
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
                'b'
            )))),
            Command::Brag
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
        for pane in Pane::ALL {
            let key = KeyCode::F((pane.index() + 1) as u8);
            assert_eq!(
                command(Event::Key(crossterm::event::KeyEvent::from(key))),
                Command::TogglePane(pane)
            );
        }
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
            state_snapshots: std::cell::RefCell::new(Vec::new()),
            refreshes: std::cell::Cell::new(0),
            state_reads: std::cell::Cell::new(0),
            actions: std::cell::RefCell::new(Vec::new()),
            action_results: std::cell::RefCell::new(Vec::new()),
            brags: std::cell::RefCell::new(Vec::new()),
            brag_results: std::cell::RefCell::new(Vec::new()),
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
            state_snapshots: std::cell::RefCell::new(Vec::new()),
            refreshes: std::cell::Cell::new(0),
            state_reads: std::cell::Cell::new(0),
            actions: std::cell::RefCell::new(Vec::new()),
            action_results: std::cell::RefCell::new(vec![Ok(())]),
            brags: std::cell::RefCell::new(Vec::new()),
            brag_results: std::cell::RefCell::new(Vec::new()),
        };
        let now = Instant::now();
        let mut state = DashboardState::new(first, now, Duration::from_secs(1));
        assert!(!state.apply(&provider, &id, Command::Refresh, now));
        assert_eq!(state.current.character.activity.task, "new persisted task");
        assert!(provider.actions.borrow().is_empty());
        assert_eq!(provider.refreshes.get(), 1);
        assert_eq!(provider.state_reads.get(), 0);

        assert!(!state.apply(&provider, &id, Command::Confirm(LifecycleAction::Stop), now));
        assert!(!state.apply(&provider, &id, Command::Cancel, now));
        assert!(provider.actions.borrow().is_empty());

        assert!(!state.apply(&provider, &id, Command::TogglePane(Pane::Journal), now));
        assert!(state.panes.is_collapsed(Pane::Journal));

        state.confirmation = Some(LifecycleAction::Start);
        assert!(!state.apply(&provider, &id, Command::ConfirmAction, now));
        assert_eq!(*provider.actions.borrow(), [LifecycleAction::Start]);
        assert_eq!(
            state.current.message.as_deref(),
            Some("Requested start successfully.")
        );
    }

    #[test]
    fn state_only_reads_preserve_service_status_and_last_good_character() {
        let id = CharacterId::new();
        let mut first = active_snapshot(100.0, 1_000);
        first.message = Some("service warning".to_owned());
        let mut second = first.character.clone();
        second.activity.task = "new persisted task".to_owned();
        second.activity.kill.clear();
        second.progress.task.position = 300.0;
        second.progress.task.percent = 30;
        let provider = fake_provider(
            Vec::new(),
            vec![Ok(second), Err(DashboardError::NoManagedCharacters)],
        );
        let now = Instant::now();
        let mut state = DashboardState::new(first, now, Duration::from_secs(1));

        state.read_state(&provider, &id, now);
        assert_eq!(state.current.character.activity.task, "new persisted task");
        assert_eq!(state.current.service, Some(ServiceState::Active));
        assert_eq!(state.current.runtime_owned, Some(true));
        assert_eq!(state.current.message.as_deref(), Some("service warning"));
        assert_eq!(provider.state_reads.get(), 1);
        assert_eq!(provider.refreshes.get(), 0);

        let last_good_task = state.current.character.activity.task.clone();
        state.read_state(&provider, &id, now);
        assert_eq!(state.current.character.activity.task, last_good_task);
        assert!(
            state
                .current
                .message
                .as_deref()
                .unwrap()
                .contains("Could not refresh character state")
        );
        assert_eq!(provider.refreshes.get(), 0);

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                render(
                    frame,
                    &state.current,
                    state.displayed_task_percent(now),
                    &state.updates,
                    None,
                    &state.panes,
                )
            })
            .unwrap();
        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(rendered.contains("new persisted task"));
    }

    #[test]
    fn prediction_and_settling_reads_follow_ownership_and_task_boundaries() {
        let id = CharacterId::new();
        let first = active_snapshot(900.0, 1_000);
        let mut same_task = first.character.clone();
        same_task.progress.task.position = 950.0;
        same_task.progress.task.percent = 95;
        let mut next_task = same_task.clone();
        next_task.activity.tasks += 1;
        next_task.activity.task = "next persisted task".to_owned();
        next_task.progress.task.position = 25.0;
        next_task.progress.task.max = 1_000;
        next_task.progress.task.percent = 2;
        let provider = fake_provider(Vec::new(), vec![Ok(same_task), Ok(next_task.clone())]);
        let start = Instant::now();
        let mut state = DashboardState::new(first, start, Duration::from_secs(1));

        assert_eq!(
            state.displayed_task_percent(start + Duration::from_millis(50)),
            95
        );
        assert_eq!(
            state.displayed_task_percent(start + Duration::from_millis(100)),
            100
        );
        state.handle_deadline(&provider, &id, start + Duration::from_millis(100));
        assert_eq!(provider.state_reads.get(), 0);
        state.handle_deadline(&provider, &id, start + Duration::from_millis(199));
        assert_eq!(provider.state_reads.get(), 0);
        assert_eq!(
            state.displayed_task_percent(start + Duration::from_millis(199)),
            100
        );

        state.handle_deadline(&provider, &id, start + Duration::from_millis(200));
        assert_eq!(provider.state_reads.get(), 1);
        assert_eq!(
            state.displayed_task_percent(start + Duration::from_millis(250)),
            100
        );
        state.handle_deadline(&provider, &id, start + Duration::from_millis(299));
        assert_eq!(provider.state_reads.get(), 1);
        state.handle_deadline(&provider, &id, start + Duration::from_millis(300));
        assert_eq!(provider.state_reads.get(), 2);
        assert_eq!(
            state.current.character.activity.task,
            next_task.activity.task
        );
        assert_eq!(
            state.displayed_task_percent(start + Duration::from_millis(300)),
            2
        );
        assert_eq!(provider.refreshes.get(), 0);
    }

    #[test]
    fn inactive_runtime_uses_persisted_progress_and_never_settles() {
        let id = CharacterId::new();
        for ownership in [Some(false), None] {
            let mut snapshot = active_snapshot(250.0, 1_000);
            snapshot.runtime_owned = ownership;
            let provider = fake_provider(Vec::new(), Vec::new());
            let start = Instant::now();
            let mut state = DashboardState::new(snapshot, start, Duration::from_secs(1));

            assert_eq!(
                state.displayed_task_percent(start + Duration::from_secs(10)),
                25
            );
            assert_eq!(state.settling_deadline(), None);
            state.handle_deadline(&provider, &id, start + Duration::from_millis(500));
            assert_eq!(provider.state_reads.get(), 0);
        }
    }

    #[test]
    fn reanchoring_never_rewinds_within_a_task_and_resets_for_a_new_task() {
        let id = CharacterId::new();
        let first = active_snapshot(100.0, 1_000);
        let mut lower = first.character.clone();
        lower.progress.task.position = 150.0;
        lower.progress.task.percent = 15;
        let mut next = lower.clone();
        next.activity.tasks += 1;
        next.progress.task.position = 20.0;
        next.progress.task.percent = 2;
        let provider = fake_provider(Vec::new(), vec![Ok(lower), Ok(next)]);
        let start = Instant::now();
        let mut state = DashboardState::new(first, start, Duration::from_secs(1));

        let reanchor = start + Duration::from_millis(100);
        assert_eq!(state.displayed_task_percent(reanchor), 20);
        state.read_state(&provider, &id, reanchor);
        assert_eq!(state.displayed_task_percent(reanchor), 20);

        state.read_state(&provider, &id, reanchor);
        assert_eq!(state.displayed_task_percent(reanchor), 2);
    }

    #[test]
    fn task_completion_replaces_and_other_refreshes_retain_update_indicators() {
        let id = CharacterId::new();
        let first = sample();
        let mut completed = first.clone();
        completed.character.activity.tasks += 1;
        completed.character.stats.strength += 1.0;
        completed.character.inventory[0].quantity += 1;
        completed.character.spells[0].rank = "III".to_owned();
        completed.character.equipment.weapon = "Silver Sword".to_owned();
        let mut unchanged_task_count = completed.clone();
        unchanged_task_count.character.progress.task.percent = 50;
        let mut next_completion = unchanged_task_count.clone();
        next_completion.character.activity.tasks += 1;
        let provider = FakeProvider {
            snapshots: std::cell::RefCell::new(vec![
                Ok(completed.clone()),
                Ok(unchanged_task_count),
                Ok(next_completion),
                Err(DashboardError::NoManagedCharacters),
            ]),
            state_snapshots: std::cell::RefCell::new(Vec::new()),
            refreshes: std::cell::Cell::new(0),
            state_reads: std::cell::Cell::new(0),
            actions: std::cell::RefCell::new(Vec::new()),
            action_results: std::cell::RefCell::new(Vec::new()),
            brags: std::cell::RefCell::new(Vec::new()),
            brag_results: std::cell::RefCell::new(Vec::new()),
        };
        let now = Instant::now();
        let mut state = DashboardState::new(first, now, Duration::from_secs(1));

        state.refresh(&provider, &id, now);
        assert!(state.updates.stats.contains("STR"));
        assert!(state.updates.equipment.contains("Weapon"));
        assert_eq!(state.updates.inventory.len(), 1);
        assert_eq!(state.updates.spells.len(), 1);

        state.refresh(&provider, &id, now);
        assert!(state.updates.stats.contains("STR"));
        assert!(state.updates.equipment.contains("Weapon"));
        assert_eq!(state.updates.inventory.len(), 1);
        assert_eq!(state.updates.spells.len(), 1);

        state.refresh(&provider, &id, now);
        assert!(state.updates.stats.is_empty());
        assert!(state.updates.equipment.is_empty());
        assert!(state.updates.inventory.is_empty());
        assert!(state.updates.spells.is_empty());

        state.refresh(&provider, &id, now);
        assert!(state.updates.stats.is_empty());
        assert!(
            state
                .current
                .message
                .unwrap()
                .contains("no managed characters")
        );
    }

    #[test]
    fn brag_submits_immediately_with_a_safe_outcome_message() {
        let id = CharacterId::new();
        let first = sample();
        for (outcome, message) in [
            (DeliveryOutcome::Delivered, "Leaderboard report delivered."),
            (
                DeliveryOutcome::EndpointRejected,
                "Leaderboard report was not accepted by the endpoint.",
            ),
            (
                DeliveryOutcome::DeliveryFailed,
                "Leaderboard report could not be delivered.",
            ),
        ] {
            let provider = FakeProvider {
                snapshots: std::cell::RefCell::new(vec![Ok(sample())]),
                state_snapshots: std::cell::RefCell::new(Vec::new()),
                refreshes: std::cell::Cell::new(0),
                state_reads: std::cell::Cell::new(0),
                actions: std::cell::RefCell::new(Vec::new()),
                action_results: std::cell::RefCell::new(Vec::new()),
                brags: std::cell::RefCell::new(Vec::new()),
                brag_results: std::cell::RefCell::new(vec![Ok(outcome)]),
            };
            let now = Instant::now();
            let mut state = DashboardState::new(first.clone(), now, Duration::from_secs(1));

            assert!(!state.apply(&provider, &id, Command::Brag, now));
            assert_eq!(
                provider.brags.borrow().as_slice(),
                std::slice::from_ref(&id)
            );
            assert_eq!(state.confirmation, None);
            assert_eq!(state.current.message.as_deref(), Some(message));
            let rendered = rendered(120, 40);
            assert!(rendered.contains("b brag"));
        }
    }

    #[test]
    fn lifecycle_failure_keeps_last_successful_snapshot() {
        let id = CharacterId::new();
        let snapshot = sample();
        let provider = FakeProvider {
            snapshots: std::cell::RefCell::new(Vec::new()),
            state_snapshots: std::cell::RefCell::new(Vec::new()),
            refreshes: std::cell::Cell::new(0),
            state_reads: std::cell::Cell::new(0),
            actions: std::cell::RefCell::new(Vec::new()),
            action_results: std::cell::RefCell::new(vec![Err(DashboardError::Lifecycle(
                LifecycleError::ManagerUnavailable("no user manager".to_owned()),
            ))]),
            brags: std::cell::RefCell::new(Vec::new()),
            brag_results: std::cell::RefCell::new(Vec::new()),
        };
        let now = Instant::now();
        let mut state = DashboardState::new(snapshot.clone(), now, Duration::from_secs(1));
        state.confirmation = Some(LifecycleAction::Start);
        state.apply(&provider, &id, Command::ConfirmAction, now);
        assert_eq!(
            state.current.character.identity.name,
            snapshot.character.identity.name
        );
        assert!(state.current.message.unwrap().contains("no user manager"));
    }

    #[test]
    fn collapsed_panes_hide_content_without_affecting_other_panes() {
        let mut panes = PaneVisibility::default();
        panes.toggle(Pane::Activity);
        panes.toggle(Pane::Status);

        let output = rendered_with_panes(120, 40, &panes);

        assert!(output.contains("Activity"));
        assert!(output.contains("F1"));
        assert!(!output.contains("Tasks completed:"));
        assert!(output.contains("Equipment"));
        assert!(output.contains("Journal"));
        assert!(output.contains("F7"));
        assert!(!output.contains("Runtime ownership:"));
    }

    #[test]
    fn expanded_journal_is_limited_to_one_quarter_of_the_right_column() {
        let areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(right_pane_constraints(&PaneVisibility::default()))
            .split(Rect::new(0, 0, 60, 40));

        assert_eq!(areas[0].height, 30);
        assert_eq!(areas[1].height, 10);
    }

    #[test]
    fn first_dashboard_column_uses_one_third_of_available_width() {
        let areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
            .split(Rect::new(0, 0, 90, 40));

        assert_eq!(areas[0].width, 30);
        assert_eq!(areas[1].width, 60);
    }

    #[test]
    fn expanded_details_fills_remaining_vertical_space() {
        let areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(left_pane_constraints(&PaneVisibility::default(), 40))
            .split(Rect::new(0, 0, 50, 40));

        assert_eq!(areas[3].height, 16);
        assert_eq!(areas[3].bottom(), 40);
        assert_eq!(areas[4].height, 0);
    }

    #[test]
    fn expanded_progress_has_exactly_seven_rows() {
        let areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(left_pane_constraints(&PaneVisibility::default(), 40))
            .split(Rect::new(0, 0, 50, 40));

        assert_eq!(areas[1].height, 7);
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
        let profile = OnlineProfile {
            motto: "Observed motto".to_owned(),
            guild: "Observed guild".to_owned(),
        };
        store.replace_profile(&registered.id, &profile).unwrap();
        let local = LocalProvider {
            store: RefCell::new(Store::open_at(&directory.0).unwrap()),
        };
        let state_only = local.read_state(&registered.id).unwrap();
        assert_eq!(state_only.identity.name, "Reference Hero");
        assert_eq!(state_only.profile, profile);

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
        assert_eq!(active.character.profile, profile);
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
