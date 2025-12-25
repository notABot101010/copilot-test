# Changelog

## [Unreleased]

### Fixed
- **TOC Scrolling**: Table of contents now automatically scrolls to keep the selected item visible when navigating through long lists. The selected item stays within the visible area, preventing it from going off-screen.

- **Paragraph Rendering**: Text now properly preserves paragraph breaks and spacing:
  - Block-level HTML elements (p, div, h1-h6) are properly converted to paragraph breaks
  - Empty lines between paragraphs are preserved
  - Text is no longer overly dense and has appropriate spacing

- **Text Centering**: Content area now uses percentage-based layout (60% width with 20% margins on each side) for more consistent centering, especially when the TOC panel is hidden.

- **Infinite Scroll Between Chapters**: Navigation now seamlessly continues across chapter boundaries:
  - Pressing arrow down at the end of a chapter automatically moves to the next chapter
  - Pressing arrow up at the beginning of a chapter automatically moves to the previous chapter
  - The scroll position is properly reset when changing chapters

### Technical Details

#### TOC Scrolling Implementation
- Added `toc_scroll_offset` field to track the scroll position in the TOC
- Implemented `update_toc_scroll()` method to calculate and maintain scroll offset
- Modified `render_toc()` to only render visible items within the viewport
- The scroll automatically adjusts when the selected item moves outside the visible area

#### Paragraph Rendering Improvements
- Enhanced HTML tag stripping logic to recognize block-level elements
- Added paragraph breaks (`\n\n`) before and after block elements
- Improved text cleanup logic to preserve empty lines as paragraph separators
- Words within a paragraph are joined, but paragraphs are separated by blank lines

#### Content Navigation Enhancements
- Modified `scroll_content_down()` to check if at the end of current section and automatically move to next section
- Modified `scroll_content_up()` to check if at the beginning of current section and automatically move to previous section
- Used `usize::MAX` as a sentinel value to indicate "scroll to bottom" when moving to previous chapter
- The rendering logic properly handles this by using `.min(max_scroll)` to cap at the actual maximum

#### Text Centering Fix
- Changed from `Constraint::Length` to `Constraint::Percentage` for layout calculations
- This ensures consistent centering regardless of terminal width
- The 60/20/20 split provides optimal reading width

### Testing
To test these changes, use the Alice in Wonderland EPUB:
```bash
wget -O alice.epub "https://www.gutenberg.org/ebooks/11.epub3.images"
cargo run --package tui-book -- alice.epub
```

Test scenarios:
1. Navigate through a long TOC (should auto-scroll)
2. Read content and verify paragraph spacing
3. Toggle TOC off/on and verify text stays centered
4. Scroll to the end of a chapter and continue scrolling (should move to next chapter)
5. Scroll to the beginning of a chapter and continue scrolling up (should move to previous chapter)
