use super::*;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Terminal, backend::TestBackend};

fn key(code: KeyCode) -> Event { Event::Key(KeyEvent::new(code, KeyModifiers::NONE)) }

fn tab() -> (tempfile::TempDir, PingTab) {
    let dir = tempfile::tempdir().unwrap();
    let history = History::at(dir.path().join("recent-targets.txt"));
    (dir, PingTab::from_history(Ok(history)))
}

#[test]
fn input_consumes_quit_and_delete_and_supports_cursor_editing() {
    let (_dir, mut tab) = tab();
    assert!(tab.handle_event(&key(KeyCode::Char('/'))));
    for character in "abq".chars() {
        assert!(tab.handle_event(&key(KeyCode::Char(character))));
    }
    tab.handle_event(&key(KeyCode::Left));
    tab.handle_event(&key(KeyCode::Delete));
    assert_eq!(tab.input.value(), "ab");
    tab.handle_event(&key(KeyCode::Esc));
    assert!(!tab.handle_event(&key(KeyCode::Char('q'))));
}

#[test]
fn delete_removes_selected_target_from_disk_and_handles_empty_list() {
    let (dir, mut tab) = tab();
    tab.entries = ["one.example", "two.example"].into_iter()
        .map(|name| PingEntry::new(Target::parse(name).unwrap())).collect();
    tab.table_state.select(Some(1));
    tab.handle_event(&key(KeyCode::Delete));
    assert_eq!(tab.entries[0].target.as_str(), "one.example");
    assert_eq!(tab.table_state.selected(), Some(0));
    let history = History::at(dir.path().join("recent-targets.txt"));
    assert_eq!(history.load().unwrap().len(), 1);
    tab.handle_event(&key(KeyCode::Delete));
    assert_eq!(tab.table_state.selected(), None);
    assert!(history.load().unwrap().is_empty());
}

#[tokio::test]
async fn completion_updates_state_without_rendering_and_delete_waits() {
    let (_dir, mut tab) = tab();
    let target = Target::parse("localhost").unwrap();
    let mut entry = PingEntry::new(target.clone());
    entry.start();
    tab.entries.push(entry);
    tab.table_state.select(Some(0));
    tab.tasks.spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        (target, PingOutcome::Reply { latency_ms: Some(2.0) })
    });
    tab.delete_selected();
    assert_eq!(tab.entries.len(), 1);
    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        while tab.busy() {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            tab.update();
        }
    }).await.unwrap();
    assert_eq!(tab.entries[0].statistics.received, 1);
    tab.shutdown().await;
}

#[test]
fn renders_and_scrolls_selected_target() {
    let (_dir, mut tab) = tab();
    tab.entries = (0..30).map(|index| {
        PingEntry::new(Target::parse(&format!("host{index}.example")).unwrap())
    }).collect();
    tab.table_state.select(Some(29));
    let mut terminal = Terminal::new(TestBackend::new(90, 30)).unwrap();
    terminal.draw(|frame| tab.render(frame, frame.area())).unwrap();
    assert!(tab.table_state.offset() > 0);
    let screen = terminal.backend().buffer().content.iter()
        .map(|cell| cell.symbol()).collect::<String>();
    assert!(screen.contains("host29.example"));
    tab.reset_layout();
    assert_eq!(tab.table_area, Rect::default());
}
