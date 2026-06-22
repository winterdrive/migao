use clap::{Parser, Subcommand};
use std::io::{self, IsTerminal, Read, Write};

#[derive(Parser)]
#[command(name = "migao", about = "IME garbled text recovery — 翻譯米糕")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Recover garbled text (pipe or argument)
    Fix {
        /// The garbled text to recover. Reads from stdin if omitted.
        text: Option<String>,

        /// Which IME layout produced the garbled text
        #[arg(long, default_value = "bopomofo-daqian")]
        ime: String,

        /// Show top N candidates. When stdout is a TTY, prompts for selection.
        #[arg(short = 'n', long = "top", default_value_t = 1)]
        top: usize,
    },
    /// List supported IME identifiers
    List,
    /// Record a correction to improve future results
    ///
    /// Example: migao report "1p4w1" "版本"
    ///
    /// Tip: to find the garbled form of a word, type it with Bopomofo IME
    /// active but keyboard in English mode, then select and press Ctrl+Alt+R.
    /// The garbled keys appear in the tray tooltip before the arrow.
    Report {
        /// The garbled ASCII key sequence (e.g. "1p4w1")
        garbled: String,
        /// The correct Chinese text (e.g. "版本")
        correct: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Fix { text, ime, top } => {
            let input = match text {
                Some(t) => t,
                None => {
                    let mut s = String::new();
                    io::stdin().read_to_string(&mut s)?;
                    s.trim_end_matches('\n').to_string()
                }
            };

            if top <= 1 {
                match migao::recover(&input, &ime) {
                    Some(result) => println!("{}", result),
                    None => {
                        eprintln!("migao: input does not look like '{}' garbled text", ime);
                        std::process::exit(1);
                    }
                }
            } else {
                let candidates = migao::recover_top_n(&input, &ime, top);
                if candidates.is_empty() {
                    eprintln!("migao: input does not look like '{}' garbled text", ime);
                    std::process::exit(1);
                }

                let is_tty = io::stdout().is_terminal();

                if !is_tty {
                    for c in &candidates {
                        println!("{}", c);
                    }
                } else {
                    for (i, c) in candidates.iter().enumerate() {
                        println!("{}  {}", i + 1, c);
                    }
                    if candidates.len() > 1 {
                        print!("Pick [1-{}] (default 1): ", candidates.len());
                        io::stdout().flush()?;

                        let mut line = String::new();
                        io::stdin().read_line(&mut line)?;
                        let choice: usize = line
                            .trim()
                            .parse::<usize>()
                            .unwrap_or(1)
                            .clamp(1, candidates.len());

                        println!("{}", candidates[choice - 1]);
                    }
                }
            }
        }

        Commands::List => {
            println!("Supported IME identifiers:");
            println!("  bopomofo-daqian  (aliases: zhuyin, 注音)  — 大千標準注音鍵盤");
            println!("  pinyin           (alias: 拼音)             — 全拼（標準 QWERTY）");
        }

        Commands::Report { garbled, correct } => {
            use migao::ime::daqian::{self, Segment};

            let segments = daqian::segment(&garbled);
            let syllables: Vec<String> = segments
                .iter()
                .filter_map(|s| {
                    if let Segment::Syllable(keys) = s {
                        if daqian::is_valid_syllable(keys) {
                            let z = daqian::keys_to_bopomofo(keys);
                            if !z.is_empty() {
                                return Some(z);
                            }
                        }
                    }
                    None
                })
                .collect();

            if syllables.is_empty() {
                eprintln!(
                    "migao: '{}' does not look like Bopomofo (大千) garbled text",
                    garbled
                );
                std::process::exit(1);
            }

            let key = syllables.concat();
            match migao::user_data::append_entry(&key, &correct) {
                Ok(()) => println!("✓  {} → {}", key, correct),
                Err(e) => {
                    eprintln!("migao: failed to write user supplement: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}
