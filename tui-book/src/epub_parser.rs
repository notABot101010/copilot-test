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

    // Extract title, handling metadata properly
    let title = doc.mdata("title")
        .and_then(|item| {
            // MetadataItem may have multiple formats, try to extract cleanly
            let debug_str = format!("{:?}", item);
            // Strip debug formatting artifacts like quotes
            Some(debug_str.trim_matches('"').to_string())
        })
        .unwrap_or_else(|| "Unknown Title".to_string());

    // Extract table of contents
    let mut toc = Vec::new();
    let toc_data = doc.toc.clone();
    let num_chapters = doc.get_num_chapters();
    
    for (index, nav_point) in toc_data.iter().enumerate() {
        toc.push(TocEntry {
            title: nav_point.label.clone(),
            section_index: if num_chapters > 0 {
                index.min(num_chapters - 1)
            } else {
                0
            },
        });
    }

    // If TOC is empty, create default entries based on spine
    if toc.is_empty() && num_chapters > 0 {
        for i in 0..num_chapters {
            toc.push(TocEntry {
                title: format!("Section {}", i + 1),
                section_index: i,
            });
        }
    }

    // Extract content from all sections
    let mut sections = Vec::new();
    
    for i in 0..num_chapters {
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
                
                // Handle line breaks and block elements
                if tag_name == "br" {
                    result.push('\n');
                } else if tag_name == "p" || tag_name == "div" || tag_name == "h1" || 
                          tag_name == "h2" || tag_name == "h3" || tag_name == "h4" || 
                          tag_name == "h5" || tag_name == "h6" {
                    // Add paragraph break before block element
                    if !result.is_empty() && !result.ends_with("\n\n") {
                        result.push_str("\n\n");
                    }
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

    // Clean up whitespace while preserving paragraph breaks
    let mut lines: Vec<String> = Vec::new();
    let mut current_para = Vec::new();
    
    for line in result.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            // Empty line - if we have content, save it and add a paragraph break
            if !current_para.is_empty() {
                lines.push(current_para.join(" "));
                current_para.clear();
                lines.push(String::new()); // Paragraph break
            }
        } else {
            current_para.push(trimmed.to_string());
        }
    }
    
    // Add any remaining content
    if !current_para.is_empty() {
        lines.push(current_para.join(" "));
    }
    
    result = lines.join("\n");

    result
}
