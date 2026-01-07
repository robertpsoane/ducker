use std::panic::{set_hook, take_hook};

pub fn init_panic_hook() {
    let original_hook = take_hook();
    set_hook(Box::new(move |panic_info| {
        // intentionally ignore errors here since we're already in a panic
        ratatui::restore();
        original_hook(panic_info);
    }));
}
