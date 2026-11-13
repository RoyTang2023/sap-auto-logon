//! 任务栏图标：按 SAP 系统标识上色、客户端号居中。
//!
//! SAP GUI 的所有会话窗口同属一个进程、一个窗口类，任务栏默认取的是**类图标**，
//! 所以开几个系统都长得一模一样。给窗口单独发 `WM_SETICON` 之后，任务栏按钮
//! 就会按窗口（也就是按系统标识 + 客户端那一组）取图标了。
//!
//! 只在启动时和每次登录后各扫一次：同一组的按钮共用图标，第一个窗口设过就够了，
//! 没必要轮询。

use std::collections::HashMap;
use std::sync::Mutex;

use windows_sys::Win32::Foundation::{HWND, LPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    CreateBitmap, CreateDIBSection, CreateFontW, DeleteObject, DrawTextW, SelectObject,
    SetBkMode, SetTextColor, ANTIALIASED_QUALITY, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
    CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_PITCH, DIB_RGB_COLORS, FF_DONTCARE,
    OUT_TT_PRECIS, TRANSPARENT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateIconIndirect, EnumWindows, GetClassNameW, GetWindowTextW, SendMessageW, ICONINFO,
};

/// SAP 会话窗口的类名，所有系统都用这一个。
const SESSION_CLASS: &str = "SAP_FRONTEND_SESSION";
const WM_SETICON: u32 = 0x0080;
const ICON_SMALL: usize = 0;
const ICON_BIG: usize = 1;
/// 图标画多大。任务栏按钮在小/中尺寸下都够用，系统再缩小也不会糊。
const ICON_SIZE: u32 = 32;

/// SID → 颜色，与卡片上 SID 标签用的是同一套：
/// 开发蓝、质量金、生产红、其余青。图标背景取这组饱和色，白字才压得住。
fn sid_color(sid: &str) -> (u8, u8, u8) {
    match sid.chars().next().map(|c| c.to_ascii_uppercase()) {
        Some('D') => (0x1F, 0x4E, 0x8C),
        Some('Q') => (0x7A, 0x54, 0x10),
        Some('P') => (0x8E, 0x2F, 0x3C),
        _ => (0x15, 0x5F, 0x5F),
    }
}

/// 已创建的图标句柄，按 `(系统标识, 客户端)` 缓存：重复创建会泄漏 GDI 对象，
/// 而且任务栏在句柄换掉时会闪一下。
static ICONS: Mutex<Option<HashMap<(String, String), isize>>> = Mutex::new(None);

/// 扫一遍现有会话窗口，把缺图标的补上。启动时和每次登录后调用即可。
pub fn apply_session_icons() {
    // 只有 SAP GUI 8.10 且任务栏按「系统标识和客户端」分组时才值得动手：
    // 窗口本来不分组的话，每张图标都一样，画了也没意义。
    if !crate::platform::system::taskbar_grouped_by_system() {
        log::debug!("任务栏图标跳过：SAP GUI 未按系统标识和客户端分组");
        return;
    }

    let sessions = collect_sessions();
    if sessions.is_empty() {
        return;
    }

    let mut guard = match ICONS.lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };
    let icons = guard.get_or_insert_with(HashMap::new);

    for (hwnd, sid, client) in sessions {
        let key = (sid, client);
        let icon = match icons.get(&key) {
            Some(icon) => *icon,
            None => {
                let icon = build_icon(&key.0, &key.1);
                icons.insert(key.clone(), icon);
                icon
            }
        };
        // 小尺寸也用同一张：32px 缩到 16px 的观感比另画一版更好，代码也少一半。
        unsafe {
            SendMessageW(hwnd, WM_SETICON, ICON_BIG, icon as LPARAM);
            SendMessageW(hwnd, WM_SETICON, ICON_SMALL, icon as LPARAM);
        }
    }
}

