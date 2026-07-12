use notify::{Watcher, RecursiveMode};
use crate::core::*;
use crate::ui::spotlight::*;
use glib::clone;
use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, Calendar, CenterBox, CssProvider,
    Entry, Label, Orientation, PasswordEntry, ProgressBar, Scale, ScrolledWindow, Switch,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::process::Command;



fn show_toast_popup(app: &Application, notif: &NotificationData) {
    let toast = ApplicationWindow::builder()
        .application(app)
        .css_classes(["popup-window"])
        .build();

    toast.init_layer_shell();
    toast.set_layer(Layer::Overlay);
    toast.set_anchor(Edge::Top, true);
    toast.set_anchor(Edge::Right, true);
    toast.set_margin(Edge::Top, 40);
    toast.set_margin(Edge::Right, 10);

    let vbox = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .css_classes(["cc-card"])
        .build();
    
    let title = Label::builder().label(&notif.summary).css_classes(["cc-title"]).halign(Align::Start).build();
    let body = Label::builder().label(&notif.body).css_classes(["cc-label-sub"]).halign(Align::Start).wrap(true).max_width_chars(30).build();
    
    vbox.append(&title);
    vbox.append(&body);
    toast.set_child(Some(&vbox));
    toast.present();

    glib::timeout_add_seconds_local(5, clone!(@weak toast => @default-return glib::ControlFlow::Break, move || {
        toast.close();
        glib::ControlFlow::Break
    }));
}

fn spawn_notification_daemon(app: &Application) {
    let (sender, receiver) = glib::MainContext::channel::<NotificationData>(glib::Priority::DEFAULT);
    
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let server = NotificationServer {
                sender,
                counter: std::sync::atomic::AtomicU32::new(1),
            };
            let _conn = zbus::connection::Builder::session()
                .unwrap()
                .name("org.freedesktop.Notifications")
                .unwrap()
                .serve_at("/org/freedesktop/Notifications", server)
                .unwrap()
                .build()
                .await
                .unwrap();
            std::future::pending::<()>().await;
        });
    });

    let app_clone = app.clone();
    receiver.attach(None, move |notif| {
        NOTIFICATIONS.with(|n| {
            let mut list = n.borrow_mut();
            if let Some(pos) = list.iter().position(|x| x.id == notif.id) {
                list[pos] = notif.clone();
            } else {
                list.insert(0, notif.clone());
            }
        });
        show_toast_popup(&app_clone, &notif);
        glib::ControlFlow::Continue
    });
}


const TOPBAR_CSS: &str = r#"
window.topbar-window {
    background-color: transparent;
}

window.bg-overlay-window {
    background-color: rgba(0, 0, 0, 0.01);
}

.topbar-container {
    background: @shell_bg;
    border-bottom: 1px solid @shell_border;
    color: @shell_fg;
    font-family: -apple-system, 'SF Pro Text', 'Inter', sans-serif;
    font-size: 13px;
    font-weight: 500;
    padding: 0 10px;
}

.macos-menu-item {
    background: transparent;
    border: none;
    border-radius: 5px;
    padding: 2px 9px;
    color: @shell_fg;
    font-size: 13px;
    font-weight: 500;
    transition: background 100ms ease;
}

.macos-menu-item:hover {
    background: @shell_hover;
    color: @shell_primary;
}

.macos-apple-logo {
    font-size: 15px;
    font-weight: 700;
    padding: 2px 8px;
}

.macos-app-title {
    font-weight: 700;
    color: @shell_primary;
    padding: 2px 10px;
}

.macos-status-item {
    background: transparent;
    border: none;
    border-radius: 5px;
    padding: 2px 8px;
    color: @shell_fg;
    font-size: 13px;
    transition: background 100ms ease;
}

.macos-status-item:hover {
    background: @shell_hover;
    color: @shell_primary;
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

.metric-card {
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.10);
    border-radius: 14px;
    padding: 14px 16px;
}
.metric-value {
    font-size: 26px;
    font-weight: 800;
    color: #ffffff;
}
.pro-applet-card {
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 12px;
    padding: 10px 14px;
}
.applet-header-card {
    background: rgba(255, 255, 255, 0.09);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 14px;
    padding: 12px 16px;
}
.pro-applet-card-btn {
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 12px;
    padding: 10px 14px;
    color: #ffffff;
    transition: all 0.15s ease;
}
.pro-applet-card-btn:hover {
    background: rgba(255, 255, 255, 0.12);
    border-color: rgba(255, 255, 255, 0.20);
}
.wifi-pwd-entry {
    background: rgba(0, 0, 0, 0.45);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 10px;
    padding: 8px 12px;
    color: #ffffff;
    min-height: 38px;
}
"#;


fn load_css() {
    let home = std::env::var("HOME").unwrap();
    let colors_path = format!("{}/.config/ermete-shell/colors.css", home);
    let colors_css = std::fs::read_to_string(&colors_path).unwrap_or_default();
    
    // In caso il file di matugen non definisca nulla o sia vuoto, diamo dei fallback di sicurezza.
    let fallback = if colors_css.is_empty() {
        r#"
        @define-color shell_bg rgba(22, 22, 25, 0.88);
        @define-color shell_fg #f5f5f7;
        @define-color shell_border rgba(255, 255, 255, 0.08);
        @define-color shell_hover rgba(255, 255, 255, 0.14);
        @define-color shell_primary #ffffff;
        @define-color popup_bg rgba(30, 30, 30, 0.95);
        @define-color popup_border rgba(255, 255, 255, 0.1);
        @define-color btn_bg rgba(50, 50, 50, 0.8);
        @define-color btn_fg #ffffff;
        @define-color btn_hover rgba(70, 70, 70, 0.9);
        "#
    } else {
        ""
    };

    let full_css = format!("{}\n{}\n{}", colors_css, fallback, TOPBAR_CSS);

    CSS_PROVIDER.with(|p| {
        let mut provider_opt = p.borrow_mut();
        let display = gtk4::gdk::Display::default().unwrap();
        
        if let Some(old_provider) = provider_opt.as_ref() {
            gtk4::style_context_remove_provider_for_display(&display, old_provider);
        }
        
        let new_provider = CssProvider::new();
        new_provider.load_from_data(&full_css);
        gtk4::style_context_add_provider_for_display(
            &display,
            &new_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        *provider_opt = Some(new_provider);
    });
}

fn spawn_css_watcher() {
    let (sender, receiver) = glib::MainContext::channel::<()>(glib::Priority::DEFAULT);
    
    std::thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = notify::recommended_watcher(tx).unwrap();
        let path = std::path::PathBuf::from(std::env::var("HOME").unwrap()).join(".config/ermete-shell");
        let _ = watcher.watch(&path, RecursiveMode::NonRecursive);
        
        while let Ok(event) = rx.recv() {
            if let Ok(ev) = event {
                if ev.kind.is_modify() {
                    let _ = sender.send(());
                }
            }
        }
    });

    receiver.attach(None, move |_| {
        load_css();
        glib::ControlFlow::Continue
    });
}

