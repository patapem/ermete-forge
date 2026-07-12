use gtk4::prelude::*;
use gtk4::{Box, ComboBoxText, Label, Orientation, Scale};
use std::process::Command;

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

    let res_box = Box::new(Orientation::Horizontal, 12);
    let res_label = Label::builder()
        .label("Risoluzione:")
        .halign(gtk4::Align::Start)
        .build();
    let res_combo = ComboBoxText::new();
    res_combo.append_text("1920x1080");
    res_combo.append_text("2560x1440");
    res_combo.append_text("3840x2160");
    res_combo.set_active(Some(0));

    res_combo.connect_changed(|combo| {
        if let Some(text) = combo.active_text() {
            println!("Risoluzione selezionata: {}", text);
        }
    });

    res_box.append(&res_label);
    res_box.append(&res_combo);
    container.append(&res_box);

    let scale_box = Box::new(Orientation::Vertical, 8);
    let scale_label = Label::builder()
        .label("Scala (Fractional Scaling):")
        .halign(gtk4::Align::Start)
        .build();

    let scale = Scale::with_range(Orientation::Horizontal, 1.0, 2.0, 0.1);
    scale.set_value(1.0);
    scale.set_draw_value(true);

    scale.connect_value_changed(|s| {
        let val = s.value();
        let val_str = format!("{:.1}", val);
        match Command::new("niri")
            .args(["msg", "output", "eDP-1", "scale", &val_str])
            .spawn()
        {
            Ok(mut child) => {
                let _ = child.wait();
                println!("Impostata scala niri a {}", val_str);
            }
            Err(e) => eprintln!("Errore nell'esecuzione di niri: {}", e),
        }
    });

    scale_box.append(&scale_label);
    scale_box.append(&scale);
    container.append(&scale_box);

    container
}