/// 窗口标题形如 `DS4(2)/100 传输组织器`：系统标识 + 会话号 + 客户端。
fn collect_sessions() -> Vec<(HWND, String, String)> {
    unsafe extern "system" fn callback(hwnd: HWND, lparam: LPARAM) -> i32 {
        let out = &mut *(lparam as *mut Vec<(HWND, String, String)>);

        let mut class = [0u16; 64];
        let class_len = GetClassNameW(hwnd, class.as_mut_ptr(), class.len() as i32).max(0) as usize;
        if String::from_utf16_lossy(&class[..class_len]) != SESSION_CLASS {
            return 1;
        }

        let mut title = [0u16; 512];
        let title_len = GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32).max(0) as usize;
        if let Some((sid, client)) = split_title(&String::from_utf16_lossy(&title[..title_len])) {
            out.push((hwnd, sid, client));
        }
        1
    }

    let mut sessions: Vec<(HWND, String, String)> = Vec::new();
    unsafe {
        EnumWindows(Some(callback), &mut sessions as *mut _ as LPARAM);
    }
    sessions
}

/// `DS4(2)/100 传输组织器` → (`DS4`, `100`)。
fn split_title(title: &str) -> Option<(String, String)> {
    let open = title.find('(')?;
    let sid = title[..open].trim();
    let rest = &title[open + 1..];
    let close = rest.find(')')?;
    let after = rest.get(close + 1..)?.strip_prefix('/')?;
    let client: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
    if sid.is_empty() || client.is_empty() {
        return None;
    }
    Some((sid.to_string(), client))
}

/// `DS4(2)/100 传输组织器` → (`DS4`, `100`)。

fn build_icon(sid: &str, client: &str) -> isize {
    let (r, g, b) = sid_color(sid);
    // 圆角底色走像素绘制（抗锯齿），数字交给 GDI 用系统字体画——点阵字放大后太笨重。
    let mut pixels = render_background(ICON_SIZE, (r, g, b));
    unsafe {
        draw_client_text(&mut pixels, ICON_SIZE, client);
    }

    unsafe {
        // 32bpp 的 DIB 位图，像素顺序是 BGRA（我们已经按这个顺序填好）。
        let color = CreateBitmap(
            ICON_SIZE as i32,
            ICON_SIZE as i32,
            1,
            32,
            pixels.as_ptr() as *const _,
        );
        // mask 全 0：透明度由 color 的 alpha 通道决定。
        let mask = CreateBitmap(ICON_SIZE as i32, ICON_SIZE as i32, 1, 1, std::ptr::null());
        let info = ICONINFO {
            fIcon: 1,
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: mask,
            hbmColor: color,
        };
        let icon = CreateIconIndirect(&info);
        DeleteObject(color as _);
        DeleteObject(mask as _);
        icon as isize
    }
}

/// 圆角方块 + 竖向微渐变 + 内侧高光。返回 BGRA（预乘 alpha）。
fn render_background(size: u32, color: (u8, u8, u8)) -> Vec<u8> {
    let mut pixels = vec![0u8; (size * size * 4) as usize];
    let radius = size as f32 * 0.27;

    for y in 0..size {
        for x in 0..size {
            let coverage = rounded_coverage(x as f32, y as f32, size as f32, radius);
            if coverage <= 0.0 {
                continue;
            }
            // 顶部略亮、底部略暗，纯色块在大面积上太死板。
            let t = y as f32 / size as f32;
            let shade = 1.0 + (0.5 - t) * 0.16;
            let mut rgb = [
                (color.0 as f32 * shade).clamp(0.0, 255.0),
                (color.1 as f32 * shade).clamp(0.0, 255.0),
                (color.2 as f32 * shade).clamp(0.0, 255.0),
            ];

            // 内缩一圈的覆盖差 = 边缘：给边缘 1px 提亮，看起来才有厚度。
            let inner = rounded_coverage(x as f32, y as f32, size as f32, radius - 1.2);
            let rim = (coverage - inner).clamp(0.0, 1.0) * 0.30;
            for channel in &mut rgb {
                *channel = *channel * (1.0 - rim) + 255.0 * rim;
            }

            let index = ((y * size + x) * 4) as usize;
            // 预乘 alpha：否则半透明边缘会出现一圈深色描边。
            pixels[index] = (rgb[2] as f32 * coverage).round() as u8;
            pixels[index + 1] = (rgb[1] as f32 * coverage).round() as u8;
            pixels[index + 2] = (rgb[0] as f32 * coverage).round() as u8;
            pixels[index + 3] = (coverage * 255.0).round() as u8;
        }
    }

    pixels
}