thread_local! {
    static ACTIVE_POPUP: std::cell::RefCell<Option<(String, glib::WeakRef<ApplicationWindow>)>> = std::cell::RefCell::new(None);
}


pub fn setup_popup_autoclose(pop: &ApplicationWindow, tag: &str) {
    let mut to_close = None;
    ACTIVE_POPUP.with(|p| {
        if let Some((_, old_weak)) = p.borrow().as_ref() {
            if let Some(old_win) = old_weak.upgrade() {
                if old_win != *pop && old_win.is_visible() {
                    to_close = Some(old_win);
                }
            }
        }
        *p.borrow_mut() = Some((tag.to_string(), pop.downgrade()));
    });

    if let Some(win) = to_close {
        win.close();
    }

    pop.set_keyboard_mode(KeyboardMode::OnDemand);

    if let Some(app) = pop.application() {
        let bg_win = ApplicationWindow::builder()
            .application(&app)
            .css_classes(["bg-overlay-window"])
            .build();
            
        bg_win.init_layer_shell();
        bg_win.set_layer(Layer::Top);
        bg_win.set_anchor(Edge::Top, true);
        bg_win.set_anchor(Edge::Bottom, true);
        bg_win.set_anchor(Edge::Left, true);
        bg_win.set_anchor(Edge::Right, true);
        bg_win.set_exclusive_zone(-1);
        bg_win.set_keyboard_mode(KeyboardMode::None);
        
        let empty_box = gtk4::Box::builder()
            .hexpand(true)
            .vexpand(true)
            .build();
        bg_win.set_child(Some(&empty_box));
        
        let click = gtk4::GestureClick::new();
        click.set_button(0); // Tutti i bottoni
        let pop_close_clone = pop.clone();
        click.connect_pressed(move |_, _, _, _| {
            pop_close_clone.close();
        });
        empty_box.add_controller(click);
        
        let bg_clone = bg_win.clone();
        pop.connect_close_request(move |win| {
            bg_clone.close();
            ACTIVE_POPUP.with(|p| {
                let mut clear = false;
                if let Some((_, old_weak)) = p.borrow().as_ref() {
                    if let Some(old_win) = old_weak.upgrade() {
                        if old_win == *win {
                            clear = true;
                        }
                    }
                }
                if clear {
                    *p.borrow_mut() = None;
                }
            });
            glib::Propagation::Proceed
        });
        
        bg_win.present();
    } else {
        pop.connect_close_request(move |win| {
            ACTIVE_POPUP.with(|p| {
                let mut clear = false;
                if let Some((_, old_weak)) = p.borrow().as_ref() {
                    if let Some(old_win) = old_weak.upgrade() {
                        if old_win == *win {
                            clear = true;
                        }
                    }
                }
                if clear {
                    *p.borrow_mut() = None;
                }
            });
            glib::Propagation::Proceed
        });
    }

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
}


// macOS Spotlight Modal (Win+D / Clic su 🔍)

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

fn show_system_monitor_modal(app: &Application) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Monitor Risorse")
        .css_classes(["popup-window"])
        .default_width(360)
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    setup_popup_autoclose(&pop, "sys-monitor");
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
        .label("MONITOR DI SISTEMA — ERMETE OS")
        .css_classes(["cc-label-sub"])
        .halign(Align::Start)
        .build();

    // CPU Metric Card
    let (cpu_text, cpu_frac) = get_cpu_load();
    let cpu_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .css_classes(["metric-card"])
        .build();
    let cpu_top = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();
    let cpu_val_lbl = Label::builder()
        .label(&format!("{:.0}%", cpu_frac * 100.0))
        .css_classes(["metric-value"])
        .halign(Align::Start)
        .build();
    let cpu_desc = Label::builder()
        .label(&format!("Processore\n{}", cpu_text))
        .css_classes(["cc-label-sub"])
        .halign(Align::Start)
        .hexpand(true)
        .build();
    cpu_top.append(&cpu_val_lbl);
    cpu_top.append(&cpu_desc);
    let cpu_bar = ProgressBar::builder()
        .fraction(cpu_frac)
        .css_classes(["cc-progress-blue"])
        .build();
    cpu_card.append(&cpu_top);
    cpu_card.append(&cpu_bar);

    // RAM Metric Card
    let (ram_text, ram_frac) = get_ram_info();
    let ram_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .css_classes(["metric-card"])
        .build();
    let ram_top = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();
    let ram_val_lbl = Label::builder()
        .label(&format!("{:.0}%", ram_frac * 100.0))
        .css_classes(["metric-value"])
        .halign(Align::Start)
        .build();
    let ram_desc = Label::builder()
        .label(&format!("Memoria RAM\n{}", ram_text))
        .css_classes(["cc-label-sub"])
        .halign(Align::Start)
        .hexpand(true)
        .build();
    ram_top.append(&ram_val_lbl);
    ram_top.append(&ram_desc);
    let ram_bar = ProgressBar::builder()
        .fraction(ram_frac)
        .css_classes(["cc-progress-indigo"])
        .build();
    ram_card.append(&ram_top);
    ram_card.append(&ram_bar);

    let sys_info = Label::builder()
        .label("Wayland / Niri Compositor — Forgia Atomica RPM")
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
    card.append(&cpu_card);
    card.append(&ram_card);
    card.append(&sys_info);
    card.append(&close_btn);

    pop.set_child(Some(&card));
    pop.present();
}

