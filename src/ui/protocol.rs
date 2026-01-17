use std::io::Write;

pub enum RioAction {
    Notify { title: String, message: String },
    Opacity(f32),
    Badge(String),
    Ghost(String),
    SplitPane { direction: String, ratio: f32, command: String },
    Preview(String),
    MiniMap(bool),
    BackgroundEffect(Option<String>),
    ProgressBar { fraction: f32, label: String },
    Edit(String),
    RequestHistory,
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
        RioAction::Preview(path) => {
            format!("\x1b]1338;preview;{}\x07", path)
        }
        RioAction::MiniMap(enabled) => {
            let val = if enabled { "1" } else { "0" };
            format!("\x1b]1338;minimap;{}\x07", val)
        }
         RioAction::BackgroundEffect(name) => {
            let effect = name.as_deref().unwrap_or("none");
            format!("\x1b]1338;effect;{}\x07", effect)
        }
        RioAction::ProgressBar { fraction, label } => {
            format!("\x1b]1338;progress;{};{}\x07", fraction, label)
        }
        RioAction::Edit(path) => {
            format!("\x1b]1338;edit;{}\x07", path)
        }
        RioAction::RequestHistory => {
            format!("\x1b]1338;request-history\x07")
        }
    };
    print!("{}", sequence);
    let _ = std::io::stdout().flush();
}
