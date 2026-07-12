use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Label, Orientation, ToggleButton};
use std::process::Command;

pub fn build_page() -> Box {
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(24)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    let title = Label::builder()
        .label("<b>Aspetto e Temi</b>")
        .use_markup(true)
        .halign(Align::Start)
        .build();
    title.add_css_class("title-1");

    container.append(&title);

    // Color Scheme Section
    let scheme_label = Label::builder()
        .label("Tema Colore")
        .halign(Align::Start)
        .build();
    scheme_label.add_css_class("heading");
    container.append(&scheme_label);

    let scheme_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .halign(Align::Center)
        .build();

    let btn_light = ToggleButton::with_label("Chiaro");
    let btn_dark = ToggleButton::with_label("Scuro");
    let btn_auto = ToggleButton::with_label("Auto");

    btn_light.set_size_request(120, 80);
    btn_dark.set_size_request(120, 80);
    btn_auto.set_size_request(120, 80);

    btn_dark.set_group(Some(&btn_light));
    btn_auto.set_group(Some(&btn_light));

    btn_light.connect_toggled(|btn| {
        if btn.is_active() {
            let _ = Command::new("dconf")
                .args(["write", "/org/gnome/desktop/interface/color-scheme", "'prefer-light'"])
                .spawn();
        }
    });

    btn_dark.connect_toggled(|btn| {
        if btn.is_active() {
            let _ = Command::new("dconf")
                .args(["write", "/org/gnome/desktop/interface/color-scheme", "'prefer-dark'"])
                .spawn();
        }
    });

    btn_auto.connect_toggled(|btn| {
        if btn.is_active() {
            let _ = Command::new("dconf")
                .args(["write", "/org/gnome/desktop/interface/color-scheme", "'default'"])
                .spawn();
        }
    });

    scheme_box.append(&btn_light);
    scheme_box.append(&btn_dark);
    scheme_box.append(&btn_auto);
    container.append(&scheme_box);

    // Accent Color Section
    let accent_label = Label::builder()
        .label("Colore Accento")
        .halign(Align::Start)
        .build();
    accent_label.add_css_class("heading");
    container.append(&accent_label);

    let accent_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .halign(Align::Center)
        .build();

    let accents = vec![
        ("Blu", "blue"),
        ("Rosso", "red"),
        ("Verde", "green"),
        ("Arancione", "orange"),
        ("Viola", "purple"),
        ("Rosa", "pink"),
    ];

    for (name, val) in accents {
        let btn = Button::with_label(name);
        btn.set_size_request(80, 40);
        let val_clone = val.to_string();
        btn.connect_clicked(move |_| {
            let _ = Command::new("gsettings")
                .args([
                    "set",
                    "org.gnome.desktop.interface",
                    "accent-color",
                    &val_clone,
                ])
                .spawn();
        });
        accent_box.append(&btn);
    }

    container.append(&accent_box);

    container
}
