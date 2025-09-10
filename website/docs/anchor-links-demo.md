______________________________________________________________________

## layout: documentation

title: "Anchor Links Demo" description: "Demo page showing anchor link
functionality for long documents"

# Anchor Links Demo

This page demonstrates the anchor links functionality implemented for the Caxton
documentation site. Anchor links provide deep-linking, smooth scrolling, and
copy-to-clipboard functionality for better navigation of long documents.

## Features Overview

The anchor links system provides several key features:

### Auto-Generated Anchor Links

All headings (h2-h6) automatically receive anchor links that appear on hover.
These links allow users to:

- Navigate directly to specific sections
- Copy links to specific sections for sharing
- Bookmark specific parts of documentation

### Smooth Scrolling Behavior

When clicking anchor links, the page smoothly scrolls to the target section with
proper offset to account for fixed headers.

### Copy-to-Clipboard Functionality

Users can copy anchor links using:

- **Right-click**: Context menu to copy the full URL
- **Ctrl+Click**: Copy link without navigation
- **Visual Feedback**: Icon changes to checkmark when copied
- **Toast Notification**: Confirmation message appears

## Table of Contents Generation

The system automatically generates table of contents for documents with 3+
headings:

### Manual TOC

You can also manually create table of contents by adding:

```html
<div class="table-of-contents">
  <h3>Contents</h3>
</div>
```

### Nested Headings Support

The TOC generator properly handles nested heading structures:

#### Subsection Example

This demonstrates how h4 headings are nested under h3 headings in the generated
table of contents.

##### Deep Nesting

Even h5 headings are properly nested and linked.

## Accessibility Features

The anchor links implementation includes several accessibility enhancements:

### Keyboard Navigation

- Anchor links are keyboard focusable (tab navigation)
- Proper ARIA labels for screen readers
- Focus management when navigating to sections

### Screen Reader Support

- Descriptive aria-labels for each anchor link
- Hidden decorative elements from screen readers
- Semantic markup preservation

### Reduced Motion Support

- Respects prefers-reduced-motion setting
- Falls back to instant scrolling when motion is reduced
- Minimal animations for accessibility

## Mobile Optimization

The anchor links are optimized for mobile devices:

### Touch-Friendly Design

- Larger touch targets on mobile
- Appropriate spacing for finger navigation
- Always visible on mobile (subtle opacity)

### Responsive Behavior

- Anchor links adapt to smaller screen sizes
- Table of contents remains usable on mobile
- Smooth scrolling works on touch devices

## Implementation Details

The anchor links system is built with:

### Performance Optimizations

- Lazy initialization to reduce page load impact
- Efficient event delegation for click handling
- Minimal DOM manipulation for better performance

### Browser Compatibility

- Modern clipboard API with fallbacks
- Works in all major browsers
- Progressive enhancement approach

### Integration

- Works seamlessly with Jekyll/GitHub Pages
- Compatible with existing CSS frameworks
- Easy to customize and theme

## Testing Section Alpha

This section tests the anchor link generation with various heading styles and
content.

### Testing Section Beta

Another test section to verify proper ID generation and linking.

### Testing Section Gamma

Final test section with special characters: "Hello, World!" & More.

## Code Examples

Here's how to manually add anchor links to existing content:

```javascript
// Initialize anchor links system
const anchorLinks = new AnchorLinks();

// Refresh anchors after dynamic content changes
anchorLinks.refreshAnchors();

// Programmatically scroll to an anchor
anchorLinks.scrollToAnchor('section-id');
```

### CSS Customization

You can customize the appearance of anchor links:

```css
.anchor-link {
  /* Custom styles for anchor links */
  background: your-custom-color;
  border-radius: 8px;
}

.anchor-icon {
  /* Custom icon styles */
  font-size: 1rem;
}
```

## Advanced Features

### Dynamic Content Support

The anchor links system can handle dynamically added content:

#### API Methods

- `refreshAnchors()`: Regenerate all anchor links
- `scrollToAnchor(id)`: Programmatically navigate to section
- `updateTableOfContents()`: Rebuild TOC after content changes

#### Event Integration

The system integrates with existing page functionality and doesn't interfere
with other JavaScript.

## Conclusion

The anchor links system enhances the documentation experience by providing:

1. **Better Navigation**: Quick access to any section
2. **Improved Sharing**: Direct links to specific content
3. **Enhanced Accessibility**: Keyboard and screen reader friendly
4. **Mobile Optimization**: Works great on all devices
5. **Performance**: Minimal impact on page load times

Try hovering over any heading on this page to see the anchor links in action!
