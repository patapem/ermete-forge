use gtk4::prelude::*;
use gtk4::{Align, Box, Label, ListBox, ListBoxRow, Orientation, Switch};
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
        .label("Mouse & Trackpad")
        .css_classes(["title-1"])
        .halign(Align::Start)
        .build();
    container.append(&title);

    let list_box = ListBox::builder()
        .css_classes(["boxed-list"])
        .selection_mode(gtk4::SelectionMode::None)
        .build();

    // Natural Scroll
    let row1 = ListBoxRow::new();
    let hbox1 = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(16)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(16)
        .margin_end(16)
        .build();

    let label1 = Label::builder()
        .label("Scrolling Naturale (Invertito come macOS)")
        .halign(Align::Start)
        .hexpand(true)
        .build();

    let switch1 = Switch::builder()
        .valign(Align::Center)
        .build();

    switch1.connect_state_set(move |_, state| {
        let val = if state { "true" } else { "false" };
        let _ = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "sed -i 's/natural-scroll.*/natural-scroll {}/' ~/.config/niri/config.kdl",
                val
            ))
            .spawn();
        glib::Propagation::Proceed
    });

    hbox1.append(&label1);
    hbox1.append(&switch1);
    row1.set_child(Some(&hbox1));
    list_box.append(&row1);

    // Tap-to-click
    let row2 = ListBoxRow::new();
    let hbox2 = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(16)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(16)
        .margin_end(16)
        .build();

    let label2 = Label::builder()
        .label("Tap-to-click")
        .halign(Align::Start)
        .hexpand(true)
        .build();

    let switch2 = Switch::builder()
        .valign(Align::Center)
        .build();

    switch2.connect_state_set(move |_, state| {
        let val = if state { "true" } else { "false" };
        let _ = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "sed -i 's/tap-to-click.*/tap-to-click {}/' ~/.config/niri/config.kdl",
                val
            ))
            .spawn();
        glib::Propagation::Proceed
    });

    hbox2.append(&label2);
    hbox2.append(&switch2);
    row2.set_child(Some(&hbox2));
    list_box.append(&row2);

    container.append(&list_box);

    container
}
