use std::{
    env, fs, io,
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    MouseButton, MouseEventKind,
};
use ratatui::{DefaultTerminal, layout::Rect, widgets::TableState};

use crate::{
    network::NetworkInfo,
    ping::{self, PingResult},
    ui,
};

pub struct PingEntry {
    pub target: String,
    pub status: String,
    pub latency: Option<f64>,
    pub sent: u32,
    pub received: u32,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub last_run: Option<Instant>,
}

impl PingEntry {
    fn new(target: String) -> Self {
        Self {
            target,
            status: "Ready".into(),
            latency: None,
            sent: 0,
            received: 0,
            sum: 0.0,
            min: f64::INFINITY,
            max: 0.0,
            last_run: None,
        }
    }

    pub fn loss(&self) -> f64 {
        if self.sent == 0 {
            return 0.0;
        }

        100.0 * (self.sent - self.received) as f64 / self.sent as f64
    }

    pub fn statistics(&self) -> String {
        let latency = if self.received == 0 {
            "Min / Avg / Max: N/A".into()
        } else {
            format!(
                "Min / Avg / Max: {:.2} / {:.2} / {:.2} ms",
                self.min,
                self.sum / self.received as f64,
                self.max,
            )
        };

        format!(
            "{}\nCompleted probes: {}   Replies: {}   Loss: {:.1}%\n{}\n{}",
            self.target,
            self.sent,
            self.received,
            self.loss(),
            latency,
            self.status,
        )
    }
}

pub struct App {
    running: bool,
    started_at: Instant,

    pub network: NetworkInfo,
    pub tab: usize,
    pub editing: bool,
    pub input: String,
    pub message: String,
    pub entries: Vec<PingEntry>,
    pub table_state: TableState,

    pub input_area: Rect,
    pub table_area: Rect,

    busy: bool,
    sender: Sender<(String, PingResult)>,
    receiver: Receiver<(String, PingResult)>,
}

impl App {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();

        let (entries, message) = match load_history() {
            Ok(entries) => (entries, String::new()),
            Err(error) => (Vec::new(), format!("Cannot load recent targets: {error}")),
        };

        let mut table_state = TableState::default();

        if !entries.is_empty() {
            table_state.select(Some(0));
        }