fn show_wifi_password_modal(app: &Application, ssid: &str) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Autenticazione Wi-Fi")
        .css_classes(["popup-window"])
        .default_width(380)
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    setup_popup_autoclose(&pop, "wifi-password");
    pop.set_anchor(Edge::Top, true);
    pop.set_anchor(Edge::Right, true);
    pop.set_margin(Edge::Top, 60);
    pop.set_margin(Edge::Right, 80);

    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(14)
        .css_classes(["cc-card"])
        .build();

    // Header
    let header_card = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["applet-header-card"])
        .valign(Align::Center)
        .build();
    let header_icon = Label::builder().label("").css_classes(["cc-circle-blue"]).build();
    let texts_box = GtkBox::builder().orientation(Orientation::Vertical).spacing(2).hexpand(true).build();
    let title_lbl = Label::builder().label("Accedi alla rete Wi-Fi").css_classes(["cc-label-main"]).halign(Align::Start).build();
    let sub_lbl = Label::builder().label(format!("Rete: {}", ssid)).css_classes(["cc-label-sub"]).halign(Align::Start).build();
    texts_box.append(&title_lbl);
    texts_box.append(&sub_lbl);
    header_card.append(&header_icon);
    header_card.append(&texts_box);

    // Password field
    let pwd_entry = PasswordEntry::builder()
        .placeholder_text("Inserisci la password Wi-Fi...")
        .show_peek_icon(true)
        .css_classes(["wifi-pwd-entry"])
        .hexpand(true)
        .build();

    // Security note
    let sec_note = Label::builder()
        .label("🔒  NetworkManager memorizzerà questa password per la riconnessione automatica.")
        .css_classes(["cc-label-sub"])
        .wrap(true)
        .halign(Align::Start)
        .build();

    // Status label
    let status_lbl = Label::builder()
        .label("")
        .css_classes(["cc-label-sub"])
        .halign(Align::Start)
        .build();

    // Action buttons
    let btn_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .halign(Align::End)
        .build();

    let cancel_btn = Button::builder()
        .label("Annulla")
        .css_classes(["cc-quick-btn"])
        .build();
    let pop_cancel = pop.clone();
    cancel_btn.connect_clicked(move |_| {
        pop_cancel.close();
    });

    let connect_btn = Button::builder()
        .label("Connetti")
        .css_classes(["cc-quick-btn"])
        .build();

    let ssid_str = ssid.to_string();
    let pwd_clone = pwd_entry.clone();
    let pop_conn = pop.clone();
    let status_clone = status_lbl.clone();
    let do_connect = move || {
        let pwd = pwd_clone.text().to_string();
        if pwd.is_empty() {
            status_clone.set_label("⚠️ Inserisci prima la password.");
            return;
        }
        status_clone.set_label("⏳ Connessione in corso...");
        let _ = Command::new("nmcli")
            .args(["device", "wifi", "connect", &ssid_str, "password", &pwd])
            .spawn();
        pop_conn.close();
    };

    let do_conn_1 = do_connect.clone();
    connect_btn.connect_clicked(move |_| {
        do_conn_1();
    });

    let do_conn_2 = do_connect.clone();
    pwd_entry.connect_activate(move |_| {
        do_conn_2();
    });

    btn_box.append(&cancel_btn);
    btn_box.append(&connect_btn);

    card.append(&header_card);
    card.append(&pwd_entry);
    card.append(&sec_note);
    card.append(&status_lbl);
    card.append(&btn_box);

    pop.set_child(Some(&card));
    pop.present();
    pwd_entry.grab_focus();
}

