use crate::epub_parser::TocEntry;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render_toc(
    frame: &mut Frame,
    area: Rect,
    toc: &[TocEntry],
    selected_index: usize,
    scroll_offset: usize,
    focused: bool,
) {
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Gray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title("Table of Contents");

    let inner_area = block.inner(area);
    let visible_height = inner_area.height as usize;

    // Calculate which items to display
    let start_index = scroll_offset;
    let end_index = (start_index + visible_height).min(toc.len());

    let items: Vec<ListItem> = toc[start_index..end_index]
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let actual_index = start_index + i;
            let style = if actual_index == selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let prefix = if actual_index == selected_index { "> " } else { "  " };
            ListItem::new(format!("{}{}", prefix, entry.title)).style(style)
        })
        .collect();

    let list = List::new(items).block(block);

    frame.render_widget(list, area);
}

pub fn render_book_content(
    frame: &mut Frame,
    area: Rect,
    content: &str,
    scroll_offset: usize,
    focused: bool,
) -> usize {
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Gray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title("Book Content");

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    // Calculate centered content area (60% width centered)
    let content_width = inner_area.width * 60 / 100;
    let margin = (inner_area.width - content_width) / 2;

    let centered_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(margin),
            Constraint::Length(content_width),
            Constraint::Length(margin),
        ])
        .split(inner_area);

    let content_area = centered_chunks[1];

    // Wrap text to fit the content area
    let wrapped_lines = wrap_text(content, content_area.width as usize);
    
    // Calculate visible lines
    let visible_height = content_area.height as usize;
    let total_lines = wrapped_lines.len();
    let max_scroll = if total_lines > visible_height {
        total_lines - visible_height
    } else {
        0
    };

    let start_line = scroll_offset.min(max_scroll);
    let end_line = (start_line + visible_height).min(total_lines);

    let visible_lines: Vec<Line> = wrapped_lines[start_line..end_line]
        .iter()
        .map(|line| Line::from(line.clone()))
        .collect();

    let paragraph = Paragraph::new(visible_lines).alignment(Alignment::Left);

    frame.render_widget(paragraph, content_area);

    max_scroll
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut wrapped = Vec::new();
    
    for paragraph in text.split('\n') {
        if paragraph.is_empty() {
            wrapped.push(String::new());
            continue;
        }

        let mut current_line = String::new();
        let mut current_length = 0;

        for word in paragraph.split_whitespace() {
            let word_len = word.chars().count();
            
            if current_length == 0 {
                // First word in line
                current_line = word.to_string();
                current_length = word_len;
            } else if current_length + 1 + word_len <= width {
                // Word fits in current line
                current_line.push(' ');
                current_line.push_str(word);
                current_length += 1 + word_len;
            } else {
                // Need to wrap to next line
                wrapped.push(current_line);
                current_line = word.to_string();
                current_length = word_len;
            }
        }

        if !current_line.is_empty() {
            wrapped.push(current_line);
        }
    }

    wrapped
}
