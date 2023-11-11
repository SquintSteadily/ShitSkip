use std::thread::sleep;
use std::time::Duration;

use rand::{thread_rng, Rng};

use image::RgbaImage;
use win_screenshot::prelude::*;

use windows::core::w;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

#[allow(unused)]
use opencv::prelude::*;
use opencv::{core, imgcodecs, imgproc, Result};

fn get_window_position(hwnd: HWND) -> Option<(i32, i32)> {
    let mut rect = RECT::default();
    unsafe {
        if GetWindowRect(hwnd, &mut rect).is_ok() {
            let x = rect.left;
            let y = rect.top;
            return Some((x, y));
        }
    }
    None
}

fn get_genshin_hwnd() -> HWND {
    let genshin_hwnd = unsafe { FindWindowW(None, w!("原神")) };
    if genshin_hwnd.0 == 0 {
        panic!("找不到原神窗口")
    } else {
        println!("获取原神窗口句柄: {:?}", genshin_hwnd);
    }
    genshin_hwnd
}

fn capture(genshin_hwnd: HWND) -> Result<()> {
    if let Ok(buf) = capture_window(genshin_hwnd.0) {
        if let Some(img) = RgbaImage::from_raw(buf.width, buf.height, buf.pixels) {
            if img.save("screenshot.jpg").is_ok() {
                println!("截图成功");
                return Ok(());
            }
        }
    }
    println!("截图失败，请用管理员权限运行");
    Err(opencv::Error::new(1, "Capture failed"))
}

fn match_template(target: &str, template: &str) -> Option<core::Point_<i32>> {
    let src = imgcodecs::imread(target, imgcodecs::IMREAD_COLOR).unwrap();
    let templ = imgcodecs::imread(template, imgcodecs::IMREAD_COLOR).unwrap();
    let mut result = core::Mat::default();
    let mask = core::Mat::default();
    if imgproc::match_template(&src, &templ, &mut result, imgproc::TM_CCOEFF_NORMED, &mask).is_err()
    {
        return None;
    }

    let mut min_val = 0.0;
    let mut max_val = 0.0;
    let mut min_loc = core::Point::default();
    let mut max_loc = core::Point::default();
    if core::min_max_loc(
        &result,
        Some(&mut min_val),
        Some(&mut max_val),
        Some(&mut min_loc),
        Some(&mut max_loc),
        &mask,
    )
    .is_err()
    {
        return None;
    }

    let threshold = 0.99; // 设置阈值，todo: 从配置中读取阈值
    let match_loc = if max_val > threshold {
        max_loc
    } else {
        core::Point::default()
    };

    if match_loc == core::Point::default() {
        return None;
    }

    Some(match_loc)
}

fn match_dotdotdot() -> Option<core::Point_<i32>> {
    // todo: 从配置读取
    match_template("screenshot.jpg", "resources/2560x1440/dotdotdot.jpg")
}

fn match_enter() -> Option<core::Point_<i32>> {
    // todo: 从配置读取
    match_template("screenshot.jpg", "resources/2560x1440/enter.jpg")
}

fn click_dotdotdot(genshin_hwnd: HWND, point: core::Point_<i32>) -> Result<()> {
    let mut rng = thread_rng();
    let x = point.x + rng.gen_range(0..100);
    let y = point.y + rng.gen_range(0..30);
    let (x_0, y_0) = get_window_position(genshin_hwnd).unwrap();
    // let cur_hwnd: HWND = unsafe { GetForegroundWindow() };
    let lparam = ((y << 16) | (x & 0xffff)) as isize;
    unsafe {
        // 设置原神窗口为前台窗口
        SetForegroundWindow(genshin_hwnd);
        // 把光标移动到按钮位置
        SetCursorPos(x_0 + x, y_0 + y).unwrap();
        // 按下鼠标左键
        SendMessageW(genshin_hwnd, WM_LBUTTONDOWN, WPARAM(1), LPARAM(lparam));
        // 等待 0.2s
        let mut rng = thread_rng();
        sleep(Duration::from_millis(150 + rng.gen_range(0..100)));
        // 抬起鼠标左键
        SendMessageW(genshin_hwnd, WM_LBUTTONUP, WPARAM(1), LPARAM(lparam));
    };
    Ok(())
}

fn press_space(genshin_hwnd: HWND) -> Result<()> {
    unsafe {
        // 设置原神窗口为前台窗口
        SetForegroundWindow(genshin_hwnd);
        // 按下 SPACE 键
        SendMessageW(genshin_hwnd, WM_KEYDOWN, WPARAM(VK_SPACE.0 as usize), LPARAM(1));
        // 等待 0.2s
        let mut rng = thread_rng();
        sleep(Duration::from_millis(150 + rng.gen_range(0..100)));
        // // 抬起 SPACE 键
        let lparam = (1 | (1 << 30) | (1 << 31)) as isize;
        SendMessageW(genshin_hwnd, WM_KEYUP, WPARAM(VK_SPACE.0 as usize), LPARAM(lparam));
    };
    Ok(())
}

fn main() {
    let genshin_hwnd: HWND = get_genshin_hwnd();
    loop {
        if capture(genshin_hwnd).is_err() {
            sleep(Duration::from_millis(1000));
            continue;
        }
        if let Some(point) = match_dotdotdot() {
            println!("检测到选项，正在点击");
            click_dotdotdot(genshin_hwnd, point).unwrap();
        } else if match_enter().is_some() {
            unsafe { SetForegroundWindow(genshin_hwnd) };
            println!("剧情结束");
            break;
        } else {
            println!("按下 SPACE 键");
            press_space(genshin_hwnd).unwrap();
        }
        // todo: 从配置读取
        println!("等待 3 秒");
        let mut rng = thread_rng();
        sleep(Duration::from_millis(2500 + rng.gen_range(0..666)));
    }
}
