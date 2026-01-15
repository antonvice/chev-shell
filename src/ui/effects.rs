use rand::Rng;
use std::io::{self, Write};
use std::time::{Duration, Instant};

enum StringSegment {
    Ansi(String),
    Text(Vec<char>),
}

struct AnsiAwareLine {
    is_secondary: bool,
    segments: Vec<StringSegment>,
    total_visible_len: usize,
}

pub async fn display_parallel_intro(lines: Vec<String>) {
    let mut rng = rand::rng();
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()_+-=[]{}|;:,.<>?".chars().collect();
    
    let primary = "\x1b[38;2;110;209;195m";
    let gray = "\x1b[38;5;240m";
    let reset = "\x1b[0m";

    let mut aware_lines = Vec::new();
    for line in lines {
        let is_secondary = line.starts_with(" ");
        let mut segments = Vec::new();
        let mut current_text = Vec::new();
        let mut i = 0;
        let line_chars: Vec<char> = line.chars().collect();
        let mut total_visible_len = 0;

        while i < line_chars.len() {
            if line_chars[i] == '\x1b' {
                if !current_text.is_empty() {
                    segments.push(StringSegment::Text(current_text.clone()));
                    current_text.clear();
                }
                let mut ansi = String::new();
                ansi.push('\x1b');
                i += 1;
                while i < line_chars.len() && !line_chars[i].is_ascii_alphabetic() {
                    ansi.push(line_chars[i]);
                    i += 1;
                }
                if i < line_chars.len() {
                    ansi.push(line_chars[i]);
                    i += 1;
                }
                segments.push(StringSegment::Ansi(ansi));
            } else {
                current_text.push(line_chars[i]);
                total_visible_len += 1;
                i += 1;
            }
        }
        if !current_text.is_empty() {
            segments.push(StringSegment::Text(current_text));
        }

        aware_lines.push(AnsiAwareLine {
            is_secondary,
            segments,
            total_visible_len,
        });
    }

    let mut settle_times = Vec::new();
    for line in &aware_lines {
        let times: Vec<Duration> = (0..line.total_visible_len)
            .map(|_| Duration::from_millis(rng.random_range(150..800)))
            .collect();
        settle_times.push(times);
    }

    let start_time = Instant::now();
    let mut finished = false;

    // Hide cursor and use save/restore to prevent ghosting on wrap
    print!("\x1b[?25l\x1b[s");
    io::stdout().flush().unwrap();

    while !finished {
        let elapsed = start_time.elapsed();
        let mut all_settled = true;
        let mut frame_output = String::new();

        for (l_idx, line) in aware_lines.iter().enumerate() {
            let color = if line.is_secondary { gray } else { primary };
            frame_output.push_str(color);

            let mut visible_idx = 0;
            for segment in &line.segments {
                match segment {
                    StringSegment::Ansi(code) => {
                        frame_output.push_str(code);
                    }
                    StringSegment::Text(chars_vec) => {
                        for &c in chars_vec {
                            if elapsed >= settle_times[l_idx][visible_idx] {
                                frame_output.push(c);
                            } else {
                                if c.is_whitespace() {
                                    frame_output.push(' ');
                                } else {
                                    frame_output.push(chars[rng.random_range(0..chars.len())]);
                                }
                                all_settled = false;
                            }
                            visible_idx += 1;
                        }
                    }
                }
            }
            frame_output.push_str(reset);
            frame_output.push_str("\n");
        }

        // Return to saved start point and clear everything down for a perfect redraw
        print!("\x1b[u\x1b[J{}", frame_output);
        io::stdout().flush().unwrap();

        if all_settled {
            finished = true;
        } else {
            tokio::time::sleep(Duration::from_millis(30)).await;
        }
    }
    
    print!("\x1b[?25h");
    io::stdout().flush().unwrap();
}
