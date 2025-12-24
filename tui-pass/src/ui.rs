use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Widget, Wrap},
};
use tui_input::Input;

use crate::crypto::Credential;

/// Widget for displaying the list of credentials (using titles only)
pub struct CredentialList<'a> {
    titles: &'a [String],
    selected: Option<usize>,
    scroll_offset: usize,
}

impl<'a> CredentialList<'a> {
    pub fn new(
        titles: &'a [String],
        selected: Option<usize>,
        scroll_offset: usize,
    ) -> Self {
        Self {
            titles,
            selected,
            scroll_offset,
        }
    }
}

impl Widget for CredentialList<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Credentials ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        block.render(area, buf);

        if self.titles.is_empty() {
            let empty_msg = Paragraph::new("No credentials yet.\n\nPress 'a' to add a new one.")
                .style(Style::default().fg(Color::DarkGray))
                .wrap(Wrap { trim: true });
            empty_msg.render(inner, buf);
            return;
        }

        // Calculate visible range
        let visible_height = inner.height as usize;
        let end = (self.scroll_offset + visible_height).min(self.titles.len());

        // Create list items for visible range
        let items: Vec<ListItem> = self.titles[self.scroll_offset..end]
            .iter()
            .enumerate()
            .map(|(idx, title)| {
                let actual_idx = self.scroll_offset + idx;
                let is_selected = Some(actual_idx) == self.selected;

                let style = if is_selected {
                    Style::default()
                        .bg(Color::Blue)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let content = format!("  {}", title);
                ListItem::new(content).style(style)
            })
            .collect();

        let list = List::new(items);
        Widget::render(list, inner, buf);
    }
}

/// Widget for displaying credential details
pub struct CredentialDetail<'a> {
    credential: Option<&'a Credential>,
    show_password: bool,
}

impl<'a> CredentialDetail<'a> {
    pub fn new(credential: Option<&'a Credential>, show_password: bool) -> Self {
        Self {
            credential,
            show_password,
        }
    }
}

impl Widget for CredentialDetail<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Details ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        block.render(area, buf);

        let Some(cred) = self.credential else {
            let empty_msg = Paragraph::new("Select a credential to view details")
                .style(Style::default().fg(Color::DarkGray))
                .wrap(Wrap { trim: true });
            empty_msg.render(inner, buf);
            return;
        };

        // Create detail lines
        let mut lines = Vec::new();

        // Title
        lines.push(Line::from(vec![
            Span::styled("Title: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(&cred.title),
        ]));
        lines.push(Line::from(""));

        // Username
        lines.push(Line::from(vec![
            Span::styled("Username: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(&cred.username),
        ]));
        lines.push(Line::from(""));

        // Password
        let password_display = if self.show_password {
            cred.password.clone()
        } else {
            "•".repeat(cred.password.len().min(12))
        };
        lines.push(Line::from(vec![
            Span::styled("Password: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(password_display),
            Span::styled(
                if self.show_password { " (visible)" } else { " (hidden)" },
                Style::default().fg(Color::DarkGray),
            ),
        ]));
        lines.push(Line::from(""));

        // URL
        lines.push(Line::from(vec![
            Span::styled("URL: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(&cred.url, Style::default().fg(Color::Blue)),
        ]));
        lines.push(Line::from(""));

        // Notes
        if !cred.notes.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("Notes: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]));
            lines.push(Line::from(cred.notes.as_str()));
        }

        let detail = Paragraph::new(lines).wrap(Wrap { trim: false });
        detail.render(inner, buf);
    }
}

/// Widget for displaying help text
pub struct HelpBar;

impl Widget for HelpBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let help_text = " ↑/↓: Navigate | Space: Toggle Password | c: Copy Mode | a: Add | e: Edit | d: Delete | s: Save | q: Quit ";
        let style = Style::default().bg(Color::DarkGray).fg(Color::White);
        let help = Paragraph::new(help_text).style(style);
        help.render(area, buf);
    }
}