fn show_wifi_details_modal(app: &Application, ssid: &str, active: bool) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title(format!("Configurazione Rete: {}", ssid))
        .css_classes(["popup-window"])
        .default_width(420)
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    setup_popup_autoclose(&pop, "wifi-details");
    pop.set_anchor(Edge::Top, true);
    pop.set_anchor(Edge::Right, true);
    pop.set_margin(Edge::Top, 50);
    pop.set_margin(Edge::Right, 60);

    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(14)
        .css_classes(["cc-card"])
        .build();

    let header_card = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["applet-header-card"])
        .valign(Align::Center)
        .build();
    let header_icon = Label::builder().label("").css_classes(["cc-circle-blue"]).build();
    let texts_box = GtkBox::builder().orientation(Orientation::Vertical).spacing(2).hexpand(true).build();
    let title_lbl = Label::builder().label(ssid).css_classes(["cc-label-main"]).halign(Align::Start).build();
    let sub_lbl = Label::builder()
        .label(if active { "Connesso — Rete Salvata" } else { "Profilo Memorizzato" })
        .css_classes(["cc-label-sub"])
        .halign(Align::Start)
        .build();
    texts_box.append(&title_lbl);
    texts_box.append(&sub_lbl);
    header_card.append(&header_icon);
    header_card.append(&texts_box);

    let mut cur_method = "auto".to_string();
    let mut cur_ip = "".to_string();
    let mut cur_gw = "".to_string();
    let mut cur_dns = "".to_string();
    let mut cur_auto = true;

    if let Ok(output) = Command::new("nmcli")
        .args(["-g", "ipv4.method,ipv4.addresses,ipv4.gateway,ipv4.dns,connection.autoconnect", "connection", "show", ssid])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        if lines.len() >= 5 {
            cur_method = lines[0].trim().to_string();
            cur_ip = lines[1].trim().to_string();
            cur_gw = lines[2].trim().to_string();
            cur_dns = lines[3].trim().to_string();
            cur_auto = lines[4].trim() != "no";
        }
    }

    let ip_section = GtkBox::builder().orientation(Orientation::Vertical).spacing(8).build();
    let ip_header = Label::builder().label("CONFIGURAZIONE IP (IPv4)").css_classes(["cc-label-sub"]).halign(Align::Start).build();
    let dhcp_row = GtkBox::builder().orientation(Orientation::Horizontal).spacing(10).build();
    let dhcp_lbl = Label::builder().label("IP Automatico (DHCP)").css_classes(["cc-label-main"]).hexpand(true).halign(Align::Start).build();
    let dhcp_sw = Switch::builder().active(cur_method == "auto").valign(Align::Center).build();
    dhcp_row.append(&dhcp_lbl);
    dhcp_row.append(&dhcp_sw);

    let ip_entry = Entry::builder()
        .placeholder_text("Indirizzo IP/Subnet (es. 192.168.1.50/24)")
        .text(&cur_ip)
        .sensitive(cur_method != "auto")
        .build();
    let gw_entry = Entry::builder()
        .placeholder_text("Gateway Router (es. 192.168.1.1)")
        .text(&cur_gw)
        .sensitive(cur_method != "auto")
        .build();

    let ip_e_clone = ip_entry.clone();
    let gw_e_clone = gw_entry.clone();
    dhcp_sw.connect_state_set(move |_, is_dhcp| {
        ip_e_clone.set_sensitive(!is_dhcp);
        gw_e_clone.set_sensitive(!is_dhcp);
        glib::Propagation::Proceed
    });

    ip_section.append(&ip_header);
    ip_section.append(&dhcp_row);
    ip_section.append(&ip_entry);
    ip_section.append(&gw_entry);

    let dns_section = GtkBox::builder().orientation(Orientation::Vertical).spacing(8).build();
    let dns_header = Label::builder().label("SERVER DNS").css_classes(["cc-label-sub"]).halign(Align::Start).build();
    let dns_entry = Entry::builder()
        .placeholder_text("DNS Personalizzati (es. 1.1.1.1, 8.8.8.8)")
        .text(&cur_dns)
        .build();
    dns_section.append(&dns_header);
    dns_section.append(&dns_entry);

    let auto_row = GtkBox::builder().orientation(Orientation::Horizontal).spacing(10).build();
    let auto_lbl = Label::builder().label("Riconnetti automaticamente").css_classes(["cc-label-main"]).hexpand(true).halign(Align::Start).build();
    let auto_sw = Switch::builder().active(cur_auto).valign(Align::Center).build();
    auto_row.append(&auto_lbl);
    auto_row.append(&auto_sw);

    let btn_box = GtkBox::builder().orientation(Orientation::Horizontal).spacing(8).build();

    let forget_btn = Button::builder().label("Dimentica").css_classes(["cc-quick-btn"]).build();
    let ssid_f = ssid.to_string();
    let pop_f = pop.clone();
    forget_btn.connect_clicked(move |_| {
        let _ = Command::new("nmcli").args(["connection", "delete", &ssid_f]).spawn();
        pop_f.close();
    });

    let disc_btn = Button::builder().label("Disconnetti").css_classes(["cc-quick-btn"]).build();
    let ssid_d = ssid.to_string();
    let pop_d = pop.clone();
    disc_btn.connect_clicked(move |_| {
        let _ = Command::new("nmcli").args(["connection", "down", &ssid_d]).spawn();
        pop_d.close();
    });

    let save_btn = Button::builder().label("Salva e Applica").css_classes(["cc-quick-btn"]).hexpand(true).build();
    let ssid_s = ssid.to_string();
    let dhcp_sw_clone = dhcp_sw.clone();
    let ip_e_s = ip_entry.clone();
    let gw_e_s = gw_entry.clone();
    let dns_e_s = dns_entry.clone();
    let auto_sw_s = auto_sw.clone();
    let pop_s = pop.clone();
    save_btn.connect_clicked(move |_| {
        if dhcp_sw_clone.is_active() {
            let _ = Command::new("nmcli").args(["connection", "modify", &ssid_s, "ipv4.method", "auto"]).output();
        } else {
            let ip_val = ip_e_s.text().to_string();
            let gw_val = gw_e_s.text().to_string();
            if !ip_val.is_empty() {
                let _ = Command::new("nmcli").args(["connection", "modify", &ssid_s, "ipv4.method", "manual", "ipv4.addresses", &ip_val]).output();
                if !gw_val.is_empty() {
                    let _ = Command::new("nmcli").args(["connection", "modify", &ssid_s, "ipv4.gateway", &gw_val]).output();
                }
            }
        }
        let dns_val = dns_e_s.text().to_string();
        if dns_val.trim().is_empty() {
            let _ = Command::new("nmcli").args(["connection", "modify", &ssid_s, "ipv4.ignore-auto-dns", "no", "ipv4.dns", ""]).output();
        } else {
            let _ = Command::new("nmcli").args(["connection", "modify", &ssid_s, "ipv4.ignore-auto-dns", "yes", "ipv4.dns", &dns_val]).output();
        }
        let auto_val = if auto_sw_s.is_active() { "yes" } else { "no" };
        let _ = Command::new("nmcli").args(["connection", "modify", &ssid_s, "connection.autoconnect", auto_val]).output();
        let _ = Command::new("nmcli").args(["connection", "up", &ssid_s]).output();
        pop_s.close();
    });

    btn_box.append(&forget_btn);
    if active {
        btn_box.append(&disc_btn);
    }
    btn_box.append(&save_btn);

    card.append(&header_card);
    card.append(&ip_section);
    card.append(&dns_section);
    card.append(&auto_row);
    card.append(&btn_box);

    pop.set_child(Some(&card));
    pop.present();
}