        Self {
            running: true,
            started_at: Instant::now(),
            network: NetworkInfo::discover(),
            tab: 1,
            editing: false,
            input: String::new(),
            message,
            entries,
            table_state,
            input_area: Rect::default(),
            table_area: Rect::default(),
            busy: false,
            sender,
            receiver,
        }
    }

    pub fn uptime(&self) -> Duration {
        self.started_at.elapsed()
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        crossterm::execute!(io::stdout(), EnableMouseCapture)?;

        let result = self.run_loop(terminal);
        let cleanup = crossterm::execute!(io::stdout(), DisableMouseCapture);

        result.and(cleanup)
    }

    fn run_loop(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while self.running {
            self.receive_results();

            terminal.draw(|frame| ui::render(frame, self))?;

            if event::poll(Duration::from_millis(50))? {
                self.handle_event(event::read()?);
            }
        }

        Ok(())
    }

    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    self.running = false;
                    return;
                }

                if self.editing {
                    match key.code {
                        KeyCode::Esc => self.editing = false,
                        KeyCode::Enter => self.submit_input(),
                        KeyCode::Backspace => {
                            self.input.pop();
                        }
                        KeyCode::Char(character)
                            if character.is_ascii()
                                && !character.is_ascii_control()
                                && self.input.len() < 253 =>
                        {
                            self.input.push(character);
                        }
                        _ => {}
                    }

                    return;
                }

                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => self.running = false,
                    KeyCode::Tab => self.tab = (self.tab + 1) % 2,
                    KeyCode::Char('1') => self.tab = 0,
                    KeyCode::Char('2') => self.tab = 1,
                    KeyCode::Char('/') if self.tab == 1 => self.editing = true,
                    KeyCode::Up if self.tab == 1 => self.move_selection(-1),
                    KeyCode::Down if self.tab == 1 => self.move_selection(1),
                    KeyCode::Enter if self.tab == 1 => self.ping_selected(),
                    KeyCode::Delete if self.tab == 1 => self.delete_selected(),
                    _ => {}
                }
            }
            Event::Mouse(mouse) if self.tab == 1 => match mouse.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    if contains(self.input_area, mouse.column, mouse.row) {
                        self.editing = true;
                    } else if contains(self.table_area, mouse.column, mouse.row) {
                        // Skip the top border and table header.
                        let first_row = self.table_area.y + 2;

                        if mouse.row >= first_row
                            && mouse.row < self.table_area.bottom().saturating_sub(1)
                        {
                            let index =
                                self.table_state.offset() + usize::from(mouse.row - first_row);

                            if index < self.entries.len() {
                                self.editing = false;
                                self.table_state.select(Some(index));
                                self.ping_selected();
                            }
                        }
                    }
                }
                MouseEventKind::ScrollUp => self.move_selection(-1),
                MouseEventKind::ScrollDown => self.move_selection(1),
                _ => {}
            },
            _ => {}
        }
    }

    fn move_selection(&mut self, direction: isize) {
        if self.entries.is_empty() {
            return;
        }

        let current = self.table_state.selected().unwrap_or(0);
        let next = current
            .saturating_add_signed(direction)
            .min(self.entries.len() - 1);

        self.table_state.select(Some(next));
    }

    fn submit_input(&mut self) {
        if self.busy {
            self.message = "Wait for the current ping to finish.".into();
            return;
        }

        let target = match ping::validate_target(&self.input) {
            Ok(target) => target,
            Err(error) => {
                self.message = error;
                return;
            }
        };

        let index = match self.entries.iter().position(|entry| entry.target == target) {
            Some(index) => index,
            None => {
                self.entries.push(PingEntry::new(target));
                self.entries.len() - 1
            }
        };

        self.editing = false;
        self.input.clear();
        self.table_state.select(Some(index));
        self.ping_selected();
    }

    fn ping_selected(&mut self) {
        if self.busy {
            self.message = "Wait for the current ping to finish.".into();
            return;
        }

        let Some(index) = self.table_state.selected() else {
            self.message = "Press / to enter a destination.".into();
            return;
        };

        // Move the most recently used destination to the top.
        let entry = self.entries.remove(index);
        self.entries.insert(0, entry);
        self.table_state.select(Some(0));

        self.message.clear();
        self.save_history();

        let entry = &mut self.entries[0];
        entry.status = "Running...".into();
        entry.latency = None;
        entry.last_run = Some(Instant::now());

        let target = entry.target.clone();
        let sender = self.sender.clone();

        match thread::Builder::new()
            .name("ping-worker".into())
            .spawn(move || {
                let result = ping::ping(&target);
                let _ = sender.send((target, result));
            }) {
            Ok(_) => self.busy = true,
            Err(error) => {
                self.entries[0].status = format!("Cannot start worker: {error}");
            }
        }
    }

    fn receive_results(&mut self) {
        while let Ok((target, result)) = self.receiver.try_recv() {
            self.busy = false;

            let Some(entry) = self.entries.iter_mut().find(|entry| entry.target == target) else {
                continue;
            };

            match result {
                Ok(Some(latency)) => {
                    entry.sent += 1;
                    entry.received += 1;
                    entry.sum += latency;
                    entry.min = entry.min.min(latency);
                    entry.max = entry.max.max(latency);
                    entry.latency = Some(latency);
                    entry.status = "Reply received".into();
                }
                Ok(None) => {
                    entry.sent += 1;
                    entry.status = "No reply".into();
                }
                Err(error) => {
                    entry.status = format!("Error: {error}");
                }
            }
        }
    }

    fn delete_selected(&mut self) {
        if self.busy {
            self.message = "Wait for the current ping before deleting a target.".into();
            return;
        }

        if let Some(index) = self.table_state.selected() {
            self.entries.remove(index);

            let selected = if self.entries.is_empty() {
                None
            } else {
                Some(index.min(self.entries.len() - 1))
            };

            self.table_state.select(selected);
            self.save_history();
        }
    }

    fn save_history(&mut self) {
        if let Err(error) = save_history(&self.entries) {
            self.message = format!("Cannot save recent targets: {error}");
        }
    }
}

fn contains(area: Rect, x: u16, y: u16) -> bool {
    x >= area.x && x < area.right() && y >= area.y && y < area.bottom()
}

fn history_path() -> io::Result<PathBuf> {
    let base = env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| {
            env::var_os("HOME")
                .map(PathBuf::from)
                .map(|home| home.join(".local/state"))
        })
        .ok_or_else(|| io::Error::other("Cannot locate the user state directory"))?;

    Ok(base.join("netracer/recent-targets.txt"))
}

fn load_history() -> io::Result<Vec<PingEntry>> {
    let content = match fs::read_to_string(history_path()?) {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error),
    };

    let mut entries = Vec::new();

    for line in content.lines() {
        if let Ok(target) = ping::validate_target(line)
            && !entries
                .iter()
                .any(|entry: &PingEntry| entry.target == target)
        {
            entries.push(PingEntry::new(target));
        }
    }

    Ok(entries)
}

fn save_history(entries: &[PingEntry]) -> io::Result<()> {
    let path = history_path()?;
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("Invalid history path"))?;

    fs::create_dir_all(parent)?;

    let content = entries
        .iter()
        .map(|entry| entry.target.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));

    fs::write(&temporary, content)?;
    fs::rename(temporary, path)
}
