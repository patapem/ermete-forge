use chrono::Local;
use glib::clone;
use gtk4::gdk::Display;
use gtk4::gio::AppInfo;
use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, Calendar, CenterBox, CssProvider,
    Entry, Label, Orientation, ProgressBar, Scale, ScrolledWindow,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::env;
use std::process::Command;

const APP_ID: &str = "os.ermete.Shell";

const TOPBAR_CSS: &str = r#"
/* ==========================================
   ERMETE OS - AUTHENTIC macOS MENU BAR 1:1
   ========================================== */

window.topbar-window {
    background-color: transparent;
}

.topbar-container {
    background: rgba(22, 22, 25, 0.88);
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    color: #f5f5f7;
    font-family: -apple-system, 'SF Pro Text', 'Inter', sans-serif;
    font-size: 13px;
    font-weight: 500;
    padding: 0 10px;
}

/* Authentic macOS Menu Bar Button Hover Style (Flat, rounded 5px rect on hover) */
.macos-menu-item {
    background: transparent;
    border: none;
    border-radius: 5px;
    padding: 2px 9px;
    color: #f5f5f7;
    font-size: 13px;
    font-weight: 500;
    transition: background 100ms ease;
}

.macos-menu-item:hover {
    background: rgba(255, 255, 255, 0.14);
    color: #ffffff;
}

.macos-apple-logo {
    font-size: 15px;
    font-weight: 700;
    padding: 2px 8px;
}

.macos-app-title {
    font-weight: 700;
    color: #ffffff;
    padding: 2px 10px;
}

/* macOS Status Items (Right side) */
.macos-status-item {
    background: transparent;
    border: none;
    border-radius: 5px;
    padding: 2px 8px;
    color: #e2e8f0;
    font-size: 13px;
    transition: background 100ms ease;
}

.macos-status-item:hover {
    background: rgba(255, 255, 255, 0.14);
    color: #ffffff;
}

.macos-clock {
    font-weight: 500;
    padding: 2px 9px;
}

/* ==========================================
   macOS SPOTLIGHT MODAL (Win+D)
   ========================================== */
window.spotlight-window {
    background-color: transparent;
}

.spotlight-card {
    background: rgba(28, 28, 32, 0.96);
    border: 1px solid rgba(255, 255, 255, 0.16);
    border-radius: 12px;
    padding: 14px;
    box-shadow: 0 30px 70px rgba(0, 0, 0, 0.80);
}

.spotlight-input {
    background: rgba(255, 255, 255, 0.07);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 8px;
    color: #ffffff;
    font-size: 18px;
    padding: 10px 14px;
}

.spotlight-input:focus {
    border-color: #38bdf8;
}

.spotlight-item {
    background: transparent;
    border: none;
    border-radius: 6px;
    padding: 8px 12px;
    color: #f5f5f7;
    font-size: 14px;
}

.spotlight-item:hover {
    background: rgba(10, 132, 255, 0.70);
    color: #ffffff;
}

/* ==========================================
   macOS CONTROL CENTER POPOVER
   ========================================== */
window.popup-window {
    background-color: transparent;
}

.cc-card {
    background: rgba(28, 28, 32, 0.94);
    border: 1px solid rgba(255, 255, 255, 0.16);
    border-radius: 18px;
    padding: 14px;
    color: #f8fafc;
    box-shadow: 0 25px 60px rgba(0, 0, 0, 0.75);
}

.cc-tile {
    background: rgba(255, 255, 255, 0.07);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 14px;
    padding: 10px;
    transition: background 120ms ease;
}

.cc-tile:hover {
    background: rgba(255, 255, 255, 0.11);
}

.cc-tile-row {
    background: transparent;
    border: none;
    border-radius: 10px;
    padding: 6px 8px;
    color: #f5f5f7;
    transition: background 100ms ease;
}

.cc-tile-row:hover {
    background: rgba(255, 255, 255, 0.08);
}