fn populate_wifi_list(list_box: &GtkBox, app: &Application, pop: &ApplicationWindow, wifi_enabled: bool) {
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

    if !wifi_enabled {
        let disabled_card = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(6)
            .css_classes(["pro-applet-card"])
            .build();
        let lbl1 = Label::builder().label("󰖪  Rete Wi-Fi disattivata").css_classes(["cc-label-main"]).halign(Align::Start).build();
        let lbl2 = Label::builder().label("Attiva l'interruttore in alto per cercare e visualizzare le reti Wi-Fi vicine.").css_classes(["cc-label-sub"]).wrap(true).halign(Align::Start).build();
        disabled_card.append(&lbl1);
        disabled_card.append(&lbl2);
        list_box.append(&disabled_card);
        return;
    }

    let mut known_ssids = std::collections::HashSet::new();
    if let Ok(saved_out) = Command::new("nmcli").args(["-t", "-f", "NAME", "connection", "show"]).output() {
        for line in String::from_utf8_lossy(&saved_out.stdout).lines() {
            if !line.is_empty() {
                known_ssids.insert(line.trim().to_string());
            }
        }
    }

    if let Ok(output) = Command::new("nmcli")
        .args(["-t", "-f", "IN-USE,SSID,SIGNAL", "device", "wifi", "list"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut count = 0;
        let mut seen = std::collections::HashSet::new();
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 && !parts[1].is_empty() && count < 8 {
                let ssid = parts[1];
                if seen.contains(ssid) {
                    continue;
                }
                seen.insert(ssid.to_string());

                let active = parts[0] == "*";
                let saved = known_ssids.contains(ssid);
                let sig = parts[2].parse::<i32>().unwrap_or(50);
                let icon = if sig > 75 {
                    "󰤨"
                } else if sig > 40 {
                    "󰤥"
                } else {
                    "󰤢"
                };

                let item_row = Button::builder()
                    .css_classes(["pro-applet-card-btn"])
                    .build();

                let inner_box = GtkBox::builder()
                    .orientation(Orientation::Horizontal)
                    .spacing(10)
                    .build();

                let icon_lbl = Label::builder().label(icon).build();
                let texts = GtkBox::builder().orientation(Orientation::Vertical).hexpand(true).build();
                let ssid_lbl = Label::builder()
                    .label(ssid)
                    .css_classes(["cc-label-main"])
                    .halign(Align::Start)
                    .build();
                let status_text = if active {
                    "Connesso — Attiva"
                } else if saved {
                    "Salvato — Clicca per impostazioni"
                } else {
                    "Disponibile — Clicca per connetterti"
                };
                let status_lbl = Label::builder()
                    .label(status_text)
                    .css_classes(["cc-label-sub"])
                    .halign(Align::Start)
                    .build();
                texts.append(&ssid_lbl);
                texts.append(&status_lbl);

                inner_box.append(&icon_lbl);
                inner_box.append(&texts);

                if active {
                    let check_lbl = Label::builder().label("✓").css_classes(["cc-label-main"]).build();
                    inner_box.append(&check_lbl);
                }

                item_row.set_child(Some(&inner_box));

                let app_clone = app.clone();
                let pop_clone = pop.clone();
                let ssid_str = ssid.to_string();
                item_row.connect_clicked(move |_| {
                    pop_clone.close();
                    if active || saved {
                        show_wifi_details_modal(&app_clone, &ssid_str, active);
                    } else {
                        show_wifi_password_modal(&app_clone, &ssid_str);
                    }
                });

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
}

fn show_wifi_popover(app: &Application) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Reti Wi-Fi")
        .css_classes(["popup-window"])
        .default_width(360)
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    setup_popup_autoclose(&pop, "wifi");
    pop.set_anchor(Edge::Top, true);
    pop.set_anchor(Edge::Right, true);
    pop.set_margin(Edge::Top, 34);
    pop.set_margin(Edge::Right, 50);

    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(14)
        .css_classes(["cc-card"])
        .build();

    let header_card = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["applet-header-card"])
        .valign(Align::Center)
        .build();
    let header_icon = Label::builder().label("").css_classes(["cc-circle-blue"]).build();
    let header_lbl = Label::builder().label("Rete Wi-Fi").css_classes(["cc-label-main"]).hexpand(true).halign(Align::Start).build();
    let wifi_enabled = if let Ok(output) = Command::new("nmcli").args(["radio", "wifi"]).output() {
        String::from_utf8_lossy(&output.stdout).trim() == "enabled"
    } else {
        true
    };
    let wifi_sw = Switch::builder().active(wifi_enabled).valign(Align::Center).build();
    header_card.append(&header_icon);
    header_card.append(&header_lbl);
    header_card.append(&wifi_sw);

    let list_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .build();

    populate_wifi_list(&list_box, app, &pop, wifi_enabled);

    let list_clone = list_box.clone();
    let app_clone = app.clone();
    let pop_clone = pop.clone();
    wifi_sw.connect_state_set(move |_, state| {
        let cmd = if state { "on" } else { "off" };
        let _ = Command::new("nmcli").args(["radio", "wifi", cmd]).spawn();
        populate_wifi_list(&list_clone, &app_clone, &pop_clone, state);
        glib::Propagation::Proceed
    });

    let close_btn = Button::builder()
        .label("Fine")
        .css_classes(["cc-quick-btn"])
        .build();
    let pop_clone2 = pop.clone();
    close_btn.connect_clicked(move |_| {
        pop_clone2.close();
    });

    card.append(&header_card);
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
        .default_width(360)
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    setup_popup_autoclose(&pop, "bluetooth");
    pop.set_anchor(Edge::Top, true);
    pop.set_anchor(Edge::Right, true);
    pop.set_margin(Edge::Top, 34);
    pop.set_margin(Edge::Right, 50);

    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(14)
        .css_classes(["cc-card"])
        .build();

    let header_card = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["applet-header-card"])
        .valign(Align::Center)
        .build();
    let header_icon = Label::builder().label("").css_classes(["cc-circle-blue"]).build();
    let header_lbl = Label::builder().label("Bluetooth").css_classes(["cc-label-main"]).hexpand(true).halign(Align::Start).build();
    let bt_enabled = if let Ok(output) = Command::new("bluetoothctl").arg("show").output() {
        String::from_utf8_lossy(&output.stdout).contains("Powered: yes")
    } else {
        true
    };
    let bt_sw = Switch::builder().active(bt_enabled).valign(Align::Center).build();
    bt_sw.connect_state_set(move |_, state| {
        let cmd = if state { "on" } else { "off" };
        let _ = Command::new("bluetoothctl").args(["power", cmd]).spawn();
        glib::Propagation::Proceed
    });
    header_card.append(&header_icon);
    header_card.append(&header_lbl);
    header_card.append(&bt_sw);

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
                    .css_classes(["pro-applet-card"])
                    .build();

                let icon_lbl = Label::builder().label("").build();
                let texts = GtkBox::builder().orientation(Orientation::Vertical).hexpand(true).build();
                let name_lbl = Label::builder()
                    .label(name)
                    .css_classes(["cc-label-main"])
                    .halign(Align::Start)
                    .build();
                let sub_lbl = Label::builder().label("Dispositivo Rilevato").css_classes(["cc-label-sub"]).halign(Align::Start).build();
                texts.append(&name_lbl);
                texts.append(&sub_lbl);

                item_row.append(&icon_lbl);
                item_row.append(&texts);
                list_box.append(&item_row);
                count += 1;
            }
        }
        if count == 0 {
            let no_bt = Label::builder()
                .label("Nessun dispositivo accoppiato")
                .css_classes(["cc-label-sub"])
                .build();
            list_box.append(&no_bt);
        }
    }

    let close_btn = Button::builder()
        .label("Fine")
        .css_classes(["cc-quick-btn"])
        .build();
    let pop_clone = pop.clone();
    close_btn.connect_clicked(move |_| {
        pop_clone.close();
    });

    card.append(&header_card);
    card.append(&list_box);
    card.append(&close_btn);

    pop.set_child(Some(&card));
    pop.present();
}

