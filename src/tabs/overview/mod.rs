mod model;
mod ui;

use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use futures_util::future::LocalBoxFuture;
use ratatui::{Frame, layout::Rect};
use tokio::{sync::mpsc, task::JoinHandle, time};

use crate::{
    components::Component,
    network::{
        interfaces::NetworkInfo,
        traffic::{self, InterfaceSnapshot},
    },
};

use model::TrafficModel;

type Collection = Result<Vec<InterfaceSnapshot>, String>;

pub struct OverviewTab {
    hostname: String,
    started_at: Instant,
    traffic: TrafficModel,
    receiver: mpsc::UnboundedReceiver<(Instant, Collection)>,
    worker: Option<JoinHandle<()>>,
    interface_list_area: Rect,
    interface_list_offset: usize,
}

impl OverviewTab {
    pub fn new() -> Self {
        let network = NetworkInfo::discover();
        let (sender, receiver) = mpsc::unbounded_channel();
        let worker = tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(1));
            interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

            loop {
                interval.tick().await;
                let result = traffic::read_interfaces()
                    .await
                    .map_err(|error| error.to_string());
                let at = Instant::now();
                if sender.send((at, result)).is_err() {
                    break;
                }
            }
        });

        Self {
            hostname: network.hostname,
            started_at: Instant::now(),
            traffic: TrafficModel::new(network.interface),
            receiver,
            worker: Some(worker),
            interface_list_area: Rect::default(),
            interface_list_offset: 0,
        }
    }
}

impl Default for OverviewTab {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for OverviewTab {
    fn id(&self) -> &'static str {
        "overview"
    }

    fn title(&self) -> &'static str {
        "Overview"
    }

    fn help(&self) -> &'static str {
        "↑/↓: interface  CLICK: select  Q/ESC: quit"
    }

    fn handle_event(&mut self, event: &Event) -> bool {
        match event {
            Event::Key(key) if key.kind != KeyEventKind::Release => match key.code {
                KeyCode::Up => {
                    self.traffic.move_selection(-1);
                    true
                }
                KeyCode::Down => {
                    self.traffic.move_selection(1);
                    true
                }
                _ => false,
            },
            Event::Mouse(mouse) if mouse.kind == MouseEventKind::Down(MouseButton::Left) => {
                let position = (mouse.column, mouse.row).into();
                if !self.interface_list_area.contains(position) {
                    return false;
                }

                let first_row = self.interface_list_area.y + 1;
                if mouse.row >= first_row {
                    let index = self.interface_list_offset + usize::from(mouse.row - first_row);
                    if index < self.traffic.choices().count() {
                        self.traffic.select(index);
                    }
                }
                true
            }
            _ => false,
        }
    }

    fn update(&mut self) {
        while let Ok((at, collection)) = self.receiver.try_recv() {
            match collection {
                Ok(interfaces) => self.traffic.record_collection(at, interfaces),
                Err(error) => self.traffic.record_failure(at, error),
            }
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        ui::render(
            frame,
            area,
            &self.hostname,
            self.started_at.elapsed(),
            &self.traffic,
            &mut self.interface_list_area,
            &mut self.interface_list_offset,
        );
    }

    fn reset_layout(&mut self) {
        self.interface_list_area = Rect::default();
    }

    fn shutdown(&mut self) -> LocalBoxFuture<'_, ()> {
        Box::pin(async move {
            if let Some(worker) = self.worker.take() {
                worker.abort();
                let _ = worker.await;
            }
        })
    }
}
