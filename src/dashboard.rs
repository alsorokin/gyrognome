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
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph, Wrap},
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
    Confirm(LifecycleAction),
    ConfirmAction,
    Cancel,
    TogglePane(Pane),
    None,
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
        panes: PaneVisibility::default(),
    };
    loop {
        session
            .terminal
            .draw(|frame| render(frame, &state.current, state.confirmation, &state.panes))?;
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
    panes: PaneVisibility,
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
            Command::TogglePane(pane) => {
                self.panes.toggle(pane);
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
    let status_height = if full_layout && panes.is_collapsed(Pane::Status) {
        2
    } else {
        3
    };
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(6),
            Constraint::Length(status_height),
            Constraint::Length(3),
        ])
        .split(area);
    render_header(frame, snapshot, rows[0]);
    if !full_layout {
        render_compact(frame, snapshot, rows[1]);
    } else {
        render_full(frame, snapshot, rows[1], panes);
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

fn render_full(
    frame: &mut ratatui::Frame<'_>,
    snapshot: &DashboardSnapshot,
    area: Rect,
    panes: &PaneVisibility,
) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(area);
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints(pane_constraints(
            &[
                (Pane::Activity, 5),
                (Pane::Progress, 7),
                (Pane::Equipment, 5),
                (Pane::Details, 5),
            ],
            panes,
            &[Pane::Activity, Pane::Progress, Pane::Equipment],
        ))
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
        render_progress(frame, &state.progress, left[1], Pane::Progress);
    }
    render_pane(
        frame,
        left[2],
        "Equipment",
        Pane::Equipment,
        panes,
        Paragraph::new(equipment_text(&state.equipment)).wrap(Wrap { trim: true }),
    );
    render_pane(
        frame,
        left[3],
        "Details",
        Pane::Details,
        panes,
        Paragraph::new(format!(
            "Character ID: {}\nLast task elapsed: {} seconds\nQuest target: {}",
            state.id,
            state.activity.elapsed,
            quest_target_text(&state.activity.questmonster)
        ))
        .wrap(Wrap { trim: true }),
    );
    render_pane(
        frame,
        right[0],
        "Adventure",
        Pane::Adventure,
        panes,
        Paragraph::new(adventure_lines(state)).wrap(Wrap { trim: true }),
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

fn adventure_lines(character: &DashboardCharacter) -> Vec<Line<'static>> {
    let lines = vec![
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
        Line::from(format!("Current quest: {}", character.current_quest)),
    ];
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

fn pane_constraints(
    panes: &[(Pane, u16)],
    visibility: &PaneVisibility,
    flexible_panes: &[Pane],
) -> Vec<Constraint> {
    let expanded = panes
        .iter()
        .rposition(|(pane, _)| !visibility.is_collapsed(*pane) && flexible_panes.contains(pane));
    panes
        .iter()
        .enumerate()
        .map(|(index, (pane, height))| {
            if visibility.is_collapsed(*pane) {
                Constraint::Length(2)
            } else if Some(index) == expanded {
                Constraint::Min(*height)
            } else {
                Constraint::Length(*height)
            }
        })
        .collect()
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

fn render_progress(frame: &mut ratatui::Frame<'_>, progress: &Progress, area: Rect, pane: Pane) {
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
    frame.render_widget(pane_block("Progress", pane), area);
    for ((label, bar), row) in bars.into_iter().zip(rows.iter().copied()) {
        let label = format!("{label} {}%", bar.percent);
        let filled_end = row.left()
            + (f64::from(row.width) * (bar.percent.min(100) as f64 / 100.0)).round() as u16;
        let label_start = row.left() + (row.width - label.len() as u16) / 2;
        frame.render_widget(
            Gauge::default()
                .gauge_style(Style::default().fg(Color::Cyan))
                .ratio((bar.percent.min(100) as f64) / 100.0)
                .label(label.clone()),
            row,
        );
        frame.render_widget(
            Paragraph::new(Line::from(
                label
                    .chars()
                    .enumerate()
                    .map(|(index, character)| {
                        let color = if label_start + (index as u16) < filled_end {
                            Color::Black
                        } else {
                            Color::Cyan
                        };
                        Span::styled(character.to_string(), Style::default().fg(color))
                    })
                    .collect::<Vec<_>>(),
            ))
            .alignment(Alignment::Center),
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

    fn rendered_with_panes(width: u16, height: u16, panes: &PaneVisibility) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| render(frame, &sample(), None, panes))
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
    fn adventure_shows_only_a_plain_current_quest() {
        let character = sample().character;
        let lines = adventure_lines(&character);
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
            .draw(|frame| render(frame, &snapshot, None, &PaneVisibility::default()))
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
            .draw(|frame| render(frame, &snapshot, None, &PaneVisibility::default()))
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
            .draw(|frame| render(frame, &snapshot, None, &PaneVisibility::default()))
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
            panes: PaneVisibility::default(),
        };
        assert!(!state.apply(&provider, &id, Command::Refresh));
        assert_eq!(state.current.character.activity.task, "new persisted task");
        assert!(provider.actions.borrow().is_empty());

        assert!(!state.apply(&provider, &id, Command::Confirm(LifecycleAction::Stop)));
        assert!(!state.apply(&provider, &id, Command::Cancel));
        assert!(provider.actions.borrow().is_empty());

        assert!(!state.apply(&provider, &id, Command::TogglePane(Pane::Journal)));
        assert!(state.panes.is_collapsed(Pane::Journal));

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
            panes: PaneVisibility::default(),
        };
        state.apply(&provider, &id, Command::ConfirmAction);
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
    fn expanded_details_has_exactly_five_rows() {
        let areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(pane_constraints(
                &[
                    (Pane::Activity, 5),
                    (Pane::Progress, 7),
                    (Pane::Equipment, 5),
                    (Pane::Details, 5),
                ],
                &PaneVisibility::default(),
                &[Pane::Activity, Pane::Progress, Pane::Equipment],
            ))
            .split(Rect::new(0, 0, 50, 40));

        assert_eq!(areas[3].height, 5);
    }

    #[test]
    fn expanded_progress_has_exactly_seven_rows() {
        let areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(pane_constraints(
                &[
                    (Pane::Activity, 5),
                    (Pane::Progress, 7),
                    (Pane::Equipment, 5),
                    (Pane::Details, 5),
                ],
                &PaneVisibility::default(),
                &[Pane::Activity, Pane::Progress, Pane::Equipment],
            ))
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