.cc-circle-blue {
    background: #0a84ff;
    border-radius: 999px;
    min-width: 28px;
    min-height: 28px;
    color: #ffffff;
    font-weight: 700;
}

.cc-circle-indigo {
    background: #5e5ce6;
    border-radius: 999px;
    min-width: 28px;
    min-height: 28px;
    color: #ffffff;
    font-weight: 700;
}

.cc-circle-gray {
    background: rgba(255, 255, 255, 0.18);
    border-radius: 999px;
    min-width: 28px;
    min-height: 28px;
    color: #ffffff;
    font-weight: 700;
}

.cc-label-main {
    font-size: 13px;
    font-weight: 600;
    color: #ffffff;
}

.cc-label-sub {
    font-size: 11px;
    font-weight: 500;
    color: #94a3b8;
}

.cc-tile-slider {
    background: rgba(255, 255, 255, 0.07);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 14px;
    padding: 10px 14px;
}

.cc-slider-icon {
    font-size: 15px;
    color: #f5f5f7;
}

.cc-quick-btn {
    background: rgba(255, 255, 255, 0.07);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 12px;
    padding: 10px 6px;
    color: #f5f5f7;
    font-size: 12px;
    font-weight: 500;
    transition: background 120ms ease;
}

.cc-quick-btn:hover {
    background: rgba(255, 255, 255, 0.14);
    color: #ffffff;
}

.cc-btn {
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 8px;
    padding: 8px 12px;
    color: #e2e8f0;
    font-weight: 500;
}

.cc-btn:hover {
    background: rgba(255, 255, 255, 0.15);
    color: #ffffff;
}

.cc-btn-danger {
    background: rgba(255, 69, 58, 0.25);
    border: 1px solid rgba(255, 69, 58, 0.45);
    border-radius: 8px;
    padding: 8px 12px;
    color: #ff8a80;
    font-weight: 600;
}

.cc-btn-danger:hover {
    background: rgba(255, 69, 58, 0.45);
    color: #ffffff;
}

progressbar.cc-progress-blue trough {
    background: rgba(255, 255, 255, 0.12);
    border-radius: 6px;
    min-height: 8px;
}
progressbar.cc-progress-blue progress {
    background: #0a84ff;
    border-radius: 6px;
    min-height: 8px;
}
progressbar.cc-progress-indigo trough {
    background: rgba(255, 255, 255, 0.12);
    border-radius: 6px;
    min-height: 8px;
}
progressbar.cc-progress-indigo progress {
    background: #5e5ce6;
    border-radius: 6px;
    min-height: 8px;
}
.applet-item {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 10px;
    padding: 8px 12px;
    color: #f8fafc;
}
"#;

