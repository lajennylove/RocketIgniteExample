//! Compiles the app's views into Rust, so a release build renders them without a
//! template interpreter. The panel's own views are compiled too, because the
//! app's views extend its layout (`ignite::layouts.app`).

fn main() {
    // ignite-panel publishes its views directory to its dependents.
    let panel_views = std::env::var("DEP_IGNITE_PANEL_VIEWS_DIR")
        .expect("ignite-panel publishes its views directory");

    propel_compiler::build::Views::new("resources/views")
        .namespace("ignite", panel_views)
        .asset_namespace("ui")
        .view("home", "crate::Home")
        .view("subscribe", "crate::subscribe::Subscribe")
        .build();
}
