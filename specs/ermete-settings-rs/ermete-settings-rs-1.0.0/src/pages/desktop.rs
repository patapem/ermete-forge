use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Grid, Label, Orientation, Switch};
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

    // Title
    let title = Label::builder()
        .label("Desktop & Dock")
        .halign(Align::Start)
        .css_classes(["title-1", "large-title"])
        .build();

    container.append(&title);

    // Wallpaper Section
    let wallpaper_label = Label::builder()
        .label("Wallpaper")
        .halign(Align::Start)
        .css_classes(["heading"])
        .build();
    container.append(&wallpaper_label);

    let wallpaper_grid = Grid::builder()
        .column_spacing(12)
        .row_spacing(12)
        .build();

    for i in 0..3 {
        let btn = Button::builder()
            .label(format!("Wallpaper {}", i + 1))
            .width_request(160)
            .height_request(90)
            .build();
        
        btn.connect_clicked(move |_| {
            // Dummy command to set wallpaper
            let _ = Command::new("sh")
                .arg("-c")
                .arg(format!("swww img /path/to/wall_{}", i + 1))
                .spawn();
            println!("Wallpaper {} selected", i + 1);
        });

        wallpaper_grid.attach(&btn, i, 0, 1, 1);
    }
    
    container.append(&wallpaper_grid);

    // Dock Section
    let dock_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .margin_top(12)
        .build();

    let dock_label = Label::builder()
        .label("Mostra Dock in basso")
        .halign(Align::Start)
        .hexpand(true)
        .build();

    let dock_switch = Switch::builder()
        .valign(Align::Center)
        .build();

    dock_switch.connect_state_set(|_, state| {
        if state {
            // Dummy command to spawn dock
            let _ = Command::new("sh")
                .arg("-c")
                .arg("echo 'Spawn dock command here'")
                .spawn();
            println!("Dock enabled");
        } else {
            // Dummy command to kill dock
            let _ = Command::new("sh")
                .arg("-c")
                .arg("echo 'Kill dock command here'")
                .spawn();
            println!("Dock disabled");
        }
        gtk4::glib::Propagation::Proceed
    });

    dock_box.append(&dock_label);
    dock_box.append(&dock_switch);

    container.append(&dock_box);

    container
}
