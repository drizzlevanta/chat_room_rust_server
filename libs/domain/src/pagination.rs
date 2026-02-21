use uuid::Uuid;

/// A cursor-based page of results.
///
/// The cursor is a UUIDv7 `public_id` that points to the boundary of the current page.
/// Clients pass it back to fetch the next page of results.
///
/// # Type Parameters
/// * `T` - The type of items in the page
pub struct CursorPage<T> {
    /// The items in this page
    pub items: Vec<T>,
    /// Cursor pointing to the next page, `None` if this is the last page
    pub next_cursor: Option<Uuid>,
    /// Whether there are more items after this page
    pub has_next_page: bool,
}