// Orologio macOS: "sab 11 lug 14:47"
fn macos_clock_string() -> String {
    Local::now().format("%a %d %b %H:%M").to_string()
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_data(TOPBAR_CSS);
    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

// macOS Spotlight Modal (Win+D / Clic su 🔍)
fn show_spotlight_modal(app: &Application) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Spotlight")
        .css_classes(["spotlight-window"])
        .default_width(620)
        .default_height(420)
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    pop.set_margin(Edge::Top, 140);

    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .css_classes(["spotlight-card"])
        .build();

    let entry = Entry::builder()
        .placeholder_text("Cerca Spotlight...")
        .css_classes(["spotlight-input"])
        .build();

    let scroll = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .min_content_height(300)
        .build();

    let list_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .build();

    for app_info in AppInfo::all() {
        if app_info.should_show() {
            let name = app_info.display_name();
            let btn = Button::builder()
                .label(name.as_str())
                .css_classes(["spotlight-item"])
                .build();
            let app_clone = app_info.clone();
            let pop_clone = pop.clone();
            btn.connect_clicked(move |_| {
                let _ = app_clone.launch(&[], gtk4::gio::AppLaunchContext::NONE);
                pop_clone.close();
            });
            list_box.append(&btn);
        }
    }

    scroll.set_child(Some(&list_box));
    card.append(&entry);
    card.append(&scroll);

    let key_ctrl = gtk4::EventControllerKey::new();
    let pop_esc = pop.clone();
    key_ctrl.connect_key_pressed(move |_, keyval, _, _| {
        if keyval == gtk4::gdk::Key::Escape {
            pop_esc.close();
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    pop.add_controller(key_ctrl);

    pop.set_child(Some(&card));
    pop.present();
    entry.grab_focus();
}

fn build_cc_row(badge_class: &str, icon_glyph: &str, title: &str, sub: &str) -> Button {
    let btn = Button::builder().css_classes(["cc-tile-row"]).build();
    let row_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .valign(Align::Center)
        .build();

    let badge = Label::builder()
        .label(icon_glyph)
        .css_classes([badge_class])
        .valign(Align::Center)
        .halign(Align::Center)
        .build();

    let text_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(1)
        .valign(Align::Center)
        .build();

    let lbl_title = Label::builder()
        .label(title)
        .css_classes(["cc-label-main"])
        .halign(Align::Start)
        .build();
    let lbl_sub = Label::builder()
        .label(sub)
        .css_classes(["cc-label-sub"])
        .halign(Align::Start)
        .build();

    text_box.append(&lbl_title);
    text_box.append(&lbl_sub);

    row_box.append(&badge);
    row_box.append(&text_box);
    btn.set_child(Some(&row_box));
    btn
}

fn build_cc_compact_tile(badge_class: &str, icon_glyph: &str, title: &str) -> Button {
    let btn = Button::builder().css_classes(["cc-tile"]).hexpand(true).build();
    let row_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .valign(Align::Center)
        .build();

    let badge = Label::builder()
        .label(icon_glyph)
        .css_classes([badge_class])
        .valign(Align::Center)
        .halign(Align::Center)
        .build();

    let lbl = Label::builder()
        .label(title)
        .css_classes(["cc-label-main"])
        .halign(Align::Start)
        .build();

    row_box.append(&badge);
    row_box.append(&lbl);
    btn.set_child(Some(&row_box));
    btn
}

fn get_ram_info() -> (String, f64) {
    let mut total_kb = 0.0;
    let mut avail_kb = 0.0;
    if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(val) = line.split_whitespace().nth(1) {
                    total_kb = val.parse::<f64>().unwrap_or(0.0);
                }
            } else if line.starts_with("MemAvailable:") {
                if let Some(val) = line.split_whitespace().nth(1) {
                    avail_kb = val.parse::<f64>().unwrap_or(0.0);
                }
            }
        }
    }
    if total_kb > 0.0 {
        let used_kb = total_kb - avail_kb;
        let frac = (used_kb / total_kb).clamp(0.0, 1.0);
        let used_gb = used_kb / 1048576.0;
        let total_gb = total_kb / 1048576.0;
        (
            format!("{:.1} GB / {:.1} GB ({:.0}%)", used_gb, total_gb, frac * 100.0),
            frac,
        )
    } else {
        ("N/D".to_string(), 0.0)
    }
}

fn get_cpu_load() -> (String, f64) {
    if let Ok(content) = std::fs::read_to_string("/proc/loadavg") {
        if let Some(load_str) = content.split_whitespace().next() {
            let load = load_str.parse::<f64>().unwrap_or(0.0);
            let frac = (load / 4.0).clamp(0.0, 1.0);
            return (format!("Carico 1m: {}", load_str), frac);
        }
    }
    ("N/D".to_string(), 0.0)
}