fn show_audio_mixer_popover(app: &Application) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Mixer Audio")
        .css_classes(["popup-window"])
        .default_width(360)
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    setup_popup_autoclose(&pop, "media-player");
    pop.set_anchor(Edge::Top, true);
    pop.set_anchor(Edge::Right, true);
    pop.set_margin(Edge::Top, 34);
    pop.set_margin(Edge::Right, 50);

    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(14)
        .css_classes(["cc-card"])
        .build();

    let header_card = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["applet-header-card"])
        .valign(Align::Center)
        .build();
    let header_icon = Label::builder().label("🎚️").css_classes(["cc-slider-icon"]).build();
    let header_texts = GtkBox::builder().orientation(Orientation::Vertical).hexpand(true).build();
    let title_lbl = Label::builder().label("MIXER AUDIO ERMETE OS").css_classes(["cc-label-main"]).halign(Align::Start).build();
    let sub_lbl = Label::builder().label("PipeWire / WirePlumber").css_classes(["cc-label-sub"]).halign(Align::Start).build();
    header_texts.append(&title_lbl);
    header_texts.append(&sub_lbl);
    header_card.append(&header_icon);
    header_card.append(&header_texts);

    // Sezione Uscita Audio
    let out_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .css_classes(["pro-applet-card"])
        .build();
    let out_header = GtkBox::builder().orientation(Orientation::Horizontal).spacing(8).build();
    let out_lbl = Label::builder().label("🔊  Uscita Audio (Speaker/Cuffie)").css_classes(["cc-label-main"]).hexpand(true).halign(Align::Start).build();
    let mute_out_btn = Button::builder().label("Muto").css_classes(["cc-quick-btn"]).build();
    mute_out_btn.connect_clicked(move |_| {
        let _ = Command::new("wpctl").args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"]).spawn();
    });
    out_header.append(&out_lbl);
    out_header.append(&mute_out_btn);

    let out_slider = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
    out_slider.set_value(80.0);
    out_slider.set_hexpand(true);
    out_slider.connect_value_changed(move |s| {
        let val = s.value() as i32;
        let _ = Command::new("wpctl")
            .arg("set-volume")
            .arg("@DEFAULT_AUDIO_SINK@")
            .arg(format!("{}%", val))
            .spawn();
    });
    out_card.append(&out_header);
    out_card.append(&out_slider);

    // Sezione Ingresso Microfono
    let in_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .css_classes(["pro-applet-card"])
        .build();
    let in_header = GtkBox::builder().orientation(Orientation::Horizontal).spacing(8).build();
    let in_lbl = Label::builder().label("🎙  Ingresso Audio (Microfono)").css_classes(["cc-label-main"]).hexpand(true).halign(Align::Start).build();
    let mute_in_btn = Button::builder().label("Muto").css_classes(["cc-quick-btn"]).build();
    mute_in_btn.connect_clicked(move |_| {
        let _ = Command::new("wpctl").args(["set-mute", "@DEFAULT_AUDIO_SOURCE@", "toggle"]).spawn();
    });
    in_header.append(&in_lbl);
    in_header.append(&mute_in_btn);

    let in_slider = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
    in_slider.set_value(75.0);
    in_slider.set_hexpand(true);
    in_slider.connect_value_changed(move |s| {
        let val = s.value() as i32;
        let _ = Command::new("wpctl")
            .arg("set-volume")
            .arg("@DEFAULT_AUDIO_SOURCE@")
            .arg(format!("{}%", val))
            .spawn();
    });
    in_card.append(&in_header);
    in_card.append(&in_slider);

    let close_btn = Button::builder()
        .label("Fine")
        .css_classes(["cc-quick-btn"])
        .build();
    let pop_clone = pop.clone();
    close_btn.connect_clicked(move |_| {
        pop_clone.close();
    });

    card.append(&header_card);
    card.append(&out_card);
    card.append(&in_card);
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
    setup_popup_autoclose(&pop, "control-center");
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

    let (net_icon, net_title, net_sub) = get_network_status();
    let wifi_btn = build_cc_row("cc-circle-blue", &net_icon, &net_title, &net_sub);
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
    let app_mixer = app.clone();
    let pop_mixer = pop.clone();
    mixer_btn.connect_clicked(move |_| {
        pop_mixer.close();
        show_audio_mixer_popover(&app_mixer);
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
        .default_width(560)
        .default_height(480)
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    setup_popup_autoclose(&pop, "launcher");
    pop.set_anchor(Edge::Top, true);
    pop.set_anchor(Edge::Left, true);
    pop.set_margin(Edge::Top, 32);
    pop.set_margin(Edge::Left, 8);

    let main_hbox = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(0)
        .css_classes(["cc-card"])
        .build();

    let sidebar = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .build();
    sidebar.set_margin_top(14);
    sidebar.set_margin_bottom(14);
    sidebar.set_margin_start(14);
    sidebar.set_margin_end(14);

    let cats_lbl = Label::builder().label("CATEGORIE").css_classes(["cc-label-sub"]).halign(Align::Start).margin_bottom(6).build();
    sidebar.append(&cats_lbl);

    let list_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .build();

    let search = Entry::builder()
        .placeholder_text("Cerca nel menu...")
        .css_classes(["spotlight-input"])
        .build();

    let current_category = std::rc::Rc::new(std::cell::RefCell::new("Tutte".to_string()));
    let cats = ["Tutte", "Internet", "Ufficio", "Grafica", "Multimedia", "Sviluppo", "Sistema", "Giochi"];
    
    for cat in cats {
        let btn = Button::builder().label(cat).css_classes(["spotlight-item"]).halign(Align::Fill).build();
        let cat_str = cat.to_string();
        let list_clone = list_box.clone();
        let entry_clone = search.clone();
        let pop_clone = pop.clone();
        let curr_cat = current_category.clone();
        btn.connect_clicked(move |_| {
            *curr_cat.borrow_mut() = cat_str.clone();
            populate_launcher_list(&list_clone, &entry_clone.text(), &cat_str, false, &pop_clone);
        });
        sidebar.append(&btn);
    }

    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .hexpand(true)
        .build();
    card.set_margin_top(14);
    card.set_margin_bottom(14);
    card.set_margin_end(14);

    let title = Label::builder()
        .label("◈  MENU APPLICAZIONI ERMETE OS")
        .css_classes(["cc-title"])
        .build();

    let scroll = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .min_content_height(310)
        .build();

    populate_launcher_list(&list_box, "", "Tutte", false, &pop);

    let list_clone2 = list_box.clone();
    let pop_clone2 = pop.clone();
    let curr_cat2 = current_category.clone();
    search.connect_changed(move |e| {
        populate_launcher_list(&list_clone2, &e.text(), &curr_cat2.borrow(), false, &pop_clone2);
    });

    scroll.set_child(Some(&list_box));

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

    main_hbox.append(&sidebar);
    
    let sep = gtk4::Separator::new(Orientation::Vertical);
    sep.set_margin_start(4);
    sep.set_margin_end(10);
    main_hbox.append(&sep);
    
    main_hbox.append(&card);

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

    pop.set_child(Some(&main_hbox));
    pop.present();
    search.grab_focus();
}

fn show_calendar_popover(app: &Application) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Calendar")
        .css_classes(["popup-window"])
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    setup_popup_autoclose(&pop, "calendar");
    pop.set_anchor(Edge::Top, true);
    pop.set_anchor(Edge::Right, true);
    pop.set_margin(Edge::Top, 32);
    pop.set_margin(Edge::Right, 10);

    let main_vbox = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .build();

    let notifs_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .css_classes(["cc-card"])
        .build();
    
    let title_hbox = GtkBox::builder().orientation(Orientation::Horizontal).build();
    let notifs_title = Label::builder().label("Notifiche").css_classes(["cc-title"]).halign(Align::Start).hexpand(true).build();
    let clear_btn = Button::builder().label("Cancella").css_classes(["cc-btn"]).build();
    
    let scroll = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .max_content_height(300)
        .propagate_natural_height(true)
        .build();
    
    let list_box = GtkBox::builder().orientation(Orientation::Vertical).spacing(8).build();
    
    let pop_clone_clear = pop.clone();
    clear_btn.connect_clicked(move |_| {
        NOTIFICATIONS.with(|n| n.borrow_mut().clear());
        pop_clone_clear.close();
    });
    
    title_hbox.append(&notifs_title);
    title_hbox.append(&clear_btn);
    notifs_card.append(&title_hbox);
    
    NOTIFICATIONS.with(|n| {
        let history = n.borrow();
        if history.is_empty() {
            list_box.append(&Label::builder().label("Nessuna nuova notifica").css_classes(["cc-label-sub"]).margin_top(10).margin_bottom(10).build());
        } else {
            for notif in history.iter() {
                let row = GtkBox::builder().orientation(Orientation::Vertical).spacing(2).build();
                let sum = Label::builder().label(&notif.summary).halign(Align::Start).css_classes(["cc-label-main"]).build();
                let bod = Label::builder().label(&notif.body).halign(Align::Start).css_classes(["cc-label-sub"]).wrap(true).max_width_chars(30).build();
                row.append(&sum);
                row.append(&bod);
                list_box.append(&row);
            }
        }
    });

    scroll.set_child(Some(&list_box));
    notifs_card.append(&scroll);

    let cal_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .css_classes(["cc-card"])
        .build();

    let cal = Calendar::builder().build();
    cal_card.append(&cal);

    main_vbox.append(&notifs_card);
    main_vbox.append(&cal_card);

    pop.set_child(Some(&main_vbox));
    pop.present();
}

