#![cfg_attr(windows, windows_subsystem = "windows")]

/// migao-watch — Windows background hotkey daemon with system tray
///
/// Registers Ctrl+Alt+R globally. On press:
///   1. If no recent replacement: copy selection, find best candidate, paste.
///   2. If recently replaced (within 3 s): Ctrl+Z undo, paste next candidate.
///   3. Cycles wrap-around: cand1 → cand2 → cand3 → original → cand1 → …
///
/// Tray icon (right-click): Pause / Resume · Exit
#[cfg(windows)]
mod win {
    use std::mem;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::sync::{mpsc, Arc, Mutex};
    use std::thread;
    use std::time::{Duration, Instant};

    use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
    use tray_icon::{TrayIcon, TrayIconBuilder};
    use winapi::shared::minwindef::TRUE;
    use winapi::um::consoleapi::SetConsoleCtrlHandler;
    use winapi::um::processthreadsapi::GetCurrentThreadId;
    use winapi::um::wincon::{CTRL_BREAK_EVENT, CTRL_CLOSE_EVENT, CTRL_C_EVENT};
    use winapi::um::winuser::{
        DispatchMessageW, GetClassNameW, GetForegroundWindow, GetMessageW, KillTimer,
        PostThreadMessageW, RegisterHotKey, SendInput, SetTimer, UnregisterHotKey, INPUT,
        KEYBDINPUT, KEYEVENTF_KEYUP, MOD_ALT, MOD_CONTROL, MSG, WM_APP, WM_HOTKEY, WM_QUIT,
        WM_TIMER,
    };
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE};
    use winreg::RegKey;

    const HOTKEY_ID: i32 = 1;
    const VK_R: u32 = 0x52;
    const VK_CTRL: u16 = 0x11;
    const VK_ALT: u16 = 0x12;
    const VK_C: u16 = 0x43;
    const VK_V: u16 = 0x56;
    const VK_X: u16 = 0x58;
    const VK_Z: u16 = 0x5A;

    const CYCLE_WINDOW: Duration = Duration::from_secs(3);

    static MAIN_THREAD_ID: AtomicU32 = AtomicU32::new(0);
    static PAUSED: AtomicBool = AtomicBool::new(false);

    const REG_RUN: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
    const REG_KEY: &str = "MigaoWatch";

    // Custom message posted by the worker thread when a correction is made.
    const WM_NOTIFY: u32 = WM_APP + 1;
    // Timer ID used to restore the tooltip text after showing the correction.
    const TIMER_TOOLTIP_RESET: usize = 1;

    fn is_autostart_enabled() -> bool {
        let Ok(run) = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(REG_RUN, KEY_READ)
        else {
            return false;
        };
        run.get_raw_value(REG_KEY).is_ok()
    }

    fn set_autostart(enable: bool) {
        let Ok(run) =
            RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(REG_RUN, KEY_SET_VALUE)
        else {
            return;
        };
        if enable {
            if let Ok(exe) = std::env::current_exe() {
                let path = format!("\"{}\"", exe.display());
                let _ = run.set_value(REG_KEY, &path);
            }
        } else {
            let _ = run.delete_value(REG_KEY);
        }
    }

    struct CycleState {
        candidates: Vec<String>,
        original: String,
        in_terminal: bool,
        /// 0..n-1 = candidates[idx], n = original (reverted).
        idx: usize,
        last_at: Instant,
    }

    // ── Console control handler ──────────────────────────────────────────────

    unsafe extern "system" fn ctrl_handler(ctrl_type: u32) -> i32 {
        match ctrl_type {
            CTRL_C_EVENT | CTRL_BREAK_EVENT | CTRL_CLOSE_EVENT => {
                let tid = MAIN_THREAD_ID.load(Ordering::SeqCst);
                if tid != 0 {
                    PostThreadMessageW(tid, WM_QUIT, 0, 0);
                }
                TRUE
            }
            _ => 0,
        }
    }

    // ── Tray icon ────────────────────────────────────────────────────────────

    #[derive(Clone, Copy)]
    enum IconState {
        Active,    // listening — active migao
        Corrected, // just fixed text — uses active migao
        Paused,    // hotkey suspended — paused migao
    }

    fn make_icon(state: IconState) -> tray_icon::Icon {
        let bytes: &[u8] = match state {
            IconState::Active | IconState::Corrected => {
                include_bytes!("../../assets/tray_active_16x16.png")
            }
            IconState::Paused => include_bytes!("../../assets/tray_paused_16x16.png"),
        };
        let img = image::load_from_memory(bytes)
            .expect("icon load failed")
            .to_rgba8();
        let (w, h) = img.dimensions();
        tray_icon::Icon::from_rgba(img.into_raw(), w, h).expect("icon creation failed")
    }

    // ── Atomic key injection ─────────────────────────────────────────────────

    fn ki(vk: u16, flags: u32) -> INPUT {
        let mut input = INPUT {
            type_: 1,
            u: unsafe { mem::zeroed() },
        };
        unsafe {
            *input.u.ki_mut() = KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            };
        }
        input
    }

    fn batch(inputs: &mut Vec<INPUT>) {
        unsafe {
            SendInput(
                inputs.len() as u32,
                inputs.as_mut_ptr(),
                mem::size_of::<INPUT>() as i32,
            );
        }
    }

    fn ctrl_c() {
        batch(&mut vec![
            ki(VK_ALT, KEYEVENTF_KEYUP),
            ki(VK_CTRL, 0),
            ki(VK_C, 0),
            ki(VK_C, KEYEVENTF_KEYUP),
            ki(VK_ALT, 0),
        ]);
    }

    fn ctrl_v() {
        batch(&mut vec![
            ki(VK_ALT, KEYEVENTF_KEYUP),
            ki(VK_CTRL, 0),
            ki(VK_V, 0),
            ki(VK_V, KEYEVENTF_KEYUP),
            ki(VK_ALT, 0),
        ]);
    }

    fn ctrl_z() {
        batch(&mut vec![
            ki(VK_ALT, KEYEVENTF_KEYUP),
            ki(VK_CTRL, 0),
            ki(VK_Z, 0),
            ki(VK_Z, KEYEVENTF_KEYUP),
            ki(VK_ALT, 0),
        ]);
    }

    fn ctrl_x() {
        batch(&mut vec![
            ki(VK_ALT, KEYEVENTF_KEYUP),
            ki(VK_CTRL, 0),
            ki(VK_X, 0),
            ki(VK_X, KEYEVENTF_KEYUP),
            ki(VK_ALT, 0),
        ]);
    }

    // Release any synthetic Ctrl/Alt that remain pressed after SendInput batches.
    // Called after every complete hotkey action so modifier keys never get stuck.
    fn release_modifiers() {
        batch(&mut vec![
            ki(VK_CTRL, KEYEVENTF_KEYUP),
            ki(VK_ALT, KEYEVENTF_KEYUP),
        ]);
    }

    /// Returns true if the foreground window is a console/terminal host.
    /// Windows Terminal: "CASCADIA_HOSTING_WINDOW_CLASS"
    /// Legacy conhost (cmd, old PowerShell): "ConsoleWindowClass"
    fn is_terminal_foreground() -> bool {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_null() {
                return false;
            }
            let mut buf = [0u16; 256];
            let len = GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
            if len <= 0 {
                return false;
            }
            let class = String::from_utf16_lossy(&buf[..len as usize]);
            matches!(
                class.as_str(),
                "CASCADIA_HOSTING_WINDOW_CLASS" | "ConsoleWindowClass"
            )
        }
    }

    fn paste(clipboard: &mut arboard::Clipboard, text: &str) -> bool {
        if clipboard.set_text(text).is_err() {
            return false;
        }
        thread::sleep(Duration::from_millis(50));
        ctrl_v();
        true
    }

    // ── Main handler ─────────────────────────────────────────────────────────

    // Returns a one-line correction summary for the tray tooltip, or None if
    // nothing was corrected (no match, already correct, or cycle action).
    // Always releases synthetic Ctrl/Alt before returning to prevent stuck keys.
    fn handle_hotkey(
        clipboard: &mut arboard::Clipboard,
        cycle: &mut Option<CycleState>,
    ) -> Option<String> {
        let result = handle_hotkey_inner(clipboard, cycle);
        // Guarantee Ctrl and Alt are released regardless of which path was taken.
        // Without this, apps like VS Code see subsequent keypresses as Ctrl+Alt+X
        // shortcuts because the synthetic modifiers from SendInput stay "pressed".
        release_modifiers();
        result
    }

    fn handle_hotkey_inner(
        clipboard: &mut arboard::Clipboard,
        cycle: &mut Option<CycleState>,
    ) -> Option<String> {
        // ── Cycle mode ───────────────────────────────────────────────────────
        if let Some(state) = cycle.as_mut() {
            if state.last_at.elapsed() < CYCLE_WINDOW {
                let was_original = state.idx >= state.candidates.len();
                state.idx = (state.idx + 1) % (state.candidates.len() + 1);

                if !was_original {
                    ctrl_z();
                    thread::sleep(Duration::from_millis(80));
                }

                if state.idx < state.candidates.len() {
                    let chosen = state.candidates[state.idx].clone();
                    let total = state.candidates.len();
                    if was_original && state.in_terminal {
                        // Terminal: original was pasted back explicitly; cut it first.
                        ctrl_x();
                        thread::sleep(Duration::from_millis(80));
                    }
                    eprintln!("migao-watch: cycle {}/{}  {}", state.idx, total, chosen);
                    paste(clipboard, &chosen).then(|| state.last_at = Instant::now());
                    let preview = truncate(&chosen, 25);
                    return Some(format!("Candidate {}/{}: {preview}", state.idx + 1, total));
                } else {
                    eprintln!("migao-watch: reverted to original");
                    if state.in_terminal {
                        // Terminal: ctrl_z only undid the paste; text is empty — restore explicitly.
                        let original = state.original.clone();
                        paste(clipboard, &original).then(|| state.last_at = Instant::now());
                    } else {
                        // Editor: ctrl_z already restored the original text via undo history.
                        state.last_at = Instant::now();
                    }
                    return Some("Reverted to original".to_string());
                }
            }
            *cycle = None;
        }

        // ── Fresh lookup ─────────────────────────────────────────────────────
        // In terminals (PSReadLine), ctrl_c copies but clears the selection, so
        // ctrl_v appends instead of replacing. Use ctrl_x (cut) there so the text
        // is gone before we paste the correction.  In regular editors, ctrl_c keeps
        // the selection live, so ctrl_v replaces it cleanly without touching the
        // undo stack unnecessarily.
        let in_terminal = is_terminal_foreground();
        if in_terminal {
            ctrl_x();
        } else {
            ctrl_c();
        }
        thread::sleep(Duration::from_millis(150));

        let text = match clipboard.get_text() {
            Ok(t) if !t.trim().is_empty() => t,
            _ => return None,
        };

        let rules: &[&str] = &["bopomofo-daqian", "english-from-bopomofo"];
        let mut best_rule: Option<(f32, &str)> = None;

        for &ime in rules {
            if let Some(rule) = migao::rules::get_rule(ime) {
                let conf = rule.confidence(&text);
                if conf >= 0.3 && best_rule.is_none_or(|(c, _)| conf > c) {
                    best_rule = Some((conf, ime));
                }
            }
        }

        let Some((_, ime)) = best_rule else {
            if in_terminal {
                paste(clipboard, &text);
            }
            return None;
        };
        let candidates = migao::recover_top_n(&text, ime, 3);

        if candidates.is_empty() || candidates[0] == text {
            if in_terminal {
                paste(clipboard, &text);
            }
            return None;
        }

        eprintln!(
            "migao-watch: {} candidate(s) for {:?}  → {:?}",
            candidates.len(),
            text,
            candidates[0]
        );

        if paste(clipboard, &candidates[0]) {
            let orig = truncate(&text, 25);
            let fixed = truncate(&candidates[0], 25);
            let summary = format!("{orig} → {fixed}");

            if candidates.len() > 1 {
                *cycle = Some(CycleState {
                    candidates,
                    original: text,
                    in_terminal,
                    idx: 0,
                    last_at: Instant::now(),
                });
            }
            return Some(summary);
        }
        None
    }

    fn truncate(s: &str, max_chars: usize) -> String {
        let mut chars = s.chars();
        let truncated: String = chars.by_ref().take(max_chars).collect();
        if chars.next().is_some() {
            format!("{truncated}…")
        } else {
            truncated
        }
    }

    // ── Entry point ──────────────────────────────────────────────────────────

    pub fn run() {
        unsafe {
            MAIN_THREAD_ID.store(GetCurrentThreadId(), Ordering::SeqCst);
            SetConsoleCtrlHandler(Some(ctrl_handler), TRUE);

            if RegisterHotKey(
                std::ptr::null_mut(),
                HOTKEY_ID,
                (MOD_CONTROL | MOD_ALT) as u32,
                VK_R,
            ) != TRUE
            {
                eprintln!("migao-watch: failed to register Ctrl+Alt+R (already in use?)");
                std::process::exit(1);
            }
        }

        // ── Tray icon ────────────────────────────────────────────────────────
        let pause_item = MenuItem::new("Pause", true, None);
        let login_item = CheckMenuItem::new("Launch at Login", true, is_autostart_enabled(), None);
        let report_item = MenuItem::new("Report Issue", true, None);
        let exit_item = MenuItem::new("Exit", true, None);
        let sep = PredefinedMenuItem::separator();
        let menu = Menu::new();
        menu.append_items(&[&pause_item, &login_item, &report_item, &sep, &exit_item])
            .expect("menu setup failed");
        let tray: TrayIcon = TrayIconBuilder::new()
            .with_icon(make_icon(IconState::Active))
            .with_tooltip("migao-watch — Ctrl+Alt+R to fix")
            .with_menu(Box::new(menu))
            .build()
            .expect("failed to create tray icon");

        let pause_id = pause_item.id().clone();
        let login_id = login_item.id().clone();
        let report_id = report_item.id().clone();
        let exit_id = exit_item.id().clone();

        // ── Worker thread ────────────────────────────────────────────────────
        let (tx, rx) = mpsc::channel::<()>();
        let notif_slot: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let notif_writer = Arc::clone(&notif_slot);

        thread::spawn(move || {
            // Pre-warm dictionaries before the first hotkey press.
            let _ = migao::recover_top_n("su3cl3", "bopomofo-daqian", 1);
            let _ = migao::recover_top_n("su3cl3", "english-from-bopomofo", 1);

            let mut clipboard = arboard::Clipboard::new().expect("clipboard init failed");
            let mut cycle: Option<CycleState> = None;

            for () in rx {
                thread::sleep(Duration::from_millis(60));
                if let Some(summary) = handle_hotkey(&mut clipboard, &mut cycle) {
                    *notif_writer.lock().unwrap() = Some(summary);
                    let tid = MAIN_THREAD_ID.load(Ordering::SeqCst);
                    unsafe { PostThreadMessageW(tid, WM_NOTIFY, 0, 0) };
                }
            }
        });

        eprintln!("migao-watch running. Right-click the tray icon to pause or exit.");

        // ── Message loop ─────────────────────────────────────────────────────
        let mut should_quit = false;
        unsafe {
            let mut msg: MSG = mem::zeroed();
            while !should_quit {
                let ret = GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0);
                if ret == 0 || ret == -1 {
                    break;
                }
                if msg.message == WM_HOTKEY
                    && msg.wParam as i32 == HOTKEY_ID
                    && !PAUSED.load(Ordering::Relaxed)
                {
                    let _ = tx.send(());
                } else if msg.message == WM_NOTIFY {
                    if let Some(summary) = notif_slot.lock().unwrap().take() {
                        let tip = format!("✓  {summary}");
                        tray.set_icon(Some(make_icon(IconState::Corrected))).ok();
                        tray.set_tooltip(Some(&tip)).ok();
                        SetTimer(std::ptr::null_mut(), TIMER_TOOLTIP_RESET, 4000, None);
                    }
                } else if msg.message == WM_TIMER && msg.wParam == TIMER_TOOLTIP_RESET {
                    KillTimer(std::ptr::null_mut(), TIMER_TOOLTIP_RESET);
                    tray.set_icon(Some(make_icon(IconState::Active))).ok();
                    tray.set_tooltip(Some("migao-watch — Ctrl+Alt+R to fix"))
                        .ok();
                }
                DispatchMessageW(&msg);

                // Tray menu events are enqueued when the hidden tray window
                // processes its messages via DispatchMessageW above.
                while let Ok(event) = MenuEvent::receiver().try_recv() {
                    if event.id == exit_id {
                        should_quit = true;
                    } else if event.id == report_id {
                        std::process::Command::new("cmd")
                            .args(["/c", "start", "", "https://github.com/winterdrive/migao/issues/new"])
                            .spawn()
                            .ok();
                    } else if event.id == login_id {
                        let enable = !is_autostart_enabled();
                        set_autostart(enable);
                        login_item.set_checked(enable);
                    } else if event.id == pause_id {
                        let now_paused = !PAUSED.load(Ordering::Relaxed);
                        PAUSED.store(now_paused, Ordering::Relaxed);
                        pause_item.set_text(if now_paused { "Resume" } else { "Pause" });
                        tray.set_icon(Some(make_icon(if now_paused {
                            IconState::Paused
                        } else {
                            IconState::Active
                        })))
                        .ok();
                        tray.set_tooltip(Some(if now_paused {
                            "migao-watch — paused"
                        } else {
                            "migao-watch — Ctrl+Alt+R to fix"
                        }))
                        .ok();
                    }
                }
            }

            UnregisterHotKey(std::ptr::null_mut(), HOTKEY_ID);
        }
        // tx drops → worker exits; tray drops → icon removed from taskbar.
        drop(tray);
        eprintln!("migao-watch: stopped.");
    }
}

#[cfg(windows)]
fn main() {
    win::run();
}

#[cfg(not(windows))]
fn main() {
    eprintln!("migao-watch is currently Windows-only.");
    eprintln!("On macOS/Linux, pipe text: echo \"su3cl3\" | migao fix");
    std::process::exit(1);
}
