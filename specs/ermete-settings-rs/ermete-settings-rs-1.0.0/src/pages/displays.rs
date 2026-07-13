use gtk4::prelude::*;
use gtk4::{Box, ComboBoxText, Label, Orientation, Scale};
use std::process::Command;

fn get_niri_outputs() -> Vec<String> {
    let mut outputs = Vec::new();
    if let Ok(output) = Command::new("niri").args(["msg", "-j", "outputs"]).output() {
        if output.status.success() {
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                if let Some(map) = json.as_object() {
                    for key in map.keys() {
                        outputs.push(key.clone());
                    }
                } else if let Some(arr) = json.as_array() {
                    for item in arr {
                        if let Some(name) = item.get("name").and_then(|n| n.as_str()) {
                            outputs.push(name.to_string());
                        }
                    }
                }
            }
        }
    }
    outputs.sort();
    outputs.dedup();
    if outputs.is_empty() {
        outputs.push("eDP-1".to_string());
    }
    outputs
}

pub fn build_page() -> Box {
    let container = Box::new(Orientation::Vertical, 16);
    container.set_margin_top(24);
    container.set_margin_bottom(24);
    container.set_margin_start(24);
    container.set_margin_end(24);

    let title = Label::builder()
        .label("<span size='large' weight='bold'>Schermi (Niri)</span>")
        .use_markup(true)
        .halign(gtk4::Align::Start)
        .build();

    container.append(&title);

    let monitor_box = Box::new(Orientation::Horizontal, 12);
    let monitor_label = Label::builder()
        .label("Schermo:")
        .halign(gtk4::Align::Start)
        .build();
    let monitor_combo = ComboBoxText::new();

    let outputs = get_niri_outputs();
    for output in &outputs {
        monitor_combo.append_text(output);
    }
    monitor_combo.set_active(Some(0));

    monitor_combo.connect_changed(|combo| {
        if let Some(text) = combo.active_text() {
            println!("Schermo selezionato: {}", text);
        }
    });

    monitor_box.append(&monitor_label);
    monitor_box.append(&monitor_combo);
    container.append(&monitor_box);

    let scale_box = Box::new(Orientation::Vertical, 8);
    let scale_label = Label::builder()
        .label("Scala (Fractional Scaling):")
        .halign(gtk4::Align::Start)
        .build();

    let scale = Scale::with_range(Orientation::Horizontal, 1.0, 2.0, 0.1);
    scale.set_value(1.0);
    scale.set_draw_value(true);

    let combo_clone = monitor_combo.clone();
    scale.connect_value_changed(move |s| {
        let selected_output = combo_clone
            .active_text()
            .map(|t| t.to_string())
            .unwrap_or_else(|| "eDP-1".to_string());

        let val = s.value();
        let val_str = format!("{:.1}", val);
        match Command::new("niri")
            .args(["msg", "output", &selected_output, "scale", &val_str])
            .spawn()
        {
            Ok(mut child) => {
                let _ = child.wait();
                println!(
                    "Impostata scala niri per lo schermo {} a {}",
                    selected_output, val_str
                );
            }
            Err(e) => eprintln!("Errore nell'esecuzione di niri: {}", e),
        }
    });

    scale_box.append(&scale_label);
    scale_box.append(&scale);
    container.append(&scale_box);

    container
}

