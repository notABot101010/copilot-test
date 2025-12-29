use crate::models::{ImageInfo, Tab};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget, Wrap},
};
use ratatui_image::{picker::Picker, StatefulImage};

// Image rendering constants
// Terminal cells to pixels conversion: typically 1 cell = 2 pixels wide, 4 pixels tall
const CELL_TO_PIXEL_WIDTH: u32 = 2;
const CELL_TO_PIXEL_HEIGHT: u32 = 4;

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
        images: &[ImageInfo],
        image_picker: &mut Option<Picker>,
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

        // Build a map of original line indices to link numbers
        let mut line_to_link: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
        for (link_idx, link) in links.iter().enumerate() {
            line_to_link.entry(link.line_index)
                .or_insert_with(Vec::new)
                .push(link_idx + 1);
        }

        // Build styled lines with link numbers and proper wrapping
        let mut styled_lines: Vec<Line> = Vec::new();
        for (line_idx, line) in content.lines().enumerate() {
            let mut spans = Vec::new();

            // Add link numbers if this line has links
            if let Some(link_numbers) = line_to_link.get(&line_idx) {
                let link_label = if link_numbers.len() == 1 {
                    format!("[{}] ", link_numbers[0])
                } else {
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
                spans.push(Span::styled(
                    link_label,
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ));
            }

            // Add the line content
            spans.push(Span::styled(
                line.to_string(),
                Style::default().fg(Color::White),
            ));

            styled_lines.push(Line::from(spans));
        }

        let total_lines = styled_lines.len();

        // Create paragraph with proper wrapping
        let paragraph = Paragraph::new(styled_lines)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false })
            .scroll((scroll_offset as u16, 0));

        paragraph.render(centered_area, buf);

        // Render images if available and if picker is initialized
        if let Some(picker) = image_picker {
            // Find images with data
            let loaded_images: Vec<&ImageInfo> = images.iter()
                .filter(|img| img.data.is_some())
                .collect();

            if !loaded_images.is_empty() {
                // Render first image in a small preview area at the bottom
                // TODO: Implement full inline rendering for all images
                // Currently only the first image is displayed as a preview
                let image_height = 10.min(centered_area.height / 3);
                if centered_area.height > image_height + 2 {
                    let image_area = Rect {
                        x: centered_area.x,
                        y: centered_area.y + centered_area.height - image_height,
                        width: centered_area.width.min(40),
                        height: image_height,
                    };

                    if let Some(first_image) = loaded_images.first() {
                        if let Some(img_data) = &first_image.data {
                            // Create a resized version that fits the area
                            // Convert terminal cell dimensions to pixel dimensions
                            let resized = img_data.resize(
                                image_area.width as u32 * CELL_TO_PIXEL_WIDTH,
                                image_area.height as u32 * CELL_TO_PIXEL_HEIGHT,
                                image::imageops::FilterType::Lanczos3
                            );

                            let mut image_state = picker.new_resize_protocol(resized);
                            let image_widget = StatefulImage::new();
                            image_widget.render(image_area, buf, &mut image_state);
                        }
                    }
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
            "Search:",
            "  Ctrl+S       - Start search in page",
            "  n/N          - Next/Previous search result",
            "  (In search):",
            "    Type       - Search as you type",
            "    Enter      - Go to next result",
            "    Esc        - Exit search",
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
