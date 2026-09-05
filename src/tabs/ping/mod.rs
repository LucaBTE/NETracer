mod history;
mod input;
mod model;
mod ui;

use crossterm::event::Event;
use futures_util::future::LocalBoxFuture;
use ratatui::{Frame, layout::Rect, widgets::TableState};
use tokio::task::JoinSet;
use tui_input::Input;

use crate::{
    components::Component,
    network::{
        ping::{self, PingOutcome},
        target::Target,
    },
};

use history::History;
use model::{PingEntry, ProbeState};

pub struct PingTab {
    input: Input,
    editing: bool,
    entries: Vec<PingEntry>,
    table_state: TableState,
    message: String,
    history: Option<History>,
    history_writable: bool,
    tasks: JoinSet<(Target, PingOutcome)>,
    input_area: Rect,
    table_area: Rect,
}

impl PingTab {
    pub fn new() -> Self {
        Self::from_history(History::discover())
    }

    fn from_history(history: std::io::Result<History>) -> Self {
        let (history, entries, message, writable) = match history {
            Ok(history) => match history.load() {
                Ok(targets) => (
                    Some(history),
                    targets.into_iter().map(PingEntry::new).collect::<Vec<_>>(),
                    String::new(),
                    true,
                ),

                Err(error) => (
                    Some(history),
                    Vec::new(),
                    format!("History not loaded; saving disabled: {error}",),
                    false,
                ),
            },

            Err(error) => (
                None,
                Vec::new(),
                format!("Recent targets will not be saved: {error}",),
                false,
            ),
        };

        let mut table_state = TableState::default();

        if !entries.is_empty() {
            table_state.select(Some(0));
        }

        Self {
            input: Input::default(),
            editing: false,
            entries,
            table_state,
            message,
            history,
            history_writable: writable,
            tasks: JoinSet::new(),
            input_area: Rect::default(),
            table_area: Rect::default(),
        }
    }

    fn busy(&self) -> bool {
        !self.tasks.is_empty()
    }

    fn submit(&mut self) {
        if self.busy() {
            self.message = "Wait for the current ping to finish.".into();
            return;
        }

        let target = match Target::parse(self.input.value()) {
            Ok(target) => target,

            Err(error) => {
                self.message = error;
                return;
            }
        };

        let index = self
            .entries
            .iter()
            .position(|entry| entry.target == target)
            .unwrap_or_else(|| {
                self.entries.push(PingEntry::new(target));
                self.entries.len() - 1
            });

        self.table_state.select(Some(index));
        self.editing = false;
        self.input.reset();

        self.start_selected();
    }

    fn start_selected(&mut self) {
        if self.busy() {
            self.message = "Wait for the current ping to finish.".into();
            return;
        }

        let Some(index) = self
            .table_state
            .selected()
            .filter(|index| *index < self.entries.len())
        else {
            self.message = "Press / to enter a destination.".into();
            return;
        };

        let mut entry = self.entries.remove(index);
        entry.start();

        let target = entry.target.clone();

        self.entries.insert(0, entry);
        self.table_state.select(Some(0));

        self.message.clear();
        self.save_history();

        self.tasks.spawn(async move {
            let outcome = ping::ping(&target).await;

            (target, outcome)
        });
    }

    fn move_selection(&mut self, direction: isize) {
        if !self.entries.is_empty() {
            let current = self.table_state.selected().unwrap_or(0);

            let next = current
                .saturating_add_signed(direction)
                .min(self.entries.len() - 1);

            self.table_state.select(Some(next));
        }
    }

    fn delete_selected(&mut self) {
        if self.busy() {
            self.message = "Wait for the current ping before removing a target.".into();
            return;
        }

        if let Some(index) = self
            .table_state
            .selected()
            .filter(|index| *index < self.entries.len())
        {
            self.entries.remove(index);

            let next = if self.entries.is_empty() {
                None
            } else {
                Some(index.min(self.entries.len() - 1))
            };

            self.table_state.select(next);

            self.message.clear();
            self.save_history();
        }
    }

    fn save_history(&mut self) {
        if !self.history_writable {
            self.message = "History unavailable: changes are kept only for this session.".into();
            return;
        }

        if let Some(history) = &self.history
            && let Err(error) = history.save(self.entries.iter().map(|entry| &entry.target))
        {
            self.message = format!("Cannot save recent targets: {error}");
        }
    }
}

impl Default for PingTab {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for PingTab {
    fn id(&self) -> &'static str {
        "ping"
    }

    fn title(&self) -> &'static str {
        "Ping"
    }

    fn help(&self) -> &'static str {
        if self.editing {
            "ENTER: ping  ESC: close input  ←/→: cursor"
        } else {
            "/: target  ↑/↓: select  ENTER: ping  DEL: remove  Q: quit"
        }
    }

    fn handle_event(&mut self, event: &Event) -> bool {
        input::handle(self, event)
    }

    fn update(&mut self) {
        while let Some(result) = self.tasks.try_join_next() {
            match result {
                Ok((target, outcome)) => {
                    if let Some(entry) =
                        self.entries.iter_mut().find(|entry| entry.target == target)
                    {
                        entry.finish(outcome);
                    }
                }

                Err(error) => {
                    for entry in &mut self.entries {
                        if matches!(entry.state, ProbeState::Running) {
                            entry.finish(PingOutcome::Error(format!(
                                "Background task failed: {error}",
                            )));
                        }
                    }
                }
            }
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        ui::render(frame, area, self);
    }

    fn reset_layout(&mut self) {
        self.input_area = Rect::default();
        self.table_area = Rect::default();
    }

    fn shutdown(&mut self) -> LocalBoxFuture<'_, ()> {
        Box::pin(async move {
            self.tasks.shutdown().await;
        })
    }
}
