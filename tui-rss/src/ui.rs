use crate::feed_manager::{Article, Feed};
use chrono::Local;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Widget, Wrap},
};
use tui_input::Input;

const ARTICLE_PREVIEW_MAX_LENGTH: usize = 60;

pub struct ArticleList;

impl ArticleList {
    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        articles: &[Article],
        selected_index: Option<usize>,
        is_focused: bool,
        _scroll_offset: usize,
    ) {
        let border_color = if is_focused { Color::Yellow } else { Color::Cyan };
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Articles ")
            .style(Style::default().fg(border_color));

        let inner_area = block.inner(area);
        block.render(area, buf);

        if articles.is_empty() {
            let text = Text::from(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "No articles yet",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Press 'n' to add a feed",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(
                    "Press 'r' to refresh feeds",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(
                    "Press '?' for help",
                    Style::default().fg(Color::DarkGray),
                )),
            ]);
            let paragraph = Paragraph::new(text).centered();
            paragraph.render(inner_area, buf);
            return;
        }

        let items: Vec<ListItem> = articles
            .iter()
            .enumerate()
            .map(|(idx, article)| {
                let time_str = article
                    .published
                    .map(|t| t.with_timezone(&Local).format("%m/%d %H:%M").to_string())
                    .unwrap_or_else(|| "No date".to_string());

                let description = if article.description.chars().count() > ARTICLE_PREVIEW_MAX_LENGTH {
                    format!("{}...", article.description.chars().take(ARTICLE_PREVIEW_MAX_LENGTH).collect::<String>())
                } else {
                    article.description.clone()
                };

                let read_indicator = if article.read { " " } else { "● " };
                let title_color = if article.read { Color::Gray } else { Color::White };

                let lines = vec![
                    Line::from(vec![
                        Span::styled(
                            read_indicator,
                            Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            &article.title,
                            Style::default()
                                .fg(title_color)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            format!("  {} | {}", time_str, description),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]),
                ];

                let style = if Some(idx) == selected_index {
                    Style::default()
                        .bg(Color::DarkGray)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(lines).style(style)
            })
            .collect();

        let list = List::new(items);
        Widget::render(list, inner_area, buf);
    }
}

pub struct ArticleReader;

impl ArticleReader {
    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        article: Option<&Article>,
        scroll_offset: usize,
        is_focused: bool,
    ) {
        let border_color = if is_focused { Color::Yellow } else { Color::Cyan };
        let block = Block::default()
            .borders(Borders::ALL)
            .title(
                article
                    .map(|a| format!(" {} ", a.title))
                    .unwrap_or_else(|| " Reader ".to_string()),
            )
            .style(Style::default().fg(border_color));

        let inner_area = block.inner(area);
        block.render(area, buf);

        if let Some(article) = article {
            let time_str = article
                .published
                .map(|t| t.with_timezone(&Local).format("%B %d, %Y at %H:%M").to_string())
                .unwrap_or_else(|| "No date".to_string());

            let mut lines = vec![
                Line::from(""),
                Line::from(Span::styled(
                    &article.title,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    time_str,
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    format!("Link: {}", article.link),
                    Style::default().fg(Color::Blue),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "─".repeat(inner_area.width as usize),
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
            ];

            // Add content lines with proper centering
            let max_width = (inner_area.width as usize).min(80);
            let padding = if inner_area.width > 80 {
                (inner_area.width as usize - 80) / 2
            } else {
                0
            };

            for content_line in article.content.lines() {
                let trimmed = content_line.trim();
                if trimmed.is_empty() {
                    lines.push(Line::from(""));
                    continue;
                }

                let chars: Vec<char> = trimmed.chars().collect();
                let mut start = 0;
                while start < chars.len() {
                    let end = (start + max_width).min(chars.len());
                    let chunk: String = chars[start..end].iter().collect();
                    
                    let padded_chunk = if padding > 0 {
                        format!("{}{}", " ".repeat(padding), chunk)
                    } else {
                        chunk
                    };
                    
                    lines.push(Line::from(padded_chunk));
                    start = end;
                }
            }

            // Apply scroll
            let visible_lines: Vec<Line> = lines.into_iter().skip(scroll_offset).collect();

            let paragraph = Paragraph::new(visible_lines).wrap(Wrap { trim: false });
            paragraph.render(inner_area, buf);
        } else {
            let text = Text::from(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "No article selected",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Select an article from the list",
                    Style::default().fg(Color::DarkGray),
                )),
            ]);
            let paragraph = Paragraph::new(text).centered();
            paragraph.render(inner_area, buf);
        }
    }
}

pub struct StatusBar;

impl StatusBar {
    pub fn render(area: Rect, buf: &mut Buffer, message: &str) {
        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));

        let inner_area = block.inner(area);
        block.render(area, buf);

        let shortcuts = if message.is_empty() {
            "q:Quit | n:New Feed | r:Refresh | /:Search | m:Manage Feeds | f:Focus Mode | ?:Help"
        } else {
            message
        };

        let text = Paragraph::new(shortcuts)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left);
        text.render(inner_area, buf);
    }
}

pub struct SearchModal;