// Left Section: Authentic macOS Menu Bar items (Flat, no pills)
fn build_left_island(app: &Application) -> GtkBox {
    let box_left = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(2)
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
    box_left.append(&apple_logo);

    let app_title = Button::builder()
        .label("Ermete OS")
        .css_classes(["macos-menu-item", "macos-app-title"])
        .build();
    box_left.append(&app_title);

    box_left
}

fn build_center_island(_app: &Application) -> GtkBox {
    let workspace_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .valign(Align::Center)
        .build();

    let scroll_ctrl = gtk4::EventControllerScroll::new(gtk4::EventControllerScrollFlags::VERTICAL);
    scroll_ctrl.connect_scroll(|_, _dx, dy| {
        if dy > 0.0 {
            let _ = Command::new("niri").args(["msg", "action", "focus-workspace-down"]).spawn();
        } else if dy < 0.0 {
            let _ = Command::new("niri").args(["msg", "action", "focus-workspace-up"]).spawn();
        }
        glib::Propagation::Stop
    });
    workspace_box.add_controller(scroll_ctrl);

    let (sender, receiver) = glib::MainContext::channel(glib::Priority::DEFAULT);
    spawn_niri_workspace_watcher(sender);

    let workspace_box_clone = workspace_box.clone();
    receiver.attach(None, move |workspaces| {
        while let Some(child) = workspace_box_clone.first_child() {
            workspace_box_clone.remove(&child);
        }

        let active_output = workspaces.iter()
            .find(|w| w.is_focused)
            .or_else(|| workspaces.iter().find(|w| w.is_active))
            .map(|w| w.output.clone())
            .unwrap_or_default();

        let mut filtered_ws: Vec<_> = workspaces.into_iter().filter(|w| w.output == active_output).collect();
        filtered_ws.sort_by_key(|w| w.idx);

        for ws in filtered_ws {
            let label = if ws.is_active { "●" } else { "○" };
            let ws_btn = Button::builder()
                .label(label)
                .css_classes(["macos-menu-item"])
                .build();
            
            if ws.is_focused {
                ws_btn.add_css_class("workspace-focused");
            } else if ws.is_active {
                ws_btn.add_css_class("workspace-active");
            }

            let ws_id = ws.id;
            ws_btn.connect_clicked(move |_| {
                let _ = Command::new("niri")
                    .args(["msg", "action", "focus-workspace", &ws_id.to_string()])
                    .spawn();
            });

            workspace_box_clone.append(&ws_btn);
        }
        glib::ControlFlow::Continue
    });

    workspace_box
}

