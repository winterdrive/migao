//! One-shot probe: can we switch Microsoft Bopomofo from English mode back to
//! Chinese mode from another process?
//!
//! Steps it performs automatically after a 5-second countdown:
//!   1. read conversion mode via WM_IME_CONTROL (ImmGetDefaultIMEWnd)
//!   2. try to set Chinese mode via IMC_SETCONVERSIONMODE / IMC_SETOPENSTATUS
//!   3. read back; if unchanged, inject a single Shift as fallback
//!
//! User procedure (one action only):
//!   open Notepad, switch Bopomofo to 英 (English mode), run
//!   `cargo run --example ime_test`, click into Notepad, wait.
//!   Then report whether the indicator ended up 中 or 英, plus this output.

#[cfg(windows)]
fn main() {
    use std::io::Write as _;
    use std::{mem, thread, time::Duration};
    use winapi::shared::basetsd::DWORD_PTR;
    use winapi::shared::minwindef::{LPARAM, WPARAM};
    use winapi::shared::windef::HWND;
    use winapi::um::winuser::{
        GetClassNameW, GetForegroundWindow, GetKeyboardLayout, GetWindowThreadProcessId, SendInput,
        SendMessageTimeoutW, INPUT, INPUT_KEYBOARD, KEYEVENTF_KEYUP, SMTO_ABORTIFHUNG,
    };

    #[link(name = "imm32")]
    extern "system" {
        fn ImmGetDefaultIMEWnd(hwnd: HWND) -> HWND;
    }

    const WM_IME_CONTROL: u32 = 0x0283;
    const IMC_GETCONVERSIONMODE: WPARAM = 0x0001;
    const IMC_SETCONVERSIONMODE: WPARAM = 0x0002;
    const IMC_GETOPENSTATUS: WPARAM = 0x0005;
    const IMC_SETOPENSTATUS: WPARAM = 0x0006;
    const IME_CMODE_NATIVE: DWORD_PTR = 0x0001;
    const VK_SHIFT: u16 = 0x10;

    unsafe fn ime_msg(ime_wnd: HWND, cmd: WPARAM, lparam: LPARAM) -> Option<DWORD_PTR> {
        let mut result: DWORD_PTR = 0;
        let ok = SendMessageTimeoutW(
            ime_wnd,
            WM_IME_CONTROL,
            cmd,
            lparam,
            SMTO_ABORTIFHUNG,
            500,
            &mut result,
        );
        if ok == 0 {
            None
        } else {
            Some(result)
        }
    }

    unsafe fn inject_shift() {
        let mut inputs: [INPUT; 2] = mem::zeroed();
        for (i, input) in inputs.iter_mut().enumerate() {
            input.type_ = INPUT_KEYBOARD;
            let ki = input.u.ki_mut();
            ki.wVk = VK_SHIFT;
            ki.dwFlags = if i == 0 { 0 } else { KEYEVENTF_KEYUP };
        }
        SendInput(2, inputs.as_mut_ptr(), mem::size_of::<INPUT>() as i32);
    }

    println!("請確認：記事本已開啟，微軟注音在「英」模式。");
    println!("倒數 5 秒內請點進記事本視窗，之後不用做任何事。");
    for i in (1..=5u64).rev() {
        print!("  {i}...");
        std::io::stdout().flush().ok();
        thread::sleep(Duration::from_secs(1));
    }
    println!();

    unsafe {
        let hwnd = GetForegroundWindow();
        let mut buf = [0u16; 256];
        let len = GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
        let class = String::from_utf16_lossy(&buf[..len.max(0) as usize]);
        let tid = GetWindowThreadProcessId(hwnd, std::ptr::null_mut());
        let hkl = GetKeyboardLayout(tid) as usize;
        let ime_wnd = ImmGetDefaultIMEWnd(hwnd);
        println!("前景視窗 class={class:?}  HKL={hkl:#010x}  ime_wnd={ime_wnd:?}");

        let conv_before = ime_msg(ime_wnd, IMC_GETCONVERSIONMODE, 0);
        let open_before = ime_msg(ime_wnd, IMC_GETOPENSTATUS, 0);
        println!("讀取: conv={conv_before:?} open={open_before:?}");

        let conv = conv_before.unwrap_or(0);
        let r1 = ime_msg(
            ime_wnd,
            IMC_SETCONVERSIONMODE,
            (conv | IME_CMODE_NATIVE) as LPARAM,
        );
        let r2 = ime_msg(ime_wnd, IMC_SETOPENSTATUS, 1);
        println!("設定: SETCONVERSIONMODE={r1:?} SETOPENSTATUS={r2:?}");

        thread::sleep(Duration::from_millis(500));
        let conv_after = ime_msg(ime_wnd, IMC_GETCONVERSIONMODE, 0);
        let open_after = ime_msg(ime_wnd, IMC_GETOPENSTATUS, 0);
        println!("讀回: conv={conv_after:?} open={open_after:?}");

        let message_method_worked = match (conv_before, conv_after) {
            (Some(b), Some(a)) => (b & IME_CMODE_NATIVE) == 0 && (a & IME_CMODE_NATIVE) != 0,
            _ => false,
        };

        if message_method_worked {
            println!("結果: 訊息法看起來有效，未注入 Shift。");
        } else {
            println!("結果: 訊息法無效或讀不到，3 秒後注入 Shift 作為備案...");
            thread::sleep(Duration::from_secs(3));
            inject_shift();
            thread::sleep(Duration::from_millis(500));
            let conv_final = ime_msg(ime_wnd, IMC_GETCONVERSIONMODE, 0);
            println!("Shift 注入後讀回: conv={conv_final:?}");
        }
    }

    println!();
    println!(">>> 現在請看右下角語言列：是「中」還是「英」？ <<<");
    println!("把這個答案＋上面全部輸出貼回給我。15 秒後自動關閉...");
    thread::sleep(Duration::from_secs(15));
}

#[cfg(not(windows))]
fn main() {
    eprintln!("Windows only");
}
