use rand::Rng;
use std::io::{self, Write};
use std::time::{Duration, Instant};

struct LineState {
    target: Vec<char>,
    settle_times: Vec<Duration>,
    is_secondary: bool,
}

pub async fn display_parallel_intro(lines: Vec<String>) {
    let mut rng = rand::rng();
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()_+-=[]{}|;:,.<>?".chars().collect();
    
    // Primary Color: #6ED1C3 -> RGB(110, 209, 195)
    // Secondary Color: #5A5A5A -> RGB(90, 90, 90)
    let primary = "\x1b[38;2;110;209;195m";
    let gray = "\x1b[38;5;240m";
    let reset = "\x1b[0m";

    let mut states = Vec::new();
    for line in lines {
        let is_secondary = line.starts_with(" ");
        let target: Vec<char> = line.chars().collect();
        let settle_times: Vec<Duration> = (0..target.len())
            .map(|_| Duration::from_millis(rng.random_range(100..1000)))
            .collect();
        states.push(LineState { target, settle_times, is_secondary });
    }

    let start_time = Instant::now();
    let mut finished = false;

    // Hide cursor
    print!("\x1b[?25l");
    io::stdout().flush().unwrap();

    while !finished {
        let elapsed = start_time.elapsed();
        let mut all_settled = true;

        // Move cursor back up for all lines except the first frame
        // (We'll just print and then use CSI to move up)
        let mut output = String::new();

        for state in &states {
            let mut line_display = String::new();
            let color = if state.is_secondary { gray } else { primary };
            
            for (i, &target_char) in state.target.iter().enumerate() {
                if elapsed >= state.settle_times[i] {
                    line_display.push(target_char);
                } else {
                    if target_char.is_whitespace() {
                        line_display.push(' ');
                    } else {
                        let rand_idx = rng.random_range(0..chars.len());
                        line_display.push(chars[rand_idx]);
                    }
                    all_settled = false;
                }
            }
            output.push_str(&format!("{}{}{}\n", color, line_display, reset));
        }

        // Print everything
        print!("\r{}", output);
        io::stdout().flush().unwrap();

        if all_settled {
            finished = true;
        } else {
            // Move cursor back up to the start of the block
            print!("\x1b[{}A", states.len());
            tokio::time::sleep(Duration::from_millis(30)).await;
        }
    }
    
    // Show cursor again
    print!("\x1b[?25h");
    io::stdout().flush().unwrap();
}
