#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod spaces;

use commands::{cmd_switch_to_space, get_space_names, get_spaces, set_space_name};
use tauri::{
    image::Image,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WebviewWindow,
};

/// Build a 22×22 RGBA tray icon: four rounded squares in a 2×2 grid.
fn tray_icon() -> Image<'static> {
    const W: usize = 22;
    const H: usize = 22;
    let mut rgba = vec![0u8; W * H * 4];

    let sq: usize = 7;
    let gap: usize = 2;
    let margin: usize = 3;
    let r: i32 = 2; // corner radius

    let origins = [
        (margin, margin),
        (margin + sq + gap, margin),
        (margin, margin + sq + gap),
        (margin + sq + gap, margin + sq + gap),
    ];

    for (sx, sy) in origins {
        for dy in 0..sq {
            for dx in 0..sq {
                // Rounded corner check
                let cx = dx as i32;
                let cy = dy as i32;
                let sq_i = sq as i32;
                let is_corner = (cx < r && cy < r && (r - cx - 1).pow(2) + (r - cy - 1).pow(2) > (r - 1).pow(2))
                    || (cx >= sq_i - r && cy < r && (cx - (sq_i - r)).pow(2) + (r - cy - 1).pow(2) > (r - 1).pow(2))
                    || (cx < r && cy >= sq_i - r && (r - cx - 1).pow(2) + (cy - (sq_i - r)).pow(2) > (r - 1).pow(2))
                    || (cx >= sq_i - r && cy >= sq_i - r && (cx - (sq_i - r)).pow(2) + (cy - (sq_i - r)).pow(2) > (r - 1).pow(2));

                if !is_corner {
                    let px = sx + dx;
                    let py = sy + dy;
                    if px < W && py < H {
                        let i = (py * W + px) * 4;
                        rgba[i] = 0;
                        rgba[i + 1] = 0;
                        rgba[i + 2] = 0;
                        rgba[i + 3] = 255;
                    }
                }
            }
        }
    }

    Image::new_owned(rgba, W as u32, H as u32)
}

fn position_window_near_tray(window: &WebviewWindow, click_x: f64, click_y: f64) {
    const WINDOW_WIDTH_LOGICAL: f64 = 280.0;

    let scale = window
        .primary_monitor()
        .ok()
        .flatten()
        .map(|m| m.scale_factor())
        .unwrap_or(1.0);

    let screen_width_physical = window
        .primary_monitor()
        .ok()
        .flatten()
        .map(|m| m.size().width as f64)
        .unwrap_or(1440.0 * scale);

    let win_width_physical = WINDOW_WIDTH_LOGICAL * scale;

    let x = (click_x - win_width_physical / 2.0)
        .max(0.0)
        .min(screen_width_physical - win_width_physical);

    // Place just below the menu bar
    let y = click_y + 4.0;

    let _ = window.set_position(tauri::PhysicalPosition::new(x as i32, y as i32));
}

fn toggle_window(app: &AppHandle, click_x: f64, click_y: f64) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else {
        position_window_near_tray(&window, click_x, click_y);
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let app_handle = app.handle().clone();
            TrayIconBuilder::new()
                .icon(tray_icon())
                .icon_as_template(true)
                .tooltip("Landmark — Space Manager")
                .show_menu_on_left_click(false)
                .on_tray_icon_event(move |_tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        position,
                        ..
                    } = event
                    {
                        toggle_window(&app_handle, position.x, position.y);
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_spaces,
            cmd_switch_to_space,
            get_space_names,
            set_space_name,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
