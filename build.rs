const COMMANDS: &[&str] = &["load", "execute", "select", "close"];

fn main() {
  tauri_plugin::Builder::new(COMMANDS)
    .build();
}
