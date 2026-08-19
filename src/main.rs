use clap::{Parser, Subcommand, ValueEnum};
use std::io::{self, IsTerminal, Read, Write};

#[derive(Parser)]
#[command(
    name = "migao",
    about = "IME garbled text recovery — 翻譯米糕",
    after_help = "Examples:
  migao fix \"su3cl3\"
  migao status
  migao config set hotkey Ctrl+Alt+K
  migao watch autostart on

From the Windows tray menu, choose \"Open Migao Command\" to open this management entry point."
)]
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
    /// Show install, config, watcher status, and next commands to try
    Status,
    /// Show or update Migao configuration
    #[command(after_help = "Examples:
  migao config
  migao config set hotkey Ctrl+Alt+K

Supported hotkey format: Ctrl+Alt+<A-Z>
Restart migao-watch after changing the hotkey.")]
    Config {
        #[command(subcommand)]
        command: Option<ConfigCommand>,
    },
    /// Manage migao-watch settings
    #[command(after_help = "Examples:
  migao watch autostart on
  migao watch autostart off

Autostart controls the same Launch at Login setting shown in the Windows tray menu.")]
    Watch {
        #[command(subcommand)]
        command: WatchCommand,
    },
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

#[derive(Subcommand)]
enum ConfigCommand {
    /// Update one supported config key
    #[command(after_help = "Examples:
  migao config set hotkey Ctrl+Alt+K
  migao config set hotkey Ctrl+Alt+R

Supported keys:
  hotkey    Ctrl+Alt+<A-Z>; restart migao-watch after changing it.")]
    Set {
        /// Config key. Currently supported: hotkey
        key: String,
        /// New config value
        value: String,
    },
}

#[derive(Subcommand)]
enum WatchCommand {
    /// Enable or disable Launch at Login
    Autostart {
        /// Desired autostart state
        state: ToggleState,
    },
}

#[derive(Clone, ValueEnum)]
enum ToggleState {
    On,
    Off,
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
            println!("  bopomofo-daqian       (aliases: zhuyin, 注音)  — 大千標準注音鍵盤");
            println!("  pinyin                (alias: 拼音)             — 全拼（標準 QWERTY）");
            println!("  english-from-bopomofo (alias: reverse)          — 注音符號 → 原始英文按鍵");
        }

        Commands::Status => print_status(),

        Commands::Config { command } => match command {
            None => print_config(),
            Some(ConfigCommand::Set { key, value }) => set_config_value(&key, &value)?,
        },

        Commands::Watch { command } => match command {
            WatchCommand::Autostart { state } => {
                let enable = matches!(state, ToggleState::On);
                migao::config::set_autostart_enabled(enable)?;
                println!(
                    "migao-watch Launch at Login: {}",
                    if enable { "on" } else { "off" }
                );
            }
        },

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

fn print_status() {
    let cfg = migao::config::load();
    let hotkey = migao::config::normalized_hotkey_or_default(&cfg.hotkey);
    println!("Migao v{}", env!("CARGO_PKG_VERSION"));
    println!("Config: {}", config_path_display());
    println!("Default IME: {}", cfg.default_ime);
    println!("Hotkey: {}", hotkey.label);
    println!(
        "Launch at Login: {}",
        if migao::config::is_autostart_enabled() {
            "on"
        } else if cfg!(windows) {
            "off"
        } else {
            "unsupported on this platform"
        }
    );
    if cfg!(windows) {
        println!("Watcher: launch Migao Watch from Start Menu or run migao-watch");
    } else {
        println!("Watcher: migao-watch is currently Windows-only");
    }
    println!();
    println!("Try:");
    println!("  migao config");
    println!("  migao config set hotkey Ctrl+Alt+K");
    println!("  migao watch autostart on");
    println!("  migao watch autostart off");
    if cfg!(windows) {
        println!();
        println!(
            "Tray: right-click Migao Watch and choose Open Migao Command to reopen this view."
        );
    }
}

fn print_config() {
    let cfg = migao::config::load();
    let hotkey = migao::config::normalized_hotkey_or_default(&cfg.hotkey);
    println!("Config: {}", config_path_display());
    println!("hotkey = \"{}\"", hotkey.label);
    println!("default_ime = \"{}\"", cfg.default_ime);
}

fn set_config_value(key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut cfg = migao::config::load();
    match key {
        "hotkey" => {
            let hotkey = migao::config::parse_hotkey(value)?;
            cfg.hotkey = hotkey.label;
            migao::config::save(&cfg)?;
            println!("hotkey = \"{}\"", cfg.hotkey);
            println!("Restart migao-watch for the new hotkey to take effect.");
        }
        _ => {
            eprintln!(
                "migao: unsupported config key '{}'. Supported keys: hotkey",
                key
            );
            std::process::exit(2);
        }
    }
    Ok(())
}

fn config_path_display() -> String {
    migao::config::config_path()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "(unavailable)".to_string())
}