fn build_right_island(app: &Application, clock_label: &Label) -> (GtkBox, Button) {
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

    // 2. Dynamic Network Dongle (macOS style: Ethernet/Wi-Fi/Off)
    let (init_icon, _, _) = get_network_status();
    let net_item = Button::builder()
        .label(&init_icon)
        .css_classes(["macos-status-item"])
        .build();
    let app_net = app.clone();
    net_item.connect_clicked(move |_| {
        show_wifi_popover(&app_net);
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
    box_right.append(&net_item);
    box_right.append(&spot_item);
    box_right.append(&cc_item);
    box_right.append(&clock_item);
    (box_right, net_item)
}

pub fn build_ui(app: &Application) {
    load_css();
    spawn_css_watcher();
    spawn_notification_daemon(app);

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
    center_box.set_center_widget(Some(&build_center_island(app)));
    let (right_island, net_btn) = build_right_island(app, &clock_label);
    center_box.set_end_widget(Some(&right_island));
    center_box.set_hexpand(true);

    container.append(&center_box);
    window.set_child(Some(&container));

    glib::timeout_add_seconds_local(
        5,
        clone!(@weak clock_label, @weak net_btn => @default-return glib::ControlFlow::Break, move || {
            clock_label.set_label(&macos_clock_string());
            let (net_icon, _, _) = get_network_status();
            net_btn.set_label(&net_icon);
            glib::ControlFlow::Continue
        }),
    );

    window.present();
}









#[allow(dead_code)]
const APP_ID: &str = "os.ermete.Shell";

#[allow(dead_code)]
pub fn toggle_or_open_popup(tag: &str, open_fn: impl FnOnce()) {
    let mut to_close = None;
    let mut already_open = false;
    ACTIVE_POPUP.with(|p| {
        if let Some((old_tag, old_weak)) = p.borrow().as_ref() {
            if let Some(old_win) = old_weak.upgrade() {
                if old_win.is_visible() {
                    to_close = Some(old_win);
                    if old_tag == tag {
                        already_open = true;
                    }
                }
            }
        }
        *p.borrow_mut() = None;
    });

    if let Some(win) = to_close {
        win.close();
    }

    if !already_open {
        open_fn();
    }
}

#[allow(dead_code)]
static UI_BUILT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[allow(dead_code)]
pub fn handle_command(app: &Application, arg: &str) {
    match arg {
        "spotlight" | "launcher" => toggle_or_open_popup("spotlight", || crate::ui::spotlight::show_spotlight_modal(app)),
        "control-center" => toggle_or_open_popup("control-center", || show_control_center_popover(app)),
        "sys-monitor" | "monitor" => toggle_or_open_popup("sys-monitor", || show_system_monitor_modal(app)),
        "calendar" => toggle_or_open_popup("calendar", || show_calendar_popover(app)),
        "media-player" | "mixer" | "audio" => toggle_or_open_popup("media-player", || show_audio_mixer_popover(app)),
        "wifi" => toggle_or_open_popup("wifi", || show_wifi_popover(app)),
        "bluetooth" => toggle_or_open_popup("bluetooth", || show_bluetooth_popover(app)),
        "start-menu" | "menu" => toggle_or_open_popup("launcher", || show_start_menu_popover(app)),
        _ => {}
    }
}
