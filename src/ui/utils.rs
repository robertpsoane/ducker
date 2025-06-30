use crate::sorting::{SortOrder, SortState};

/// Renders a column header with optional sort indicators
pub fn render_column_header(column_name: &str, is_sorted: bool, sort_order: SortOrder) -> String {
    if is_sorted {
        match sort_order {
            SortOrder::Ascending => format!("{} ↑", column_name),
            SortOrder::Descending => format!("{} ↓", column_name),
        }
    } else {
        column_name.to_string()
    }
}

/// Helper function to determine if a field is currently being sorted
pub fn is_field_sorted<T: PartialEq + Clone>(sort_state: &SortState<T>, field: &T) -> bool {
    sort_state.is_field_sorted(field.clone())
}

/// Gets the current sort order for a field, or None if not sorted
pub fn get_field_sort_order<T: PartialEq + Clone>(
    sort_state: &SortState<T>,
    field: &T,
) -> Option<SortOrder> {
    sort_state.get_order_for_field(field.clone())
}
