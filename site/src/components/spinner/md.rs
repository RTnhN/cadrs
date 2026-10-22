// SPDX-FileCopyrightText: 2026 Proa Labs, Inc.
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::Spinner;
use proa_core::WebContext;
use proa_core::{MdRenderSync, WriteBuf, WriteError};

/// Spinner is visual-only — renders nothing in markdown.
#[proa_macros::content(skip, reason = "Browser-only control has no static document content")]
impl<Loader: ::proa_core::DataLoader, B: WriteBuf> MdRenderSync<Loader, B> for Spinner {
    fn render_md(self, _cx: &mut WebContext<Loader, B>) -> Result<(), WriteError> {
        Ok(())
    }
}
