use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Label, ListBox, Orientation, Switch};
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
        .label("Bluetooth")
        .halign(Align::Start)
        .css_classes(["title-1"])
        .build();

    container.append(&title);

    // Global Switch
    let switch_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build();

    let switch_label = Label::builder()
        .label("Attiva Bluetooth")
        .halign(Align::Start)
        .hexpand(true)
        .build();

    let power_switch = Switch::builder()
        .valign(Align::Center)
        .build();

    power_switch.connect_active_notify(|switch| {
        let state = switch.is_active();
        let arg = if state { "on" } else { "off" };
        let _ = Command::new("bluetoothctl")
            .args(["power", arg])
            .spawn();
    });

    switch_box.append(&switch_label);
    switch_box.append(&power_switch);
    container.append(&switch_box);

    // Search button
    let search_button = Button::builder()
        .label("Cerca Dispositivi")
        .halign(Align::Start)
        .build();

    search_button.connect_clicked(|_| {
        println!("Esecuzione dummy command per cercare dispositivi bluetooth...");
    });

    container.append(&search_button);

    // Mock list of devices
    let list_box = ListBox::builder()
        .selection_mode(gtk4::SelectionMode::None)
        .css_classes(["boxed-list"])
        .build();

    let devices = vec!["AirPods", "Mouse", "Tastiera"];

    for device in devices {
        let row_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();
            
        let label = Label::new(Some(device));
        label.set_halign(Align::Start);
        label.set_hexpand(true);
        
        let connect_btn = Button::builder()
            .label("Connetti")
            .valign(Align::Center)
            .build();
            
        row_box.append(&label);
        row_box.append(&connect_btn);
        
        list_box.append(&row_box);
    }

    container.append(&list_box);

    container
}
