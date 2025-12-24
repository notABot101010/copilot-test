use crate::mock_data::Conversation;
use chrono::Local;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Widget, Wrap},
};

const MESSAGE_PREVIEW_MAX_LENGTH: usize = 30;

pub struct ConversationList;

impl ConversationList {
    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        conversations: &[Conversation],
        selected_index: Option<usize>,
    ) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Conversations ")
            .style(Style::default().fg(Color::Cyan));

        let inner_area = block.inner(area);
        block.render(area, buf);

        let items: Vec<ListItem> = conversations
            .iter()
            .enumerate()
            .map(|(idx, conv)| {
                let time_str = conv
                    .last_message_time
                    .map(|t| t.with_timezone(&Local).format("%H:%M").to_string())
                    .unwrap_or_else(String::new);

                let last_msg = conv
                    .last_message
                    .as_ref()
                    .map(|m| {
                        if m.chars().count() > MESSAGE_PREVIEW_MAX_LENGTH {
                            format!("{}...", m.chars().take(MESSAGE_PREVIEW_MAX_LENGTH).collect::<String>())
                        } else {
                            m.to_string()
                        }
                    })
                    .unwrap_or_else(|| "No messages".to_string());

                let mut lines = vec![Line::from(vec![
                    Span::raw(format!("{} ", conv.avatar)),
                    Span::styled(
                        &conv.name,
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" ".repeat(
                        inner_area
                            .width
                            .saturating_sub(conv.name.len() as u16 + 4 + time_str.len() as u16)
                            as usize,
                    )),
                    Span::styled(time_str, Style::default().fg(Color::DarkGray)),
                ])];

                let unread_indicator = if conv.unread_count > 0 {
                    format!(" [{}]", conv.unread_count)
                } else {
                    "".to_string()
                };

                lines.push(Line::from(vec![
                    Span::styled(
                        format!("  {}{}", last_msg, unread_indicator),
                        Style::default().fg(Color::Gray),
                    ),
                ]));

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

pub struct MessageView;

impl MessageView {
    pub fn render(area: Rect, buf: &mut Buffer, conversation: Option<&Conversation>, scroll_offset: usize) -> usize {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(
                conversation
                    .map(|c| format!(" {} {} ", c.avatar, c.name))
                    .unwrap_or_else(|| " Chat ".to_string()),
            )
            .style(Style::default().fg(Color::Cyan));

        let inner_area = block.inner(area);
        block.render(area, buf);

        if let Some(conv) = conversation {
            let messages: Vec<Line> = conv
                .messages
                .iter()
                .flat_map(|msg| {
                    let time_str = msg.timestamp.with_timezone(&Local).format("%H:%M").to_string();
                    let style = if msg.is_own {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Yellow)
                    };

                    let header = Line::from(vec![
                        Span::styled(
                            format!("{} ", msg.sender),
                            style.add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(format!("[{}]", time_str), Style::default().fg(Color::DarkGray)),
                    ]);

                    let mut result = vec![header];
                    
                    // Wrap message content
                    let max_width = inner_area.width.saturating_sub(2) as usize;
                    for line in msg.content.lines() {
                        let char_count = line.chars().count();
                        if char_count > max_width {
                            let chars: Vec<char> = line.chars().collect();
                            let mut start = 0;
                            while start < chars.len() {
                                let end = (start + max_width).min(chars.len());
                                let chunk: String = chars[start..end].iter().collect();
                                result.push(Line::from(Span::raw(format!("  {}", chunk))));
                                start = end;
                            }
                        } else {
                            result.push(Line::from(Span::raw(format!("  {}", line))));
                        }
                    }
                    
                    result.push(Line::from(""));
                    result
                })
                .collect();

            // Apply scroll offset with bounds checking
            let total_lines = messages.len();
            let max_scroll = total_lines.saturating_sub(inner_area.height as usize);
            let capped_offset = scroll_offset.min(max_scroll);
            
            let visible_messages: Vec<Line> = messages
                .into_iter()
                .skip(capped_offset)
                .collect();

            let paragraph = Paragraph::new(visible_messages).wrap(Wrap { trim: false });
            paragraph.render(inner_area, buf);
            
            // Return total lines so caller can calculate scroll position
            total_lines
        } else {
            let text = Text::from(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "No conversation selected",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Press ↑/↓ or j/k to navigate conversations",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(
                    "Press Enter to select a conversation",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(
                    "Click on a conversation to select it",
                    Style::default().fg(Color::DarkGray),
                )),
            ]);
            let paragraph = Paragraph::new(text).centered();
            paragraph.render(inner_area, buf);
            0
        }
    }
}

pub struct InputBox;

impl InputBox {
    pub fn render(area: Rect, buf: &mut Buffer, content: &str, focused: bool) {
        let title = if focused {
            " Type your message (Enter to send, Shift+Enter for new line, Esc to cancel) "
        } else {
            " Press Enter to type a message "
        };

        let border_color = if focused { Color::Yellow } else { Color::Cyan };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .style(Style::default().fg(border_color));

        let inner_area = block.inner(area);
        block.render(area, buf);

        let text = if content.is_empty() && !focused {
            Text::from(Span::styled(
                "Type a message...",
                Style::default().fg(Color::DarkGray),
            ))
        } else {
            Text::from(content)
        };

        let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
        paragraph.render(inner_area, buf);

        // Show cursor when focused
        if focused && inner_area.width > 0 && inner_area.height > 0 {
            let cursor_line = content.lines().count().saturating_sub(1);
            let cursor_col = content.lines().last().map(|l| l.len()).unwrap_or(0);

            if cursor_line < inner_area.height as usize && cursor_col < inner_area.width as usize {
                let x = inner_area.x + cursor_col as u16;
                let y = inner_area.y + cursor_line as u16;
                if x < inner_area.right() && y < inner_area.bottom() {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_style(Style::default().bg(Color::White).fg(Color::Black));
                    }
                }
            }
        }
    }
}
