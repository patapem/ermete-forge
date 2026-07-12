use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Label, Orientation, Switch};

pub fn build_page() -> GtkBox {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(24)
        .margin_top(32)
        .margin_bottom(32)
        .margin_start(32)
        .margin_end(32)
        .build();

    // Titolo
    let title = Label::builder()
        .label("<span size='xx-large' weight='bold'>Privacy &amp; Sicurezza</span>")
        .use_markup(true)
        .halign(Align::Start)
        .build();
    container.append(&title);

    // Contenitore per le impostazioni
    let settings_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .build();

    // Toggle: Posizione
    let location_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build();
    let location_label = Label::builder()
        .label("Permetti alla shell di accedere alla posizione")
        .halign(Align::Start)
        .hexpand(true)
        .build();
    let location_switch = Switch::builder()
        .valign(Align::Center)
        .build();
    location_box.append(&location_label);
    location_box.append(&location_switch);
    settings_box.append(&location_box);

    // Toggle: Diagnostica
    let diag_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build();
    let diag_label = Label::builder()
        .label("Invia Dati Diagnostici")
        .halign(Align::Start)
        .hexpand(true)
        .build();
    let diag_switch = Switch::builder()
        .valign(Align::Center)
        .build();
    diag_box.append(&diag_label);
    diag_box.append(&diag_switch);
    settings_box.append(&diag_box);

    container.append(&settings_box);

    // Pulsante: Pulisci Cache
    let cache_btn = Button::builder()
        .label("Pulisci Cache Sistema")
        .halign(Align::Start)
        .margin_top(16)
        .css_classes(vec!["destructive-action"])
        .build();
    
    cache_btn.connect_clicked(|_| {
        println!("Dummy action: Pulisci Cache Sistema");
    });
    container.append(&cache_btn);

    container
}
