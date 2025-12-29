use crate::models::Tab;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

pub struct TabBar;

impl TabBar {
    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        tabs: &[Tab],
        selected_index: usize,
        is_focused: bool,
    ) {
        let border_style = if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Tabs (Ctrl+T: New | ←/→: Switch) ")
            .style(border_style);

        let inner_area = block.inner(area);
        block.render(area, buf);

        if tabs.is_empty() {
            return;
        }

        let mut x_offset = inner_area.x;
        let y = inner_area.y;

        for (idx, tab) in tabs.iter().enumerate() {
            let is_selected = idx == selected_index;
            
            let tab_style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            let loading_indicator = if tab.loading { " ⟳" } else { "" };
            let tab_text = format!(" {} {}{} ", idx + 1, tab.title, loading_indicator);
            let tab_width = tab_text.len() as u16;

            if x_offset + tab_width > inner_area.x + inner_area.width {
                break;
            }

            let tab_span = Span::styled(tab_text, tab_style);
            
            // Draw the tab
            for (i, ch) in tab_span.content.chars().enumerate() {
                if x_offset + i as u16 >= inner_area.x + inner_area.width {
                    break;
                }
                if let Some(cell) = buf.cell_mut((x_offset + i as u16, y)) {
                    cell.set_char(ch);
                    cell.set_style(tab_style);
                }
            }

            x_offset += tab_width + 1;
        }
    }
}

pub struct UrlBar;

impl UrlBar {
    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        url: &str,
        cursor_position: usize,
        is_focused: bool,
    ) {
        let border_style = if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" URL Bar (Enter: Navigate) ")
            .style(border_style);

        let inner_area = block.inner(area);
        block.render(area, buf);

        let display_text = if url.is_empty() {
            "Enter URL...".to_string()
        } else {
            url.to_string()
        };

        let text_style = if url.is_empty() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::White)
        };

        // Draw the URL text
        for (i, ch) in display_text.chars().enumerate() {
            if i >= inner_area.width as usize {
                break;
            }
            if let Some(cell) = buf.cell_mut((inner_area.x + i as u16, inner_area.y)) {
                cell.set_char(ch);
                cell.set_style(text_style);
            }
        }

        // Draw cursor if focused
        if is_focused && cursor_position < inner_area.width as usize {
            let cursor_x = inner_area.x + cursor_position as u16;
            if cursor_x < inner_area.x + inner_area.width {
                if let Some(cell) = buf.cell_mut((cursor_x, inner_area.y)) {
                    cell.set_style(
                        Style::default()
                            .bg(Color::White)
                            .fg(Color::Black)
                    );
                }
            }
        }
    }
}

pub struct ContentArea;

impl ContentArea {
    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        content: &str,
        scroll_offset: usize,
        is_focused: bool,
        is_loading: bool,
        links: &[crate::Link],
        width_percent: f32,
    ) -> usize {
        let border_style = if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Content (↑/↓: Scroll | PgUp/PgDn: Page | Type number+Enter: Navigate link) ")
            .style(border_style);

        let inner_area = block.inner(area);
        block.render(area, buf);

        // Calculate centered content area with dynamic width and margins
        let content_width = (inner_area.width as f32 * width_percent) as u16;
        let margin_width = (inner_area.width as f32 * (1.0 - width_percent) / 2.0) as u16;
        
        let centered_area = Rect {
            x: inner_area.x + margin_width,
            y: inner_area.y,
            width: content_width,
            height: inner_area.height,
        };

        // Show loading indicator
        if is_loading {
            let loading_msg = "Loading page, please wait...";
            let paragraph = Paragraph::new(loading_msg)
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
            
            paragraph.render(centered_area, buf);
            return 0;
        }

        if content.is_empty() {
            let empty_msg = "No content loaded. Enter a URL and press Enter to navigate.";
            let paragraph = Paragraph::new(empty_msg)
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
            
            paragraph.render(centered_area, buf);
            return 0;
        }

        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();
        let visible_height = centered_area.height as usize;

        let start_line = scroll_offset.min(total_lines.saturating_sub(1));
        let end_line = (start_line + visible_height).min(total_lines);

        // Build a map of line indices to link numbers for quick lookup
        let mut line_to_link: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
        for (link_idx, link) in links.iter().enumerate() {
            line_to_link.entry(link.line_index)
                .or_insert_with(Vec::new)
                .push(link_idx + 1);
        }

        for (i, line) in lines[start_line..end_line].iter().enumerate() {
            let y = centered_area.y + i as u16;
            let absolute_line_index = start_line + i;
            
            let mut x_offset = 0;
            
            // Check if this line has any links and prepend link numbers
            if let Some(link_numbers) = line_to_link.get(&absolute_line_index) {
                // Display link numbers at the start of the line
                let link_label = if link_numbers.len() == 1 {
                    format!("[{}] ", link_numbers[0])
                } else {
                    // Format multiple link numbers efficiently
                    let mut label = String::from("[");
                    for (idx, num) in link_numbers.iter().enumerate() {
                        if idx > 0 {
                            label.push(',');
                        }
                        label.push_str(&num.to_string());
                    }
                    label.push_str("] ");
                    label
                };
                
                let link_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
                
                for (j, ch) in link_label.chars().enumerate() {
                    if x_offset + j >= centered_area.width as usize {
                        break;
                    }
                    if let Some(cell) = buf.cell_mut((centered_area.x + (x_offset + j) as u16, y)) {
                        cell.set_char(ch);
                        cell.set_style(link_style);
                    }
                }
                x_offset += link_label.len();
            }
            
            // Display the line content
            let line_style = Style::default().fg(Color::White);
            for (j, ch) in line.chars().enumerate() {
                if x_offset + j >= centered_area.width as usize {
                    break;
                }
                if let Some(cell) = buf.cell_mut((centered_area.x + (x_offset + j) as u16, y)) {
                    cell.set_char(ch);
                    cell.set_style(line_style);
                }
            }
        }

        total_lines
    }
}

