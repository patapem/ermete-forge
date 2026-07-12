use gtk4::prelude::*;
use gtk4::{Align, Box, Label, Orientation, Scale, Switch, Adjustment};
use std::process::Command;

pub fn build_page() -> Box {
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(24)
        .margin_top(32)
        .margin_bottom(32)
        .margin_start(32)
        .margin_end(32)
        .build();

    let title = Label::builder()
        .label("Audio e Suoni")
        .css_classes(["title-1"])
        .halign(Align::Start)
        .build();
    container.append(&title);

    // Volume section
    let volume_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(16)
        .build();

    let volume_label = Label::builder()
        .label("Volume Principale")
        .halign(Align::Start)
        .build();

    let adjustment = Adjustment::new(0.5, 0.0, 1.0, 0.05, 0.1, 0.0);
    let volume_scale = Scale::builder()
        .orientation(Orientation::Horizontal)
        .adjustment(&adjustment)
        .digits(2)
        .draw_value(true)
        .hexpand(true)
        .valign(Align::Center)
        .build();

    volume_scale.connect_value_changed(|scale| {
        let val = scale.value();
        let _ = Command::new("wpctl")
            .arg("set-volume")
            .arg("@DEFAULT_AUDIO_SINK@")
            .arg(format!("{:.2}", val))
            .spawn();
    });

    volume_box.append(&volume_label);
    volume_box.append(&volume_scale);
    container.append(&volume_box);

    // Mute section
    let mute_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(16)
        .build();

    let mute_label = Label::builder()
        .label("Muto")
        .halign(Align::Start)
        .hexpand(true)
        .build();

    let mute_switch = Switch::builder()
        .valign(Align::Center)
        .build();

    mute_switch.connect_active_notify(|_| {
        let _ = Command::new("wpctl")
            .arg("set-mute")
            .arg("@DEFAULT_AUDIO_SINK@")
            .arg("toggle")
            .spawn();
    });

    mute_box.append(&mute_label);
    mute_box.append(&mute_switch);
    container.append(&mute_box);

    container
}