fn show_system_monitor_modal(app: &Application) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Monitor Risorse")
        .css_classes(["popup-window"])
        .default_width(340)
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    pop.set_anchor(Edge::Top, true);
    pop.set_anchor(Edge::Right, true);
    pop.set_margin(Edge::Top, 34);
    pop.set_margin(Edge::Right, 50);

    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(14)
        .css_classes(["cc-card"])
        .build();

    let header = Label::builder()
        .label("MONITOR RISORSE ERMETE OS")
        .css_classes(["cc-label-sub"])
        .halign(Align::Start)
        .build();

    // CPU Section
    let (cpu_text, cpu_frac) = get_cpu_load();
    let cpu_lbl = Label::builder()
        .label(&format!("Processore (CPU) — {}", cpu_text))
        .css_classes(["cc-label-main"])
        .halign(Align::Start)
        .build();
    let cpu_bar = ProgressBar::builder()
        .fraction(cpu_frac)
        .css_classes(["cc-progress-blue"])
        .build();

    // RAM Section
    let (ram_text, ram_frac) = get_ram_info();
    let ram_lbl = Label::builder()
        .label(&format!("Memoria RAM — {}", ram_text))
        .css_classes(["cc-label-main"])
        .halign(Align::Start)
        .build();
    let ram_bar = ProgressBar::builder()
        .fraction(ram_frac)
        .css_classes(["cc-progress-indigo"])
        .build();

    // System Info
    let sys_info = Label::builder()
        .label("Sistema: Ermete OS (Wayland / Niri)")
        .css_classes(["cc-label-sub"])
        .halign(Align::Start)
        .build();

    let close_btn = Button::builder()
        .label("Chiudi")
        .css_classes(["cc-quick-btn"])
        .build();
    let pop_clone = pop.clone();
    close_btn.connect_clicked(move |_| {
        pop_clone.close();
    });

    card.append(&header);
    card.append(&cpu_lbl);
    card.append(&cpu_bar);
    card.append(&ram_lbl);
    card.append(&ram_bar);
    card.append(&sys_info);
    card.append(&close_btn);

    pop.set_child(Some(&card));
    pop.present();
}

fn show_wifi_popover(app: &Application) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Reti Wi-Fi")
        .css_classes(["popup-window"])
        .default_width(340)
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    pop.set_anchor(Edge::Top, true);
    pop.set_anchor(Edge::Right, true);
    pop.set_margin(Edge::Top, 34);
    pop.set_margin(Edge::Right, 50);

    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .css_classes(["cc-card"])
        .build();

    let header = Label::builder()
        .label("RETI WI-FI RILEVATE")
        .css_classes(["cc-label-sub"])
        .halign(Align::Start)
        .build();

    let list_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .build();

    if let Ok(output) = Command::new("nmcli")
        .args(["-t", "-f", "IN-USE,SSID,SIGNAL", "device", "wifi", "list"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut count = 0;
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 && !parts[1].is_empty() && count < 8 {
                let active = parts[0] == "*";
                let ssid = parts[1];
                let sig = parts[2].parse::<i32>().unwrap_or(50);
                let icon = if sig > 75 {
                    "󰤨"
                } else if sig > 40 {
                    "󰤥"
                } else {
                    "󰤢"
                };

                let item_row = GtkBox::builder()
                    .orientation(Orientation::Horizontal)
                    .spacing(10)
                    .css_classes(["applet-item"])
                    .build();

                let icon_lbl = Label::builder().label(icon).build();
                let ssid_lbl = Label::builder()
                    .label(ssid)
                    .hexpand(true)
                    .halign(Align::Start)
                    .build();
                let status_lbl = Label::builder()
                    .label(if active { "Connesso" } else { "" })
                    .css_classes(["cc-label-sub"])
                    .build();

                item_row.append(&icon_lbl);
                item_row.append(&ssid_lbl);
                item_row.append(&status_lbl);
                list_box.append(&item_row);
                count += 1;
            }
        }
        if count == 0 {
            let no_wifi = Label::builder()
                .label("Nessuna rete Wi-Fi rilevata")
                .css_classes(["cc-label-sub"])
                .build();
            list_box.append(&no_wifi);
        }
    } else {
        let err_lbl = Label::builder()
            .label("Impossibile interrogare NetworkManager")
            .css_classes(["cc-label-sub"])
            .build();
        list_box.append(&err_lbl);
    }

    let close_btn = Button::builder()
        .label("Chiudi")
        .css_classes(["cc-quick-btn"])
        .build();
    let pop_clone = pop.clone();
    close_btn.connect_clicked(move |_| {
        pop_clone.close();
    });

    card.append(&header);
    card.append(&list_box);
    card.append(&close_btn);

    pop.set_child(Some(&card));
    pop.present();
}

