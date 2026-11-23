//! HTML rendering hooks layered over comrak's default formatter.
//!
//! [`MdxFormatter`] renders every node exactly as comrak does, except
//! `Image` nodes when [`RenderFlags::lazy_images`] is set: those gain
//! `loading="lazy"` and `decoding="async"` so browsers defer
//! off-screen images and decode them off the main thread.

use comrak::html::{format_node_default, ChildRendering, Context};
use comrak::nodes::{AstNode, NodeLink, NodeValue};
use comrak::options::Plugins;
use comrak::{create_formatter, Options};
use std::fmt;

/// Attributes appended to every `<img>` when lazy images are on.
pub(crate) const LAZY_IMAGE_ATTRS: &str =
    r#" loading="lazy" decoding="async""#;

/// Per-render switches read by [`MdxFormatter`].
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct RenderFlags {
    /// Add [`LAZY_IMAGE_ATTRS`] to images.
    pub(crate) lazy_images: bool,
}

create_formatter!(MdxFormatter<RenderFlags>, {
    NodeValue::Image(ref link) => |context, node, entering| {
        if entering || !context.user.lazy_images {
            return format_node_default(context, node, entering);
        }
        return close_lazy_image(context, link);
    },
});

/// Writes the closing half of an `<img>` tag, mirroring comrak's own
/// image renderer but with the lazy-loading attributes inserted
/// before the self-closing `/>`.
fn close_lazy_image(
    context: &mut Context<RenderFlags>,
    link: &NodeLink,
) -> Result<ChildRendering, fmt::Error> {
    use fmt::Write;
    if !link.title.is_empty() {
        context.write_str("\" title=\"")?;
        context.escape(&link.title)?;
    }
    context.write_str("\"")?;
    context.write_str(LAZY_IMAGE_ATTRS)?;
    context.write_str(" />")?;
    if context.options.render.figure_with_caption {
        if !link.title.is_empty() {
            context.write_str("<figcaption>")?;
            context.escape(&link.title)?;
            context.write_str("</figcaption>")?;
        }
        context.write_str("</figure>")?;
    }
    Ok(ChildRendering::HTML)
}

/// Renders `root` to `output` with [`MdxFormatter`].
pub(crate) fn render_html<'a>(
    root: &'a AstNode<'a>,
    options: &Options,
    output: &mut String,
    plugins: &Plugins,
    flags: RenderFlags,
) -> fmt::Result {
    MdxFormatter::format_document_with_plugins(
        root, options, output, plugins, flags,
    )
    .map(|_| ())
}
