use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Label, Orientation};
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
        .label("Rete Cablata")
        .halign(Align::Start)
        .css_classes(["title-1"])
        .build();
    container.append(&title);

    let status_str = get_ethernet_status();
    let status_label = Label::builder()
        .label(&status_str)
        .halign(Align::Start)
        .css_classes(["heading"])
        .build();
    container.append(&status_label);

    let proxy_button = Button::builder()
        .label("Configura Proxy")
        .halign(Align::Start)
        .build();
    proxy_button.connect_clicked(|_| {
        println!("Azione dummy: Configura Proxy cliccato");
    });
    container.append(&proxy_button);

    container
}

fn get_ethernet_status() -> String {
    // Esegue nmcli per ottenere i dispositivi, filtra per ethernet.
    // L'output formattato sarà simile a "eth0:802-3-ethernet:connected"
    let output = Command::new("sh")
        .arg("-c")
        .arg("nmcli -t -f DEVICE,TYPE,STATE dev | grep -i ethernet | head -n 1")
        .output();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !stdout.is_empty() {
            let parts: Vec<&str> = stdout.split(':').collect();
            if parts.len() >= 3 {
                let dev = parts[0];
                let state = match parts[2] {
                    "connected" => "Connesso",
                    "disconnected" => "Scollegato",
                    "unavailable" => "Non disponibile",
                    "connecting" => "In connessione",
                    other => other,
                };
                return format!("{} - {}", dev, state);
            }
            return stdout;
        }
    }
    "Scollegato".to_string()
}