fn show_bluetooth_popover(app: &Application) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Bluetooth")
        .css_classes(["popup-window"])
        .default_width(340)
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    pop.set_anchor(Edge::Top, true);
    pop.set_anchor(Edge::Right, true);
    pop.set_margin(Edge::Top, 34);
    pop.set_margin(Edge::Right, 50);

    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .css_classes(["cc-card"])
        .build();

    let header = Label::builder()
        .label("DISPOSITIVI BLUETOOTH")
        .css_classes(["cc-label-sub"])
        .halign(Align::Start)
        .build();

    let list_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .build();

    if let Ok(output) = Command::new("bluetoothctl").arg("devices").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut count = 0;
        for line in stdout.lines() {
            let parts: Vec<&str> = line.splitn(3, ' ').collect();
            if parts.len() >= 3 && count < 8 {
                let name = parts[2];
                let item_row = GtkBox::builder()
                    .orientation(Orientation::Horizontal)
                    .spacing(10)
                    .css_classes(["applet-item"])
                    .build();

                let icon_lbl = Label::builder().label("").build();
                let name_lbl = Label::builder()
                    .label(name)
                    .hexpand(true)
                    .halign(Align::Start)
                    .build();

                item_row.append(&icon_lbl);
                item_row.append(&name_lbl);
                list_box.append(&item_row);
                count += 1;
            }
        }
        if count == 0 {
            let no_bt = Label::builder()
                .label("Nessun dispositivo rilevato")
                .css_classes(["cc-label-sub"])
                .build();
            list_box.append(&no_bt);
        }
    }

    let close_btn = Button::builder()
        .label("Chiudi")
        .css_classes(["cc-quick-btn"])
        .build();
    let pop_clone = pop.clone();
    close_btn.connect_clicked(move |_| {
        pop_clone.close();
    });

    card.append(&header);
    card.append(&list_box);
    card.append(&close_btn);

    pop.set_child(Some(&card));
    pop.present();
}