/// 把客户端号用系统字体画进像素缓冲：GDI 写字比点阵干净，字号也能精确控制。
unsafe fn draw_client_text(pixels: &mut [u8], size: u32, client: &str) {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Graphics::Gdi::{
        CreateCompatibleDC, DeleteDC, GetDC, ReleaseDC,
    };

    let screen = GetDC(std::ptr::null_mut());
    let memory = CreateCompatibleDC(screen);

    let mut info: BITMAPINFO = std::mem::zeroed();
    info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    info.bmiHeader.biWidth = size as i32;
    // 负高度 = 自上而下，像素顺序和我们的一致。
    info.bmiHeader.biHeight = -(size as i32);
    info.bmiHeader.biPlanes = 1;
    info.bmiHeader.biBitCount = 32;
    info.bmiHeader.biCompression = BI_RGB;
    let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
    let dib = CreateDIBSection(
        memory,
        &info,
        DIB_RGB_COLORS,
        &mut bits,
        std::ptr::null_mut(),
        0,
    );
    if dib.is_null() || bits.is_null() {
        DeleteDC(memory);
        ReleaseDC(std::ptr::null_mut(), screen);
        return;
    }

    std::ptr::copy_nonoverlapping(pixels.as_ptr(), bits as *mut u8, pixels.len());

    let old_bitmap = SelectObject(memory, dib as _);
    let font_name: Vec<u16> = std::ffi::OsStr::new("Segoe UI")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    // 18px 的粗体：三位数字在 32px 图标里占满中间，两边还留得下一点呼吸。
    let font = CreateFontW(
        -18,
        0,
        0,
        0,
        700,
        0,
        0,
        0,
        DEFAULT_CHARSET as u32,
        OUT_TT_PRECIS as u32,
        CLIP_DEFAULT_PRECIS as u32,
        ANTIALIASED_QUALITY as u32,
        DEFAULT_PITCH as u32 | FF_DONTCARE as u32,
        font_name.as_ptr(),
    );
    let old_font = SelectObject(memory, font as _);

    SetBkMode(memory, TRANSPARENT as i32);
    SetTextColor(memory, 0x00FF_FFFF); // COLORREF 是 BGR

    let text: Vec<u16> = client
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let mut rect = windows_sys::Win32::Foundation::RECT {
        left: 0,
        top: 0,
        right: size as i32,
        bottom: size as i32,
    };
    // DT_CENTER | DT_VCENTER | DT_SINGLELINE
    DrawTextW(memory, text.as_ptr(), -1, &mut rect, 0x0000_0001 | 0x0000_0004 | 0x0000_0020);

    std::ptr::copy_nonoverlapping(bits as *const u8, pixels.as_mut_ptr(), pixels.len());

    SelectObject(memory, old_font);
    SelectObject(memory, old_bitmap);
    DeleteObject(font as _);
    DeleteObject(dib as _);
    DeleteDC(memory);
    ReleaseDC(std::ptr::null_mut(), screen);
}

/// 圆角矩形的覆盖率，每像素 4×4 超采样，边缘就不会有硬锯齿。
fn rounded_coverage(x: f32, y: f32, size: f32, radius: f32) -> f32 {
    const SAMPLES: u32 = 4;
    let mut hits = 0;
    for sy in 0..SAMPLES {
        for sx in 0..SAMPLES {
            let px = x + (sx as f32 + 0.5) / SAMPLES as f32;
            let py = y + (sy as f32 + 0.5) / SAMPLES as f32;
            if inside_round_rect(px, py, size, radius) {
                hits += 1;
            }
        }
    }
    hits as f32 / (SAMPLES * SAMPLES) as f32
}

fn inside_round_rect(x: f32, y: f32, size: f32, radius: f32) -> bool {
    if x < 0.0 || y < 0.0 || x > size || y > size {
        return false;
    }
    let radius = radius.max(0.0).min(size / 2.0);
    let cx = x.clamp(radius, size - radius);
    let cy = y.clamp(radius, size - radius);
    let dx = x - cx;
    let dy = y - cy;
    dx * dx + dy * dy <= radius * radius
}


