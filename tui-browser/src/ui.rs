use crate::models::{Bookmark, Tab};
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

pub struct FavoritesBar;

impl FavoritesBar {
    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        bookmarks: &[Bookmark],
        selected_index: Option<usize>,
        is_focused: bool,
    ) {
        let border_style = if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Favorites (Ctrl+F: Add | ←/→: Navigate | Enter: Open) ")
            .style(border_style);

        let inner_area = block.inner(area);
        block.render(area, buf);

        if bookmarks.is_empty() {
            let empty_msg = "No bookmarks yet. Press Ctrl+F to add current page.";
            let empty_span = Span::styled(empty_msg, Style::default().fg(Color::DarkGray));
            
            for (i, ch) in empty_span.content.chars().enumerate() {
                if i >= inner_area.width as usize {
                    break;
                }
                if let Some(cell) = buf.cell_mut((inner_area.x + i as u16, inner_area.y)) {
                    cell.set_char(ch);
                    cell.set_style(Style::default().fg(Color::DarkGray));
                }
            }
            return;
        }

        let mut x_offset = inner_area.x;
        let y = inner_area.y;

        for (idx, bookmark) in bookmarks.iter().enumerate() {
            let is_selected = Some(idx) == selected_index;
            
            let bookmark_style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Cyan)
            };

            let bookmark_text = format!(" ★ {} ", bookmark.title);
            let bookmark_width = bookmark_text.len() as u16;

            if x_offset + bookmark_width > inner_area.x + inner_area.width {
                break;
            }

            // Draw the bookmark
            for (i, ch) in bookmark_text.chars().enumerate() {
                if x_offset + i as u16 >= inner_area.x + inner_area.width {
                    break;
                }
                if let Some(cell) = buf.cell_mut((x_offset + i as u16, y)) {
                    cell.set_char(ch);
                    cell.set_style(bookmark_style);
                }
            }

            x_offset += bookmark_width + 2;
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

        // Show loading indicator
        if is_loading {
            let loading_msg = "Loading page, please wait...";
            let paragraph = Paragraph::new(loading_msg)
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
            
            paragraph.render(inner_area, buf);
            return 0;
        }

        if content.is_empty() {
            let empty_msg = "No content loaded. Enter a URL and press Enter to navigate.";
            let paragraph = Paragraph::new(empty_msg)
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
            
            paragraph.render(inner_area, buf);
            return 0;
        }

        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();
        let visible_height = inner_area.height as usize;

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
            let y = inner_area.y + i as u16;
            let absolute_line_index = start_line + i;
            
            let mut x_offset = 0;
            
            // Check if this line has any links and prepend link numbers
            if let Some(link_numbers) = line_to_link.get(&absolute_line_index) {
                // Display link numbers at the start of the line
                let link_label = if link_numbers.len() == 1 {
                    format!("[{}] ", link_numbers[0])
                } else {
                    format!("[{}] ", link_numbers.iter()
                        .map(|n| n.to_string())
                        .collect::<Vec<_>>()
                        .join(","))
                };
                
                let link_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
                
                for (j, ch) in link_label.chars().enumerate() {
                    if x_offset + j >= inner_area.width as usize {
                        break;
                    }
                    if let Some(cell) = buf.cell_mut((inner_area.x + (x_offset + j) as u16, y)) {
                        cell.set_char(ch);
                        cell.set_style(link_style);
                    }
                }
                x_offset += link_label.len();
            }
            
            // Display the line content
            let line_style = Style::default().fg(Color::White);
            for (j, ch) in line.chars().enumerate() {
                if x_offset + j >= inner_area.width as usize {
                    break;
                }
                if let Some(cell) = buf.cell_mut((inner_area.x + (x_offset + j) as u16, y)) {
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
            "",
            "Favorites:",
            "  Ctrl+F       - Add current page to favorites",
            "  ←/→ (favs)   - Navigate favorites",
            "  Enter        - Open selected favorite",
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
