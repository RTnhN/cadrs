use axum::{http::StatusCode, response::IntoResponse};
use proa_core::{DataLoader, WebContext, WebRenderSync, WriteError};
use proa_framework_axum::RouteResponse;
use proa_macros::html_sync;

use crate::layouts::RootDocument;

#[derive(Debug, Clone, Copy)]
pub struct NotFoundPage;

impl<L: DataLoader> WebRenderSync<L> for NotFoundPage {
    fn render(self, cx: &mut WebContext<L>) -> Result<(), WriteError> {
        let home = cx.request().url("/");
        html_sync! {
            <main class="mx-auto flex min-h-screen max-w-xl flex-col justify-center px-4 sm:px-6">
                <p class="text-sm font-medium text-accent">"404"</p>
                <h1 class="mt-2 text-3xl font-bold tracking-tight">"There is no page here"</h1>
                <p class="mt-4 text-muted">"The link may be old, or the address mistyped."</p>
                <a
                    href={home}
                    class="mt-8 self-start rounded-lg bg-accent px-5 py-2.5 font-semibold text-on-accent hover:bg-accent-strong"
                >
                    "Back to cadrs"
                </a>
            </main>
        }
        .render(cx)
    }
}

pub async fn handler() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        RouteResponse::ssr_async(RootDocument {
            title: "Page not found · cadrs",
            description: "This page does not exist.",
            content: proa_core::to_async(NotFoundPage),
        }),
    )
}