impl SearchModal {
    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        title: &str,
        prompt: &str,
        input: &Input,
    ) {
        // Create a centered modal
        let modal_width = area.width.min(60);
        let modal_height = 7;
        let modal_x = (area.width.saturating_sub(modal_width)) / 2;
        let modal_y = (area.height.saturating_sub(modal_height)) / 2;

        let modal_area = Rect {
            x: area.x + modal_x,
            y: area.y + modal_y,
            width: modal_width,
            height: modal_height,
        };

        // Clear the modal area
        Clear.render(modal_area, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", title))
            .style(Style::default().fg(Color::Yellow));

        let inner_area = block.inner(modal_area);
        block.render(modal_area, buf);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(inner_area);

        let prompt_text = Paragraph::new(prompt)
            .style(Style::default().fg(Color::White));
        prompt_text.render(chunks[0], buf);

        // Render input with cursor
        let input_value = input.value();
        let cursor_pos = input.cursor();

        let input_text = Paragraph::new(input_value)
            .style(Style::default().fg(Color::White).bg(Color::DarkGray));
        input_text.render(chunks[2], buf);

        // Draw cursor
        if cursor_pos <= input_value.len() && chunks[2].width > 0 {
            let cursor_x = chunks[2].x + (cursor_pos as u16).min(chunks[2].width - 1);
            let cursor_y = chunks[2].y;
            if let Some(cell) = buf.cell_mut((cursor_x, cursor_y)) {
                cell.set_style(Style::default().bg(Color::White).fg(Color::Black));
            }
        }

        let help_text = Paragraph::new("Press Enter to confirm, Esc to cancel")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        help_text.render(chunks[3], buf);
    }
}

pub struct FeedList;

impl FeedList {
    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        feeds: &[Feed],
        selected_index: Option<usize>,
    ) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Feed Management ")
            .style(Style::default().fg(Color::Yellow));

        let inner_area = block.inner(area);
        block.render(area, buf);

        if feeds.is_empty() {
            let text = Text::from(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "No feeds added yet",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Press Esc to return",
                    Style::default().fg(Color::DarkGray),
                )),
            ]);
            let paragraph = Paragraph::new(text).centered();
            paragraph.render(inner_area, buf);
            return;
        }

        let items: Vec<ListItem> = feeds
            .iter()
            .enumerate()
            .map(|(idx, feed)| {
                let last_updated = feed
                    .last_updated
                    .map(|t| t.with_timezone(&Local).format("%m/%d %H:%M").to_string())
                    .unwrap_or_else(|| "Never".to_string());

                let lines = vec![
                    Line::from(vec![
                        Span::styled(
                            &feed.title,
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            format!("  URL: {}", feed.url),
                            Style::default().fg(Color::Gray),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            format!("  Last updated: {}", last_updated),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]),
                    Line::from(""),
                ];

                let style = if Some(idx) == selected_index {
                    Style::default()
                        .bg(Color::DarkGray)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(lines).style(style)
            })
            .collect();

        let list = List::new(items);
        Widget::render(list, inner_area, buf);

        // Add help text at the bottom
        let help_y = inner_area.bottom().saturating_sub(3);
        if help_y > inner_area.y {
            let help_area = Rect {
                x: inner_area.x,
                y: help_y,
                width: inner_area.width,
                height: 3,
            };

            let help_text = Text::from(vec![
                Line::from(Span::styled(
                    "─".repeat(inner_area.width as usize),
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(
                    "↑/↓: Navigate | d: Delete | r: Refresh | Esc: Back",
                    Style::default().fg(Color::Yellow),
                )),
            ]);
            let paragraph = Paragraph::new(help_text).alignment(Alignment::Center);
            paragraph.render(help_area, buf);
        }
    }
}

pub struct HelpOverlay;

impl HelpOverlay {
    pub fn render(area: Rect, buf: &mut Buffer) {
        // Create a centered help window
        let help_width = area.width.min(70);
        let help_height = area.height.min(30);
        let help_x = (area.width.saturating_sub(help_width)) / 2;
        let help_y = (area.height.saturating_sub(help_height)) / 2;

        let help_area = Rect {
            x: area.x + help_x,
            y: area.y + help_y,
            width: help_width,
            height: help_height,
        };

        Clear.render(help_area, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Help ")
            .style(Style::default().fg(Color::Yellow));

        let inner_area = block.inner(help_area);
        block.render(help_area, buf);

        let help_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "TUI RSS Reader - Keyboard Shortcuts",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Navigation",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from("  ↑/↓ or j/k       Navigate articles"),
            Line::from("  Enter            Select article / Open reader"),
            Line::from("  Tab              Switch between panels"),
            Line::from("  Esc              Return to article list"),
            Line::from(""),
            Line::from(Span::styled(
                "Article Reader",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from("  ↑/↓ or j/k       Scroll content"),
            Line::from("  PageUp/PageDown  Scroll by page"),
            Line::from("  n                Next article"),
            Line::from("  p                Previous article"),
            Line::from("  t                Toggle read/unread status"),
            Line::from(""),
            Line::from(Span::styled(
                "Features",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from("  n                Add new RSS feed"),
            Line::from("  r                Refresh all feeds"),
            Line::from("  f                Toggle focus mode (hide sidebar)"),
            Line::from("  /                Search articles"),
            Line::from("  m                Manage feeds"),
            Line::from("  u                Toggle show unread only"),
            Line::from("  ?                Show this help"),
            Line::from("  q                Quit"),
            Line::from(""),
            Line::from(Span::styled(
                "Press Esc or ? to close this help",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let paragraph = Paragraph::new(help_text).alignment(Alignment::Left);
        paragraph.render(inner_area, buf);
    }
}