pub struct StatusBar;

impl StatusBar {
    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        status_text: &str,
        help_text: &str,
    ) {
        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));

        let inner_area = block.inner(area);
        block.render(area, buf);

        // Status text on the left
        for (i, ch) in status_text.chars().enumerate() {
            if i >= inner_area.width as usize / 2 {
                break;
            }
            if let Some(cell) = buf.cell_mut((inner_area.x + i as u16, inner_area.y)) {
                cell.set_char(ch);
                cell.set_style(Style::default().fg(Color::Cyan));
            }
        }

        // Help text on the right
        let help_x_start = inner_area.x + inner_area.width.saturating_sub(help_text.len() as u16);
        for (i, ch) in help_text.chars().enumerate() {
            let x = help_x_start + i as u16;
            if x >= inner_area.x + inner_area.width {
                break;
            }
            if let Some(cell) = buf.cell_mut((x, inner_area.y)) {
                cell.set_char(ch);
                cell.set_style(Style::default().fg(Color::DarkGray));
            }
        }
    }
}

pub struct HelpDialog;

impl HelpDialog {
    pub fn render(area: Rect, buf: &mut Buffer, scroll_offset: usize) {
        // Create a centered dialog
        let dialog_width = 60.min(area.width.saturating_sub(4));
        let dialog_height = 20.min(area.height.saturating_sub(4));
        
        let dialog_x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
        let dialog_y = area.y + (area.height.saturating_sub(dialog_height)) / 2;
        
        let dialog_area = Rect {
            x: dialog_x,
            y: dialog_y,
            width: dialog_width,
            height: dialog_height,
        };

        // Draw background
        for y in dialog_area.y..dialog_area.y + dialog_area.height {
            for x in dialog_area.x..dialog_area.x + dialog_area.width {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_style(Style::default().bg(Color::Black).fg(Color::White));
                    cell.set_char(' ');
                }
            }
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Keyboard Shortcuts (↑/↓: Scroll | Esc: Close) ")
            .style(Style::default().fg(Color::Cyan).bg(Color::Black));

        let inner_area = block.inner(dialog_area);
        block.render(dialog_area, buf);

        let help_text = vec![
            "Navigation:",
            "  Tab          - Cycle between panels",
            "  Ctrl+T       - Open new tab",
            "  Ctrl+W       - Close current tab",
            "  ←/→ (tabs)   - Switch between tabs",
            "",
            "URL Bar:",
            "  Enter        - Navigate to URL",
            "  Ctrl+L       - Focus URL bar",
            "  Ctrl+R       - Refresh current page",
            "",
            "Content:",
            "  ↑/↓ or j/k   - Scroll line by line",
            "  PgUp/PgDn    - Scroll page by page",
            "  0-9          - Type link number",
            "  Enter        - Navigate to typed link number",
            "  Ctrl+Enter   - Open typed link in new tab",
            "  Backspace    - Clear link number or go back",
            "  Esc          - Clear link number",
            "  Ctrl+←       - Go back in history",
            "  Ctrl+→       - Go forward in history",
            "  +/-          - Zoom in/out (adjust text width)",
            "",
            "General:",
            "  Ctrl+H       - Show this help",
            "  Ctrl+Q or q  - Quit browser",
        ];

        let visible_height = inner_area.height as usize;
        let total_lines = help_text.len();
        let start_line = scroll_offset.min(total_lines.saturating_sub(visible_height));
        let end_line = (start_line + visible_height).min(total_lines);

        for (i, line) in help_text[start_line..end_line].iter().enumerate() {
            let y = inner_area.y + i as u16;
            let style = if line.ends_with(':') {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            for (j, ch) in line.chars().enumerate() {
                if j >= inner_area.width as usize {
                    break;
                }
                if let Some(cell) = buf.cell_mut((inner_area.x + j as u16, y)) {
                    cell.set_char(ch);
                    cell.set_style(style);
                }
            }
        }
    }
}