// macOS Control Center Popover (Clic su ❖)
fn show_control_center_popover(app: &Application) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Control Center")
        .css_classes(["popup-window"])
        .default_width(350)
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    pop.set_anchor(Edge::Top, true);
    pop.set_anchor(Edge::Right, true);
    pop.set_margin(Edge::Top, 34);
    pop.set_margin(Edge::Right, 50);

    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .css_classes(["cc-card"])
        .build();

    // 1. TOP SECTION (Grid a 2 Colonne)
    let top_grid = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();

    // Colonna Sinistra (Connettività)
    let conn_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .css_classes(["cc-tile"])
        .hexpand(true)
        .build();

    let wifi_btn = build_cc_row("cc-circle-blue", "", "Rete Wi-Fi", "Reti Rilevate");
    let app_wifi = app.clone();
    let pop_wifi = pop.clone();
    wifi_btn.connect_clicked(move |_| {
        pop_wifi.close();
        show_wifi_popover(&app_wifi);
    });
    let bt_btn = build_cc_row("cc-circle-blue", "", "Bluetooth", "Dispositivi");
    let app_bt = app.clone();
    let pop_bt = pop.clone();
    bt_btn.connect_clicked(move |_| {
        pop_bt.close();
        show_bluetooth_popover(&app_bt);
    });
    let sys_btn = build_cc_row("cc-circle-blue", "⚙", "Risorse", "Monitor Live");
    let app_sys = app.clone();
    let pop_sys = pop.clone();
    sys_btn.connect_clicked(move |_| {
        pop_sys.close();
        show_system_monitor_modal(&app_sys);
    });

    conn_box.append(&wifi_btn);
    conn_box.append(&bt_btn);
    conn_box.append(&sys_btn);

    // Colonna Destra (2 Card verticali)
    let right_col = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .homogeneous(true)
        .hexpand(true)
        .build();

    let screenshot_tile = build_cc_compact_tile("cc-circle-indigo", "📷", "Screenshot");
    let pop_shot = pop.clone();
    screenshot_tile.connect_clicked(move |_| {
        pop_shot.close();
        let _ = Command::new("niri")
            .args(["msg", "action", "screenshot"])
            .spawn();
    });

    let lock_tile = build_cc_compact_tile("cc-circle-blue", "🔒", "Blocca");
    let pop_lock = pop.clone();
    lock_tile.connect_clicked(move |_| {
        pop_lock.close();
        let _ = Command::new("swaylock").spawn();
    });

    right_col.append(&screenshot_tile);
    right_col.append(&lock_tile);

    top_grid.append(&conn_box);
    top_grid.append(&right_col);

    // 2. MIDDLE SECTION (Slider Apple-Style)
    // Slider Luminosità
    let bright_card = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["cc-tile-slider"])
        .valign(Align::Center)
        .build();
    let bright_icon = Label::builder().label("☀").css_classes(["cc-slider-icon"]).build();
    let bright_slider = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
    bright_slider.set_value(75.0);
    bright_slider.set_hexpand(true);
    bright_slider.connect_value_changed(move |s| {
        let val = s.value() as i32;
        let _ = Command::new("brightnessctl")
            .args(["set", &format!("{}%", val)])
            .spawn();
    });
    bright_card.append(&bright_icon);
    bright_card.append(&bright_slider);

    // Slider Volume Audio
    let audio_card = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["cc-tile-slider"])
        .valign(Align::Center)
        .build();
    let audio_icon = Label::builder().label("🔊").css_classes(["cc-slider-icon"]).build();
    let audio_slider = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
    audio_slider.set_value(80.0);
    audio_slider.set_hexpand(true);
    audio_slider.connect_value_changed(move |s| {
        let val = s.value() as i32;
        let _ = Command::new("wpctl")
            .arg("set-volume")
            .arg("@DEFAULT_AUDIO_SINK@")
            .arg(format!("{}%", val))
            .spawn();
    });
    audio_card.append(&audio_icon);
    audio_card.append(&audio_slider);

    // 3. BOTTOM SECTION (4 Quick Toggles Grid)
    let bottom_grid = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .homogeneous(true)
        .build();

    let dark_btn = Button::builder()
        .label("☾   Scuro")
        .css_classes(["cc-quick-btn"])
        .build();
    dark_btn.connect_clicked(move |_| {
        let _ = Command::new("gsettings")
            .args([
                "set",
                "org.gnome.desktop.interface",
                "color-scheme",
                "prefer-dark",
            ])
            .spawn();
    });

    let standby_btn = Button::builder()
        .label("🖥   Standby")
        .css_classes(["cc-quick-btn"])
        .build();
    let pop_std = pop.clone();
    standby_btn.connect_clicked(move |_| {
        pop_std.close();
        let _ = Command::new("niri")
            .args(["msg", "action", "power-off-monitors"])
            .spawn();
    });

    let mixer_btn = Button::builder()
        .label("🎚️   Mixer")
        .css_classes(["cc-quick-btn"])
        .build();
    mixer_btn.connect_clicked(move |_| {
        let _ = Command::new("foot")
            .args(["--title", "PipeWire Audio Top", "-e", "pw-top"])
            .spawn();
    });

    let term_btn = Button::builder()
        .label(">_   Shell")
        .css_classes(["cc-quick-btn"])
        .build();
    let pop_term = pop.clone();
    term_btn.connect_clicked(move |_| {
        pop_term.close();
        let _ = Command::new("foot").spawn();
    });

    bottom_grid.append(&dark_btn);
    bottom_grid.append(&standby_btn);
    bottom_grid.append(&mixer_btn);
    bottom_grid.append(&term_btn);

    card.append(&top_grid);
    card.append(&bright_card);
    card.append(&audio_card);
    card.append(&bottom_grid);

    let key_ctrl = gtk4::EventControllerKey::new();
    let pop_esc = pop.clone();
    key_ctrl.connect_key_pressed(move |_, keyval, _, _| {
        if keyval == gtk4::gdk::Key::Escape {
            pop_esc.close();
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    pop.add_controller(key_ctrl);

    pop.set_child(Some(&card));
    pop.present();
}

// KDE / Windows Style Start Menu Popover (Clic su ◈)
fn show_start_menu_popover(app: &Application) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Start Menu")
        .css_classes(["popup-window"])
        .default_width(360)
        .default_height(480)
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    pop.set_anchor(Edge::Top, true);
    pop.set_anchor(Edge::Left, true);
    pop.set_margin(Edge::Top, 32);
    pop.set_margin(Edge::Left, 8);

    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .css_classes(["cc-card"])
        .build();

    let title = Label::builder()
        .label("◈  MENU APPLICAZIONI ERMETE OS")
        .css_classes(["cc-title"])
        .build();

    let search = Entry::builder()
        .placeholder_text("Cerca nel menu...")
        .css_classes(["spotlight-input"])
        .build();

    let scroll = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .min_content_height(310)
        .build();

    let list_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .build();

    for app_info in AppInfo::all() {
        if app_info.should_show() {
            let name = app_info.display_name();
            let btn = Button::builder()
                .label(name.as_str())
                .css_classes(["spotlight-item"])
                .build();
            let app_clone = app_info.clone();
            let pop_clone = pop.clone();
            btn.connect_clicked(move |_| {
                let _ = app_clone.launch(&[], gtk4::gio::AppLaunchContext::NONE);
                pop_clone.close();
            });
            list_box.append(&btn);
        }
    }

    scroll.set_child(Some(&list_box));

    // Footer con i pulsanti di sessione in basso stile Windows / KDE
    let footer = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();

    let off_btn = Button::builder()
        .label("⏻  Spegni")
        .css_classes(["cc-btn-danger"])
        .hexpand(true)
        .build();
    off_btn.connect_clicked(move |_| {
        let _ = Command::new("systemctl").arg("poweroff").spawn();
    });

    let reb_btn = Button::builder()
        .label("↻  Riavvia")
        .css_classes(["cc-btn"])
        .hexpand(true)
        .build();
    reb_btn.connect_clicked(move |_| {
        let _ = Command::new("systemctl").arg("reboot").spawn();
    });

    footer.append(&off_btn);
    footer.append(&reb_btn);

    card.append(&title);
    card.append(&search);
    card.append(&scroll);
    card.append(&footer);

    let key_ctrl = gtk4::EventControllerKey::new();
    let pop_esc = pop.clone();
    key_ctrl.connect_key_pressed(move |_, keyval, _, _| {
        if keyval == gtk4::gdk::Key::Escape {
            pop_esc.close();
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    pop.add_controller(key_ctrl);

    pop.set_child(Some(&card));
    pop.present();
    search.grab_focus();
}

// macOS Calendar Popover
fn show_calendar_popover(app: &Application) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Calendar")
        .css_classes(["popup-window"])
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    pop.set_anchor(Edge::Top, true);
    pop.set_anchor(Edge::Right, true);
    pop.set_margin(Edge::Top, 32);
    pop.set_margin(Edge::Right, 10);

    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .css_classes(["cc-card"])
        .build();

    let cal = Calendar::builder().build();
    let close_btn = Button::builder()
        .label("Chiudi")
        .css_classes(["cc-btn"])
        .build();
    close_btn.connect_clicked(clone!(@weak pop => move |_| {
        pop.close();
    }));

    card.append(&cal);
    card.append(&close_btn);
    pop.set_child(Some(&card));
    pop.present();
}

// Left Section: Authentic macOS Menu Bar items (Flat, no pills)
fn build_left_island(app: &Application) -> GtkBox {
    let box_left = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(1)
        .valign(Align::Center)
        .build();

    let apple_logo = Button::builder()
        .label("◈")
        .css_classes(["macos-menu-item", "macos-apple-logo"])
        .build();
    let app_clone = app.clone();
    apple_logo.connect_clicked(move |_| {
        show_start_menu_popover(&app_clone);
    });

    let app_title = Button::builder()
        .label("Ermete OS")
        .css_classes(["macos-menu-item", "macos-app-title"])
        .build();

    let file_item = Button::builder()
        .label("File")
        .css_classes(["macos-menu-item"])
        .build();
    let edit_item = Button::builder()
        .label("Modifica")
        .css_classes(["macos-menu-item"])
        .build();
    let view_item = Button::builder()
        .label("Vista")
        .css_classes(["macos-menu-item"])
        .build();
    let win_item = Button::builder()
        .label("Finestra")
        .css_classes(["macos-menu-item"])
        .build();

    box_left.append(&apple_logo);
    box_left.append(&app_title);
    box_left.append(&file_item);
    box_left.append(&edit_item);
    box_left.append(&view_item);
    box_left.append(&win_item);
    box_left
}

// Right Section: Authentic macOS Dongles/Status Items
fn build_right_island(app: &Application, clock_label: &Label) -> GtkBox {
    let box_right = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(2)
        .valign(Align::Center)
        .build();

    // 1. Battery / Power Dongle (macOS style)
    let batt_item = Button::builder()
        .label("100% 󰁹")
        .css_classes(["macos-status-item"])
        .build();

    // 2. Wi-Fi Dongle (macOS style)
    let wifi_item = Button::builder()
        .label("")
        .css_classes(["macos-status-item"])
        .build();
    wifi_item.connect_clicked(move |_| {
        let _ = Command::new("nm-connection-editor").spawn();
    });

    // 3. Spotlight Dongle (macOS style)
    let spot_item = Button::builder()
        .label("🔍")
        .css_classes(["macos-status-item"])
        .build();
    let app_clone1 = app.clone();
    spot_item.connect_clicked(move |_| {
        show_spotlight_modal(&app_clone1);
    });

    // 4. Control Center Dongle (macOS style)
    let cc_item = Button::builder()
        .label("❖")
        .css_classes(["macos-status-item"])
        .build();
    let app_clone2 = app.clone();
    cc_item.connect_clicked(move |_| {
        show_control_center_popover(&app_clone2);
    });

    // 5. Clock Dongle (macOS style)
    let clock_item = Button::builder()
        .css_classes(["macos-status-item", "macos-clock"])
        .build();
    clock_item.set_child(Some(clock_label));
    let app_clone3 = app.clone();
    clock_item.connect_clicked(move |_| {
        show_calendar_popover(&app_clone3);
    });

    box_right.append(&batt_item);
    box_right.append(&wifi_item);
    box_right.append(&spot_item);
    box_right.append(&cc_item);
    box_right.append(&clock_item);
    box_right
}

fn build_ui(app: &Application) {
    load_css();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Ermete Shell")
        .css_classes(["topbar-window"])
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Top);
    window.set_namespace("bar");
    window.auto_exclusive_zone_enable();

    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);

    // macOS Sonoma / Sequoia height = 28px exactly
    window.set_height_request(28);

    let container = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["topbar-container"])
        .hexpand(true)
        .build();

    let clock_label = Label::new(Some(&macos_clock_string()));

    let center_box = CenterBox::new();
    center_box.set_start_widget(Some(&build_left_island(app)));
    // Center is empty exactly like macOS Menu Bar
    center_box.set_end_widget(Some(&build_right_island(app, &clock_label)));
    center_box.set_hexpand(true);

    container.append(&center_box);
    window.set_child(Some(&container));

    glib::timeout_add_seconds_local(
        5,
        clone!(@weak clock_label => @default-return glib::ControlFlow::Break, move || {
            clock_label.set_label(&macos_clock_string());
            glib::ControlFlow::Continue
        }),
    );

    window.present();

    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "spotlight") {
        show_spotlight_modal(app);
    }
}

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}
