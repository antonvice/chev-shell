use std::io::Write;

pub enum RioAction {
    Notify { title: String, message: String },
    Opacity(f32),
    Badge(String),
    Ghost(String),
    SplitPane { direction: String, ratio: f32, command: String },
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
        RioAction::Ghost(text) => {
            format!("\x1b]1338;ghost;{}\x07", text)
        }
        RioAction::SplitPane { direction, ratio, command } => {
            format!("\x1b]1338;split;{};{};{}\x07", direction, ratio, command)
        }
    };
    print!("{}", sequence);
    let _ = std::io::stdout().flush();
}
