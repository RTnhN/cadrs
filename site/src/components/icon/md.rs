// SPDX-FileCopyrightText: 2026 Proa Labs, Inc.
// SPDX-License-Identifier: (MIT OR Apache-2.0) AND (ISC AND MIT)

use super::{Icon, IconDataRef};
use proa_core::WebContext;
use proa_core::{MdRenderSync, WriteBuf, WriteError};

/// Icon is purely decorative SVG — it renders nothing in markdown.
#[proa_macros::content(
    skip,
    reason = "SVG icons are visual chrome without Markdown document content"
)]
impl<Loader: ::proa_core::DataLoader, I: IconDataRef, B: WriteBuf> MdRenderSync<Loader, B>
    for Icon<I>
{
    fn render_md(self, _cx: &mut WebContext<Loader, B>) -> Result<(), WriteError> {
        Ok(())
    }
}
