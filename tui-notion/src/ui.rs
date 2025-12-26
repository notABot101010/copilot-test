use crate::editor::Editor;
use crate::search::SearchDialog;
use crate::toc::TableOfContents;
use crate::tree::DocumentTree;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Widget, Wrap},
};

pub fn render_tree(area: Rect, buf: &mut Buffer, tree: &DocumentTree, focused: bool) {
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let block = Block::default()
        .title("Documents")
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    block.render(area, buf);

    let documents = tree.documents();
    let items: Vec<ListItem> = documents
        .iter()
        .enumerate()
        .map(|(idx, doc)| {
            let style = if Some(idx) == tree.selected_index() {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let content = format!("📄 {}", doc.title);
            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items);
    list.render(inner, buf);
}

pub fn render_editor(
    area: Rect,
    buf: &mut Buffer,
    editor: &Editor,
    focused: bool,
    mode: &str,
) {
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let block = Block::default()
        .title(format!("Editor [{}]", mode))
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    block.render(area, buf);

    let lines = editor.lines();
    let scroll_offset = editor.scroll_offset();
    let (cursor_line, cursor_col) = editor.cursor_position();

    let visible_height = inner.height as usize;
    let visible_lines: Vec<Line> = lines
        .iter()
        .skip(scroll_offset)
        .take(visible_height)
        .enumerate()
        .map(|(idx, line)| {
            let actual_line = scroll_offset + idx;
            let styled_line = highlight_markdown(line, actual_line == cursor_line);
            styled_line
        })
        .collect();

    let paragraph = Paragraph::new(visible_lines).wrap(Wrap { trim: false });
    paragraph.render(inner, buf);

    // Show cursor position indicator at the bottom
    if focused {
        let cursor_info = format!("Ln {}, Col {}", cursor_line + 1, cursor_col + 1);
        let cursor_area = Rect {
            x: inner.x,
            y: inner.y + inner.height.saturating_sub(1),
            width: inner.width,
            height: 1,
        };
        let cursor_para = Paragraph::new(cursor_info)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Right);
        cursor_para.render(cursor_area, buf);
    }
}

pub fn render_toc(area: Rect, buf: &mut Buffer, toc: &TableOfContents, focused: bool) {
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let block = Block::default()
        .title("Outline")
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    block.render(area, buf);

    let entries = toc.entries();
    let items: Vec<ListItem> = entries
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let style = if Some(idx) == toc.selected_index() {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let indent = "  ".repeat(entry.level.saturating_sub(1));
            let content = format!("{}{}", indent, entry.title);
            ListItem::new(content).style(style)
        })
        .collect();

    if items.is_empty() {
        let empty_msg = Paragraph::new("No headings found")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        empty_msg.render(inner, buf);
    } else {
        let list = List::new(items);
        list.render(inner, buf);
    }
}

pub fn render_search_dialog(area: Rect, buf: &mut Buffer, search: &SearchDialog, tree: &DocumentTree) {
    // Create a centered dialog
    let dialog_width = area.width.min(60);
    let dialog_height = area.height.min(20);
    let dialog_x = (area.width - dialog_width) / 2;
    let dialog_y = (area.height - dialog_height) / 2;

    let dialog_area = Rect {
        x: area.x + dialog_x,
        y: area.y + dialog_y,
        width: dialog_width,
        height: dialog_height,
    };

    // Clear the area behind the dialog
    Clear.render(dialog_area, buf);

    let block = Block::default()
        .title("Search Documents (Ctrl+K)")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(dialog_area);
    block.render(dialog_area, buf);

    // Split into search input and results
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(inner);

    // Render search input
    let input_block = Block::default()
        .title("Query")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    let input_inner = input_block.inner(chunks[0]);
    input_block.render(chunks[0], buf);

    let input_text = Paragraph::new(search.query())
        .style(Style::default().fg(Color::White));
    input_text.render(input_inner, buf);

    // Render results (placeholder - needs tree reference to show document titles)
    let results_block = Block::default()
        .title(format!("Results ({})", search.results().len()))
        .borders(Borders::ALL);
    let results_inner = results_block.inner(chunks[1]);
    results_block.render(chunks[1], buf);

    let results: Vec<ListItem> = search
        .results()
        .iter()
        .enumerate()
        .map(|(idx, doc_id)| {
            let style = if Some(idx) == search.selected_index() {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let title = tree
                .get_document(*doc_id)
                .map(|doc| doc.title.clone())
                .unwrap_or_else(|| "Unknown".to_string());
            let content = format!("📄 {}", title);
            ListItem::new(content).style(style)
        })
        .collect();

    if results.is_empty() && !search.query().is_empty() {
        let empty_msg = Paragraph::new("No matches found")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        empty_msg.render(results_inner, buf);
    } else if !results.is_empty() {
        let list = List::new(results);
        list.render(results_inner, buf);
    }
}

fn highlight_markdown(line: &str, is_cursor_line: bool) -> Line<'_> {
    let base_style = if is_cursor_line {
        Style::default().bg(Color::DarkGray)
    } else {
        Style::default()
    };

    // Simple markdown highlighting
    if line.trim_start().starts_with('#') {
        let level = line.trim_start().chars().take_while(|&c| c == '#').count();
        let color = match level {
            1 => Color::LightBlue,
            2 => Color::LightCyan,
            3 => Color::LightGreen,
            _ => Color::Green,
        };
        Line::from(Span::styled(
            line.to_string(),
            base_style.fg(color).add_modifier(Modifier::BOLD),
        ))
    } else if line.trim_start().starts_with("```") {
        Line::from(Span::styled(
            line.to_string(),
            base_style.fg(Color::Yellow),
        ))
    } else if line.trim_start().starts_with("- ") || line.trim_start().starts_with("* ") {
        Line::from(Span::styled(line.to_string(), base_style.fg(Color::Cyan)))
    } else {
        Line::from(Span::styled(line.to_string(), base_style))
    }
}
