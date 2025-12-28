use crate::models::{MindMap, Node, NodeColor};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap, Widget},
};
use unicode_width::UnicodeWidthStr;
use uuid::Uuid;

pub struct Canvas<'a> {
    pub mindmap: &'a MindMap,
    pub zoom: f64,
    pub pan_x: f64,
    pub pan_y: f64,
    pub selected_node: Option<Uuid>,
    pub connecting_from: Option<Uuid>,
}

impl<'a> Canvas<'a> {
    pub fn new(mindmap: &'a MindMap, zoom: f64, pan_x: f64, pan_y: f64) -> Self {
        Self {
            mindmap,
            zoom,
            pan_x,
            pan_y,
            selected_node: None,
            connecting_from: None,
        }
    }

    pub fn selected(mut self, node_id: Option<Uuid>) -> Self {
        self.selected_node = node_id;
        self
    }

    pub fn connecting(mut self, from_id: Option<Uuid>) -> Self {
        self.connecting_from = from_id;
        self
    }
}

impl<'a> Widget for Canvas<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear the area
        Clear.render(area, buf);

        // Draw connections first (so they appear below nodes)
        for conn in &self.mindmap.connections {
            if let (Some(from_node), Some(to_node)) = (
                self.mindmap.get_node_by_id(conn.from),
                self.mindmap.get_node_by_id(conn.to),
            ) {
                self.draw_connection(area, buf, from_node, to_node);
            }
        }

        // Draw nodes
        for node in &self.mindmap.nodes {
            self.draw_node(area, buf, node);
        }

        // Draw help text at the bottom
        self.draw_help(area, buf);
    }
}

impl<'a> Canvas<'a> {
    fn node_color_to_ratatui(node_color: NodeColor) -> Color {
        match node_color {
            NodeColor::Default => Color::DarkGray,
            NodeColor::Red => Color::Red,
            NodeColor::Green => Color::Green,
            NodeColor::Blue => Color::Blue,
            NodeColor::Yellow => Color::Yellow,
            NodeColor::Magenta => Color::Magenta,
            NodeColor::Cyan => Color::Cyan,
        }
    }