/// Input dialog for adding/editing credentials
pub struct InputDialog<'a> {
    pub title: &'a str,
    pub title_input: &'a Input,
    pub username_input: &'a Input,
    pub password_input: &'a Input,
    pub url_input: &'a Input,
    pub notes_input: &'a Input,
    pub active_field: usize,
}

impl<'a> InputDialog<'a> {
    /// Calculate cursor position for the active field
    pub fn cursor_position(&self, area: Rect) -> Option<(u16, u16)> {
        let fields = [
            self.title_input,
            self.username_input,
            self.password_input,
            self.url_input,
            self.notes_input,
        ];

        // Calculate dialog size (centered) - same as in render
        let width = area.width.min(60);
        let height = (fields.len() * 3 + 4).min(area.height as usize) as u16;
        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;

        let dialog_area = Rect {
            x: area.x + x,
            y: area.y + y,
            width,
            height,
        };

        // Calculate inner area (same as in render)
        let block = Block::default().borders(Borders::ALL);
        let inner = block.inner(dialog_area);

        // Calculate position of active field
        let mut y_offset = 0;
        for (idx, input) in fields.iter().enumerate() {
            if y_offset >= inner.height {
                return None;
            }

            if idx == self.active_field {
                // The input field is rendered with a bordered block
                // The actual text area is inside the block borders
                let value_area = Rect {
                    x: inner.x,
                    y: inner.y + y_offset,
                    width: inner.width,
                    height: 3,
                };

                // Account for the block borders (1 char on each side)
                let text_inner_x = value_area.x + 1;
                let text_inner_y = value_area.y + 1;
                let text_inner_width = value_area.width.saturating_sub(2);

                // Get cursor position within the input
                let cursor_offset = input.visual_cursor();
                let scroll = input.visual_scroll(text_inner_width as usize);

                // Calculate actual cursor position accounting for scroll
                let cursor_x = text_inner_x + (cursor_offset.saturating_sub(scroll)) as u16;
                let cursor_y = text_inner_y;

                return Some((cursor_x, cursor_y));
            }

            y_offset += 3;
        }

        None
    }
}

impl Widget for InputDialog<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Field definitions with labels
        let fields = [
            ("Title:", self.title_input),
            ("Username:", self.username_input),
            ("Password:", self.password_input),
            ("URL:", self.url_input),
            ("Notes:", self.notes_input),
        ];

        // Calculate dialog size (centered)
        let width = area.width.min(60);
        let height = (fields.len() * 3 + 4).min(area.height as usize) as u16;
        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;

        let dialog_area = Rect {
            x: area.x + x,
            y: area.y + y,
            width,
            height,
        };

        // Draw semi-transparent background
        for dy in 0..area.height {
            for dx in 0..area.width {
                if let Some(cell) = buf.cell_mut((area.x + dx, area.y + dy)) {
                    cell.set_style(Style::default().bg(Color::Black).fg(Color::DarkGray));
                }
            }
        }

        // Draw dialog
        let block = Block::default()
            .title(format!(" {} ", self.title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black));

        let inner = block.inner(dialog_area);
        block.render(dialog_area, buf);

        // Draw fields
        let mut y_offset = 0;
        for (idx, (label, input)) in fields.iter().enumerate() {
            if y_offset >= inner.height {
                break;
            }

            let is_active = idx == self.active_field;
            let style = if is_active {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            // Label
            let label_area = Rect {
                x: inner.x,
                y: inner.y + y_offset,
                width: inner.width,
                height: 1,
            };
            // let label_text = Paragraph::new(*label).style(style);
            // label_text.render(label_area, buf);
            // y_offset += 1;

            // Value (using tui-input's rendering)
            if y_offset < inner.height {
                let value_area = Rect {
                    x: inner.x,
                    y: inner.y + y_offset,
                    width: inner.width,
                    height: 3,
                };

                let value_style = if is_active {
                    Style::default().bg(Color::DarkGray).fg(Color::White)
                } else {
                    Style::default().fg(Color::Gray)
                };

                // Create paragraph from Input value
                // let display_value = if is_active {
                //     format!("{}_", input.value())
                // } else {
                //     input.value().to_string()
                // };

                let scroll = input.visual_scroll(value_area.width as usize);

                let value_text = Paragraph::new(input.value()).style(value_style)
                .scroll((0, scroll as u16))
                .block(Block::bordered().title(*label));
                value_text.render(value_area, buf);
                y_offset += 3;
            }
        }

        // Draw instructions at the bottom
        if y_offset < inner.height {
            let instructions_area = Rect {
                x: inner.x,
                y: inner.y + inner.height.saturating_sub(1),
                width: inner.width,
                height: 1,
            };
            let instructions = Paragraph::new("↑/↓: Navigate | Tab: Next | Shift+Tab: Prev | Enter: Save | Esc: Cancel")
                .style(Style::default().fg(Color::DarkGray));
            instructions.render(instructions_area, buf);
        }
    }
}

