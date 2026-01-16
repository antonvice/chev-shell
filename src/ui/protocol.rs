use std::io::Write;

pub enum RioAction {
    Notify { title: String, message: String },
    Opacity(f32),
    Badge(String),
}

pub fn send_rio(action: RioAction) {
    let sequence = match action {
        RioAction::Notify { title, message } => {
            format!("\x1b]1338;notify;{};{}\x07", title, message)
        }
        RioAction::Opacity(val) => {
            format!("\x1b]1338;opacity;{}\x07", val)
        }
        RioAction::Badge(text) => {
            format!("\x1b]1338;badge;{}\x07", text)
        }
    };
    print!("{}", sequence);
    let _ = std::io::stdout().flush();
}
