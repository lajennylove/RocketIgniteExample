//! The `ignite` command for this app: `cargo run --bin ignite -- user create --email you@example.com --name "You"`.

fn main() {
    // The app reads DATABASE_URL from .env, so the CLI does too.
    let _ = dotenvy::dotenv();
    ignite_panel::cli::main();
}