    fn draw_node(&self, area: Rect, buf: &mut Buffer, node: &Node) {
        let screen_x = ((node.x - self.pan_x) * self.zoom) as u16;
        let screen_y = ((node.y - self.pan_y) * self.zoom) as u16;
        let screen_width = (node.width as f64 * self.zoom) as u16;
        let screen_height = (node.height as f64 * self.zoom) as u16;

        // Skip if node is outside visible area
        if screen_x >= area.width || screen_y >= area.height {
            return;
        }

        let node_area = Rect {
            x: area.x + screen_x,
            y: area.y + screen_y,
            width: screen_width.min(area.width.saturating_sub(screen_x)),
            height: screen_height.min(area.height.saturating_sub(screen_y)),
        };

        if node_area.width == 0 || node_area.height == 0 {
            return;
        }

        let is_selected = self.selected_node == Some(node.id);
        let is_connecting = self.connecting_from == Some(node.id);

        let node_bg_color = Self::node_color_to_ratatui(node.color);
        let mut style = Style::default().fg(Color::White).bg(node_bg_color);
        let mut border_style = Style::default().fg(Color::Gray);

        if is_selected {
            style = style.bg(Color::Blue);
            border_style = border_style.fg(Color::Yellow).add_modifier(Modifier::BOLD);
        } else if is_connecting {
            style = style.bg(Color::Magenta);
            border_style = border_style.fg(Color::Magenta).add_modifier(Modifier::BOLD);
        }

        let title_text = if node.document.title.len() > screen_width.saturating_sub(4) as usize {
            let max_len = (screen_width.saturating_sub(5) as usize).min(node.document.title.len());
            format!("{}…", &node.document.title[..max_len])
        } else {
            node.document.title.clone()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .style(style);

        let paragraph = Paragraph::new(title_text)
            .block(block)
            .style(style);

        paragraph.render(node_area, buf);
    }

    fn draw_connection(&self, area: Rect, buf: &mut Buffer, from: &Node, to: &Node) {
        let from_x = ((from.x - self.pan_x + from.width as f64 / 2.0) * self.zoom) as u16;
        let from_y = ((from.y - self.pan_y + from.height as f64 / 2.0) * self.zoom) as u16;
        let to_x = ((to.x - self.pan_x + to.width as f64 / 2.0) * self.zoom) as u16;
        let to_y = ((to.y - self.pan_y + to.height as f64 / 2.0) * self.zoom) as u16;

        // Simple line drawing using Bresenham's algorithm
        self.draw_line(area, buf, from_x, from_y, to_x, to_y);
    }

    fn draw_line(&self, area: Rect, buf: &mut Buffer, x0: u16, y0: u16, x1: u16, y1: u16) {
        let dx = (x1 as i32 - x0 as i32).abs();
        let dy = (y1 as i32 - y0 as i32).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;
        let mut x = x0 as i32;
        let mut y = y0 as i32;

        loop {
            if x >= 0 && y >= 0 {
                let ux = x as u16;
                let uy = y as u16;
                if ux < area.width && uy < area.height {
                    let cell_x = area.x + ux;
                    let cell_y = area.y + uy;
                    if cell_x < buf.area.width && cell_y < buf.area.height {
                        if let Some(cell) = buf.cell_mut((cell_x, cell_y)) {
                            cell.set_symbol("─").set_fg(Color::DarkGray);
                        }
                    }
                }
            }

            if x == x1 as i32 && y == y1 as i32 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    fn draw_help(&self, area: Rect, buf: &mut Buffer) {
        let help_text = if self.connecting_from.is_some() {
            "Click another node to connect | Esc to cancel"
        } else if self.selected_node.is_some() {
            "Double-click: Open | N: New | D: Delete | C: Connect | X: Disconnect | R: Color | F: Search"
        } else {
            "+/-: Zoom | N: New | D: Delete | R: Color | F: Search | S: Save | L: Load | Q: Quit"
        };

        let help_y = area.y + area.height.saturating_sub(1);
        if help_y < buf.area.height {
            let help_line = Line::from(vec![
                Span::styled(help_text, Style::default().fg(Color::Cyan)),
            ]);

            let help_area = Rect {
                x: area.x,
                y: help_y,
                width: area.width,
                height: 1,
            };

            Paragraph::new(help_line).render(help_area, buf);
        }
    }
}

pub struct DocumentDialog<'a> {
    pub title_value: &'a str,
    pub title_cursor: usize,
    pub body_value: &'a str,
    pub body_cursor: usize,
    pub editing: bool,
    pub editing_title: bool,
}

impl<'a> Widget for DocumentDialog<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Calculate dialog size (centered, 60% of screen)
        let dialog_width = (area.width as f32 * 0.6) as u16;
        let dialog_height = (area.height as f32 * 0.6) as u16;
        let dialog_x = (area.width.saturating_sub(dialog_width)) / 2;
        let dialog_y = (area.height.saturating_sub(dialog_height)) / 2;

        let dialog_area = Rect {
            x: area.x + dialog_x,
            y: area.y + dialog_y,
            width: dialog_width,
            height: dialog_height,
        };

        // Clear the dialog area
        Clear.render(dialog_area, buf);

        let border_color = if self.editing {
            Color::Green
        } else {
            Color::Yellow
        };

        let mode_text = if self.editing {
            if self.editing_title {
                " [EDITING TITLE] "
            } else {
                " [EDITING BODY] "
            }
        } else {
            " [VIEW MODE] "
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(mode_text)
            .style(Style::default().bg(Color::Black));

        let inner = block.inner(dialog_area);
        block.render(dialog_area, buf);

        // Split inner area for title and body
        let title_height = 3;
        let title_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: title_height,
        };

        let body_area = Rect {
            x: inner.x,
            y: inner.y + title_height,
            width: inner.width,
            height: inner.height.saturating_sub(title_height + 2),
        };

        let help_area = Rect {
            x: inner.x,
            y: inner.y + inner.height.saturating_sub(2),
            width: inner.width,
            height: 2,
        };

        // Render title
        let title_style = if self.editing && self.editing_title {
            Style::default().fg(Color::Green).add_modifier(Modifier::UNDERLINED)
        } else {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        };

        let title_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray));

        let title_paragraph = Paragraph::new(self.title_value)
            .block(title_block)
            .style(title_style)
            .wrap(Wrap { trim: false });

        title_paragraph.render(title_area, buf);

        // Render cursor for title if editing
        if self.editing && self.editing_title {
            // Calculate visual width up to cursor position
            let text_before_cursor = &self.title_value.chars().take(self.title_cursor).collect::<String>();
            let visual_width = text_before_cursor.width();
            let cursor_x = title_area.x + (visual_width as u16).min(title_area.width.saturating_sub(1));
            let cursor_y = title_area.y;
            if cursor_x < buf.area.width && cursor_y < buf.area.height {
                if let Some(cell) = buf.cell_mut((cursor_x, cursor_y)) {
                    cell.set_style(Style::default().bg(Color::Green).fg(Color::Black));
                }
            }
        }

        // Render body
        let body_style = if self.editing && !self.editing_title {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::Gray)
        };

        let body_paragraph = Paragraph::new(self.body_value)
            .style(body_style)
            .wrap(Wrap { trim: false });

        body_paragraph.render(body_area, buf);

        // Render cursor for body if editing
        if self.editing && !self.editing_title {
            // Calculate visual width up to cursor position
            let text_before_cursor = &self.body_value.chars().take(self.body_cursor).collect::<String>();
            let visual_width = text_before_cursor.width();
            let cursor_x = body_area.x + (visual_width as u16).min(body_area.width.saturating_sub(1));
            let cursor_y = body_area.y;
            if cursor_x < buf.area.width && cursor_y < buf.area.height {
                if let Some(cell) = buf.cell_mut((cursor_x, cursor_y)) {
                    cell.set_style(Style::default().bg(Color::White).fg(Color::Black));
                }
            }
        }

        // Render help text
        let help_text = if self.editing {
            "Tab: Switch fields | Esc: Cancel | Enter: Save"
        } else {
            "Enter: Edit | Esc: Close"
        };

        let help_line = Line::from(vec![
            Span::styled(help_text, Style::default().fg(Color::Cyan)),
        ]);

        Paragraph::new(help_line).render(help_area, buf);
    }
}

pub struct SearchBox<'a> {
    pub query: &'a str,
    pub results_count: usize,
}

impl<'a> Widget for SearchBox<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Calculate search box size (top center, smaller)
        let box_width = 50.min(area.width);
        let box_height = 3;
        let box_x = (area.width.saturating_sub(box_width)) / 2;
        let box_y = 2;

        let box_area = Rect {
            x: area.x + box_x,
            y: area.y + box_y,
            width: box_width,
            height: box_height,
        };

        // Clear the box area
        Clear.render(box_area, buf);

        let title = if self.results_count > 0 {
            format!(" Search ({} results) ", self.results_count)
        } else {
            " Search ".to_string()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            .title(title)
            .style(Style::default().bg(Color::Black));

        let inner = block.inner(box_area);
        block.render(box_area, buf);

        let paragraph = Paragraph::new(self.query)
            .style(Style::default().fg(Color::White));

        paragraph.render(inner, buf);
    }
}
