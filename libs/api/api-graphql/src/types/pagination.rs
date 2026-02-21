use async_graphql::SimpleObject;
use uuid::Uuid;

use super::message::Message;
use domain::message::Message as DomainMessage;
use domain::pagination::CursorPage;

/// Cursor-based page of messages returned by the GraphQL API.
#[derive(SimpleObject)]
pub struct MessagePage {
    pub items: Vec<Message>,
    /// Cursor to pass for the next page. `None` when on the last page.
    pub next_cursor: Option<Uuid>,
    pub has_next_page: bool,
}

impl From<CursorPage<DomainMessage>> for MessagePage {
    fn from(page: CursorPage<DomainMessage>) -> Self {
        Self {
            items: page.items.into_iter().map(Message::from).collect(),
            next_cursor: page.next_cursor,
            has_next_page: page.has_next_page,
        }
    }
}
