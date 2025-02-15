// Copyright 2018 The Druid Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Rich text with style spans.

use std::ops::{Range, RangeBounds};
use std::sync::Arc;

use super::attribute::Link;
use super::{Attribute, AttributeSpans, EnvUpdateCtx, TextStorage};
use crate::piet::{
    util, Color, FontFamily, FontStyle, FontWeight, PietTextLayoutBuilder, TextLayoutBuilder,
    TextStorage as PietTextStorage,
};
use crate::{ArcStr, Command, Data, Env, FontDescriptor, KeyOrValue};

/// Text with optional style spans.
#[derive(Clone, Debug, Data)]
pub struct RichText {
    buffer: ArcStr,
    attrs: Arc<AttributeSpans>,
    links: Arc<[Link]>,
}

impl RichText {
    /// Create a new `RichText` object with the provided text.
    pub fn new(buffer: ArcStr) -> Self {
        RichText::new_with_attributes(buffer, Default::default())
    }

    /// Create a new `RichText`, providing explicit attributes.
    pub fn new_with_attributes(buffer: ArcStr, attributes: AttributeSpans) -> Self {
        RichText {
            buffer,
            attrs: Arc::new(attributes),
            links: Arc::new([]),
        }
    }

    /// Builder-style method for adding an [`Attribute`] to a range of text.
    ///
    /// [`Attribute`]: enum.Attribute.html
    pub fn with_attribute(mut self, range: impl RangeBounds<usize>, attr: Attribute) -> Self {
        self.add_attribute(range, attr);
        self
    }

    /// The length of the buffer, in utf8 code units.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns `true` if the underlying buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Add an [`Attribute`] to the provided range of text.
    ///
    /// [`Attribute`]: enum.Attribute.html
    pub fn add_attribute(&mut self, range: impl RangeBounds<usize>, attr: Attribute) {
        let range = util::resolve_range(range, self.buffer.len());
        Arc::make_mut(&mut self.attrs).add(range, attr);
    }
}

impl PietTextStorage for RichText {
    fn as_str(&self) -> &str {
        self.buffer.as_str()
    }
}

impl TextStorage for RichText {
    fn add_attributes(
        &self,
        mut builder: PietTextLayoutBuilder,
        env: &Env,
    ) -> PietTextLayoutBuilder {
        for (range, attr) in self.attrs.to_piet_attrs(env) {
            builder = builder.range_attribute(range, attr);
        }
        builder
    }

    fn env_update(&self, ctx: &EnvUpdateCtx) -> bool {
        self.attrs.env_update(ctx)
    }

    fn links(&self) -> &[Link] {
        &self.links
    }
}

/// A builder for creating [`RichText`] objects.
///
/// This builder allows you to construct a [`RichText`] object by building up a sequence
/// of styled sub-strings; first you [`push`](RichTextBuilder::push) a `&str` onto the string,
/// and then you can optionally add styles to that text via the returned [`AttributesAdder`]
/// object.
///
/// # Example
/// ```
/// # use druid::text::{Attribute, RichTextBuilder};
/// # use druid::{FontWeight, Color};
/// let mut builder = RichTextBuilder::new();
/// builder.push("Hello ");
/// builder.push("World!").weight(FontWeight::BOLD);
///
/// // Can also use write!
/// write!(builder, "Here is your number: {}", 1).underline(true).text_color(Color::RED);
///
/// let rich_text = builder.build();
/// ```
///
/// [`RichText`]: RichText
#[derive(Default)]
pub struct RichTextBuilder {
    buffer: String,
    attrs: AttributeSpans,
    links: Vec<Link>,
}

impl RichTextBuilder {
    /// Create a new `RichTextBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a `&str` to the end of the text.
    ///
    /// This method returns a [`AttributesAdder`] that can be used to style the newly
    /// added string slice.
    pub fn push(&mut self, string: &str) -> AttributesAdder {
        let range = self.buffer.len()..(self.buffer.len() + string.len());
        self.buffer.push_str(string);
        self.add_attributes_for_range(range)
    }

    /// Glue for usage of the write! macro.
    ///
    /// This method should generally not be invoked manually, but rather through the write! macro itself.
    #[doc(hidden)]
    pub fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> AttributesAdder {
        use std::fmt::Write;
        let start = self.buffer.len();
        self.buffer
            .write_fmt(fmt)
            .expect("a formatting trait implementation returned an error");
        self.add_attributes_for_range(start..self.buffer.len())
    }

    /// Get an [`AttributesAdder`] for the given range.
    ///
    /// This can be used to modify styles for a given range after it has been added.
    pub fn add_attributes_for_range(&mut self, range: impl RangeBounds<usize>) -> AttributesAdder {
        let range = util::resolve_range(range, self.buffer.len());
        AttributesAdder {
            rich_text_builder: self,
            range,
        }
    }

    /// Build the `RichText`.
    pub fn build(self) -> RichText {
        RichText {
            buffer: self.buffer.into(),
            attrs: self.attrs.into(),
            links: self.links.into(),
        }
    }
}

/// Adds Attributes to the text.
///
/// See also: [`RichTextBuilder`](RichTextBuilder)
pub struct AttributesAdder<'a> {
    rich_text_builder: &'a mut RichTextBuilder,
    range: Range<usize>,
}

impl AttributesAdder<'_> {
    /// Add the given attribute.
    pub fn add_attr(&mut self, attr: Attribute) -> &mut Self {
        self.rich_text_builder.attrs.add(self.range.clone(), attr);
        self
    }

    /// Add a font size attribute.
    pub fn size(&mut self, size: impl Into<KeyOrValue<f64>>) -> &mut Self {
        self.add_attr(Attribute::size(size));
        self
    }

    /// Add a forground color attribute.
    pub fn text_color(&mut self, color: impl Into<KeyOrValue<Color>>) -> &mut Self {
        self.add_attr(Attribute::text_color(color));
        self
    }

    /// Add a font family attribute.
    pub fn font_family(&mut self, family: FontFamily) -> &mut Self {
        self.add_attr(Attribute::font_family(family));
        self
    }

    /// Add a `FontWeight` attribute.
    pub fn weight(&mut self, weight: FontWeight) -> &mut Self {
        self.add_attr(Attribute::weight(weight));
        self
    }

    /// Add a `FontStyle` attribute.
    pub fn style(&mut self, style: FontStyle) -> &mut Self {
        self.add_attr(Attribute::style(style));
        self
    }

    /// Add a underline attribute.
    pub fn underline(&mut self, underline: bool) -> &mut Self {
        self.add_attr(Attribute::underline(underline));
        self
    }

    /// Add a `FontDescriptor` attribute.
    pub fn font_descriptor(&mut self, font: impl Into<KeyOrValue<FontDescriptor>>) -> &mut Self {
        self.add_attr(Attribute::font_descriptor(font));
        self
    }

    /// Add a [`Link`] attribute.
    ///
    /// [`Link`]: super::attribute::Link
    pub fn link(&mut self, command: impl Into<Command>) -> &mut Self {
        self.rich_text_builder
            .links
            .push(Link::new(self.range.clone(), command.into()));
        self
    }
}
