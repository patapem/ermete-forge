use gtk4::prelude::*;
use gtk4::{Box, Button, Label, ListBox, Orientation};
use std::process::Command;

pub fn build_page() -> Box {
    let container = Box::new(Orientation::Vertical, 16);
    container.set_margin_top(24);
    container.set_margin_bottom(24);
    container.set_margin_start(24);
    container.set_margin_end(24);

    // Title
    let title = Label::new(Some("Rete (Wi-Fi)"));
    title.add_css_class("title-1");
    title.set_halign(gtk4::Align::Start);
    container.append(&title);

    // Scan Button
    let scan_btn = Button::with_label("Scansiona Reti");
    scan_btn.set_halign(gtk4::Align::Start);
    container.append(&scan_btn);

    // ListBox for networks
    let list_box = ListBox::new();
    list_box.add_css_class("boxed-list");
    container.append(&list_box);

    // Connect scan button click handler
    scan_btn.connect_clicked(move |_| {
        // Clear list_box first
        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }

        // Run nmcli command
        let output = Command::new("nmcli")
            .args(["-t", "-f", "SSID", "dev", "wifi"])
            .output();

        let mut networks = vec![];

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let ssid = line.trim();
                    if !ssid.is_empty() {
                        networks.push(ssid.to_string());
                    }
                }
            }
            _ => {
                // Command failed or nmcli not found, add dummy names
                networks.push("Home_5G".to_string());
                networks.push("Guest".to_string());
            }
        }

        // Populate ListBox with networks
        for ssid in networks {
            let label = Label::new(Some(&ssid));
            label.set_halign(gtk4::Align::Start);
            label.set_margin_top(12);
            label.set_margin_bottom(12);
            label.set_margin_start(12);
            label.set_margin_end(12);
            list_box.append(&label);
        }
    });

    container
}
