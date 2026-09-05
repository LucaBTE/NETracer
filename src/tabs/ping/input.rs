use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind};
use tui_input::{InputRequest, backend::crossterm::EventHandler};

use super::PingTab;

const MAX_INPUT_BYTES: usize = 254;

pub(super) fn handle(tab: &mut PingTab, event: &Event) -> bool {
    match event {
        Event::Key(key) if key.kind != KeyEventKind::Release => {
            if tab.editing {
                match key.code {
                    KeyCode::Esc => {
                        tab.editing = false;
                    }

                    KeyCode::Enter if key.kind == KeyEventKind::Press => {
                        tab.submit();
                    }

                    KeyCode::Enter => {}

                    KeyCode::Tab | KeyCode::BackTab => {
                        return false;
                    }

                    KeyCode::Char(character)
                        if !key
                            .modifiers
                            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
                    {
                        if character.is_ascii()
                            && !character.is_ascii_control()
                            && tab.input.value().len() < MAX_INPUT_BYTES
                        {
                            tab.input.handle(InputRequest::InsertChar(character));
                        }
                    }

                    _ => {
                        tab.input.handle_event(event);
                    }
                }

                return true;
            }

            // Allow navigation repeats, but not repeated deletion.
            match key.code {
                KeyCode::Up => {
                    tab.move_selection(-1);
                }

                KeyCode::Down => {
                    tab.move_selection(1);
                }

                KeyCode::Char('/') if key.kind == KeyEventKind::Press => {
                    tab.editing = true;
                }

                KeyCode::Enter if key.kind == KeyEventKind::Press => {
                    tab.start_selected();
                }

                KeyCode::Delete if key.kind == KeyEventKind::Press => {
                    tab.delete_selected();
                }

                _ => return false,
            }

            true
        }

        Event::Paste(text) if tab.editing => {
            let remaining = MAX_INPUT_BYTES.saturating_sub(tab.input.value().len());

            if text.len() > remaining || !text.is_ascii() || text.chars().any(char::is_control) {
                tab.message = "Paste a single ASCII hostname or IP address \
                     (max 254 characters)."
                    .into();
            } else {
                for character in text.chars() {
                    tab.input.handle(InputRequest::InsertChar(character));
                }
            }

            true
        }

        Event::Mouse(mouse) => {
            let position = (mouse.column, mouse.row).into();

            match mouse.kind {
                MouseEventKind::Down(MouseButton::Left) if tab.input_area.contains(position) => {
                    tab.editing = true;
                    true
                }

                MouseEventKind::Down(MouseButton::Left) if tab.table_area.contains(position) => {
                    let first_row = tab.table_area.y + 2;

                    if mouse.row >= first_row
                        && mouse.row < tab.table_area.bottom().saturating_sub(1)
                    {
                        let index = tab.table_state.offset() + usize::from(mouse.row - first_row);

                        if index < tab.entries.len() {
                            tab.editing = false;
                            tab.table_state.select(Some(index));
                            tab.start_selected();
                        }
                    }

                    true
                }

                MouseEventKind::ScrollUp if tab.table_area.contains(position) => {
                    tab.move_selection(-1);
                    true
                }

                MouseEventKind::ScrollDown if tab.table_area.contains(position) => {
                    tab.move_selection(1);
                    true
                }

                _ => false,
            }
        }

        _ => false,
    }
}