/// Confirmation dialog
pub struct ConfirmDialog<'a> {
    pub message: &'a str,
}

impl Widget for ConfirmDialog<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Calculate dialog size (centered)
        let width = area.width.min(50);
        let height = 7;
        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;

        let dialog_area = Rect {
            x: area.x + x,
            y: area.y + y,
            width,
            height,
        };

        // Draw semi-transparent background
        for dy in 0..area.height {
            for dx in 0..area.width {
                if let Some(cell) = buf.cell_mut((area.x + dx, area.y + dy)) {
                    cell.set_style(Style::default().bg(Color::Black).fg(Color::DarkGray));
                }
            }
        }

        // Draw dialog
        let block = Block::default()
            .title(" Confirm ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red))
            .style(Style::default().bg(Color::Black));

        let inner = block.inner(dialog_area);
        block.render(dialog_area, buf);

        // Message
        let msg_area = Rect {
            x: inner.x,
            y: inner.y + 1,
            width: inner.width,
            height: 2,
        };
        let message = Paragraph::new(self.message)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });
        message.render(msg_area, buf);

        // Options
        let options_area = Rect {
            x: inner.x,
            y: inner.y + 3,
            width: inner.width,
            height: 1,
        };
        let options = Paragraph::new("Y: Yes | N: No")
            .style(Style::default().fg(Color::Yellow));
        options.render(options_area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tui_input::Input;

    #[test]
    fn test_input_dialog_cursor_position() {
        // Create test inputs
        let title_input = Input::new("Test Title".to_string());
        let username_input = Input::new("testuser".to_string());
        let password_input = Input::new("password123".to_string());
        let url_input = Input::new("https://example.com".to_string());
        let notes_input = Input::new("Some notes".to_string());

        // Create dialog with active field = 0 (title)
        let dialog = InputDialog {
            title: "Test Dialog",
            title_input: &title_input,
            username_input: &username_input,
            password_input: &password_input,
            url_input: &url_input,
            notes_input: &notes_input,
            active_field: 0,
        };

        // Create a test area
        let area = Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 24,
        };

        // Get cursor position
        let cursor_pos = dialog.cursor_position(area);
        
        // Cursor should be present
        assert!(cursor_pos.is_some());
        
        let (x, y) = cursor_pos.unwrap();
        
        // Cursor should be within the area bounds
        assert!(x < area.width);
        assert!(y < area.height);
        
        // Cursor x should be at least at the start of the dialog (accounting for centering and borders)
        assert!(x > 0);
    }

    #[test]
    fn test_input_dialog_cursor_position_different_fields() {
        let title_input = Input::new("".to_string());
        let username_input = Input::new("".to_string());
        let password_input = Input::new("".to_string());
        let url_input = Input::new("".to_string());
        let notes_input = Input::new("".to_string());

        let area = Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 24,
        };

        // Test each field
        for active_field in 0..5 {
            let dialog = InputDialog {
                title: "Test",
                title_input: &title_input,
                username_input: &username_input,
                password_input: &password_input,
                url_input: &url_input,
                notes_input: &notes_input,
                active_field,
            };

            let cursor_pos = dialog.cursor_position(area);
            assert!(cursor_pos.is_some(), "Cursor should be present for field {}", active_field);
        }
    }
}
