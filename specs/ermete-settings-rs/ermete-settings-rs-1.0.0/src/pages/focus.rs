use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Label, Orientation, Switch};
use std::process::Command;

pub fn build_page() -> GtkBox {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(24)
        .margin_top(32)
        .margin_bottom(32)
        .margin_start(32)
        .margin_end(32)
        .build();

    // Title
    let title = Label::builder()
        .label("<span size='xx-large' weight='bold'>Focus (Full-Screen)</span>")
        .use_markup(true)
        .halign(Align::Start)
        .build();

    // Row for the toggle
    let row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(16)
        .build();

    let switch_label = Label::builder()
        .label("Nascondi Topbar quando un'app è in fullscreen")
        .halign(Align::Start)
        .hexpand(true)
        .build();

    let toggle = Switch::builder()
        .valign(Align::Center)
        .build();

    toggle.connect_active_notify(|switch| {
        let is_active = switch.is_active();
        if is_active {
            let _ = Command::new("sh")
                .arg("-c")
                .arg("niri msg window-rule add hide-bar-on-fullscreen || true")
                .spawn();
        } else {
            let _ = Command::new("sh")
                .arg("-c")
                .arg("niri msg window-rule remove hide-bar-on-fullscreen || true")
                .spawn();
        }
    });

    row.append(&switch_label);
    row.append(&toggle);

    container.append(&title);
    container.append(&row);

    container
}
