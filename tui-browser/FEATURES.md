# TUI Browser Features

## Text Wrapping

The browser now properly wraps text content to fit the terminal width:

- Uses ratatui's `Paragraph` widget with `Wrap { trim: false }`
- Text flows naturally across multiple lines instead of being truncated
- Link numbers are preserved at the start of wrapped lines
- Content width is adjustable with zoom (+/-) keys

## Image Support with Sixel

The browser can now display images inline using the sixel protocol:

### Supported Protocols

The `ratatui-image` library auto-detects and uses the best available protocol:
- **Sixel** - Supported by xterm, mlterm, foot, and others
- **Kitty Graphics Protocol** - For Kitty terminal
- **iTerm2 Inline Images** - For iTerm2 terminal

### Features

- Automatically extracts images from HTML `<img>` tags
- Converts relative URLs to absolute URLs (with validation)
- Downloads up to 5 images per page by default (configurable via `MAX_IMAGES_PER_PAGE` constant)
- Shows image loading status (✓ for loaded, ✗ for failed)
- Displays first loaded image as a preview in the content area
- Images are resized to fit the display area using high-quality Lanczos3 filtering
- Skips images with invalid URLs

### How It Works

1. When a page is loaded, HTML is parsed for `<img>` tags
2. Image URLs are extracted and converted to absolute URLs
3. Images are downloaded asynchronously using reqwest
4. Image data is decoded using the `image` crate
5. The `ratatui-image` picker detects terminal capabilities
6. Images are rendered using the best available protocol

### Usage

Simply navigate to a webpage with images:
1. Enter a URL in the URL bar
2. Press Enter to load the page
3. Images will be downloaded automatically
4. The status bar shows "Loaded: {title} ({n} images)"
5. Scroll down to see image previews

### Example

```
═══ Images on this Page ═══
✓ [IMG 1] Test Image 1
[IMAGE_PLACEHOLDER_1]
✓ [IMG 2] Test Image 2
[IMAGE_PLACEHOLDER_2]
═══════════════════════════
```

## Technical Details

### Dependencies

- `ratatui-image` v1.0 - Multi-protocol image rendering
- `image` v0.25 - Image decoding and manipulation

### Image Rendering Pipeline

```rust
HTML → Extract <img> tags → Resolve URLs → Download → Decode → Resize → Render with sixel
```

### Terminal Compatibility

The image rendering will work in terminals that support:
- Sixel graphics (most common)
- Kitty graphics protocol
- iTerm2 inline images protocol

If none of these are available, images are shown as text placeholders.

## Future Enhancements

Possible improvements for image support:
- Render all images inline instead of just the first one
- Add image gallery view mode
- Support for inline image cycling with keyboard shortcuts
- Cache downloaded images to disk
- Support for more image formats (currently supports PNG, JPEG, GIF, etc.)
- Lazy loading of images as you scroll
