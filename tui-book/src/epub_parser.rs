use anyhow::{Context, Result};
use epub::doc::EpubDoc;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct TocEntry {
    pub title: String,
    pub section_index: usize,
}

pub struct BookContent {
    pub title: String,
    pub toc: Vec<TocEntry>,
    pub sections: Vec<String>,
}

pub fn parse_epub<P: AsRef<Path>>(path: P) -> Result<BookContent> {
    let mut doc = EpubDoc::new(path).context("Failed to open EPUB file")?;

    let title = doc.mdata("title")
        .map(|item| format!("{:?}", item))
        .unwrap_or_else(|| "Unknown Title".to_string());

    // Extract table of contents
    let mut toc = Vec::new();
    let toc_data = doc.toc.clone();
    
    for (index, nav_point) in toc_data.iter().enumerate() {
        toc.push(TocEntry {
            title: nav_point.label.clone(),
            section_index: index.min(doc.get_num_chapters() - 1),
        });
    }

    // If TOC is empty, create default entries based on spine
    if toc.is_empty() {
        for i in 0..doc.get_num_chapters() {
            toc.push(TocEntry {
                title: format!("Section {}", i + 1),
                section_index: i,
            });
        }
    }

    // Extract content from all sections
    let mut sections = Vec::new();
    let num_pages = doc.get_num_chapters();
    
    for i in 0..num_pages {
        doc.set_current_chapter(i);
        
        if let Some((content_bytes, _mime)) = doc.get_current_str() {
            // Strip HTML tags and extract plain text
            let text = strip_html_tags(&content_bytes);
            sections.push(text);
        } else {
            sections.push(String::new());
        }
    }

    Ok(BookContent {
        title,
        toc,
        sections,
    })
}

fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut inside_script_or_style = false;
    let mut tag_name = String::new();

    let chars: Vec<char> = html.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        if ch == '<' {
            tag_name.clear();
            i += 1;
            
            // Check for closing tag
            if i < chars.len() && chars[i] == '/' {
                i += 1;
                while i < chars.len() && chars[i] != '>' && !chars[i].is_whitespace() {
                    tag_name.push(chars[i].to_ascii_lowercase());
                    i += 1;
                }
                if tag_name == "script" || tag_name == "style" {
                    inside_script_or_style = false;
                }
            } else {
                // Opening tag
                while i < chars.len() && chars[i] != '>' && !chars[i].is_whitespace() {
                    tag_name.push(chars[i].to_ascii_lowercase());
                    i += 1;
                }
                if tag_name == "script" || tag_name == "style" {
                    inside_script_or_style = true;
                }
                
                // Handle line breaks
                if tag_name == "br" || tag_name == "p" || tag_name == "div" {
                    result.push('\n');
                }
            }
            
            // Skip to end of tag
            while i < chars.len() && chars[i] != '>' {
                i += 1;
            }
        } else if !inside_script_or_style {
            // Decode HTML entities
            if ch == '&' {
                let mut entity = String::new();
                let mut j = i + 1;
                while j < chars.len() && chars[j] != ';' && j < i + 10 {
                    entity.push(chars[j]);
                    j += 1;
                }
                
                if j < chars.len() && chars[j] == ';' {
                    let decoded = match entity.as_str() {
                        "amp" => '&',
                        "lt" => '<',
                        "gt" => '>',
                        "quot" => '"',
                        "apos" => '\'',
                        "nbsp" => ' ',
                        _ => {
                            if entity.starts_with('#') {
                                // Numeric entity
                                if let Some(num_str) = entity.strip_prefix('#') {
                                    if let Ok(code) = num_str.parse::<u32>() {
                                        std::char::from_u32(code).unwrap_or(ch)
                                    } else {
                                        ch
                                    }
                                } else {
                                    ch
                                }
                            } else {
                                ch
                            }
                        }
                    };
                    result.push(decoded);
                    i = j;
                } else {
                    result.push(ch);
                }
            } else {
                result.push(ch);
            }
        }
        
        i += 1;
    }

    // Clean up extra whitespace
    result = result
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    result
}
