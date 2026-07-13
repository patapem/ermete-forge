use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Label, ListBox, Orientation, Switch};

#[zbus::dbus_proxy(
    interface = "os.ermete.Bedrock.Bluetooth",
    default_service = "os.ermete.Bedrock",
    default_path = "/os/ermete/Bedrock/Bluetooth"
)]
trait Bluetooth {
    #[dbus_proxy(property)]
    fn power(&self) -> zbus::Result<bool>;
    #[dbus_proxy(property)]
    fn set_power(&self, value: bool) -> zbus::Result<()>;

    fn get_devices(&self) -> zbus::Result<Vec<String>>;
}

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

    // Set initial state
    let power_switch_clone = power_switch.clone();
    let ctx = gtk4::glib::MainContext::default();
    ctx.spawn_local(async move {
        match crate::get_connection().await {
            Ok(conn) => {
                match BluetoothProxy::new(&conn).await {
                    Ok(proxy) => {
                        match proxy.power().await {
                            Ok(power) => power_switch_clone.set_active(power),
                            Err(e) => eprintln!("Error getting Bluetooth power state: {:?}", e),
                        }
                    }
                    Err(e) => eprintln!("Error creating DBus proxy for Bluetooth: {:?}", e),
                }
            }
            Err(e) => eprintln!("Error connecting to DBus: {:?}", e),
        }
    });

    power_switch.connect_state_set(|_switch, state| {
        let ctx = gtk4::glib::MainContext::default();
        ctx.spawn_local(async move {
            match crate::get_connection().await {
                Ok(conn) => {
                    match BluetoothProxy::new(&conn).await {
                        Ok(proxy) => {
                            if let Err(e) = proxy.set_power(state).await {
                                eprintln!("Error setting Bluetooth power state: {:?}", e);
                            }
                        }
                        Err(e) => eprintln!("Error creating DBus proxy for Bluetooth: {:?}", e),
                    }
                }
                Err(e) => eprintln!("Error connecting to DBus: {:?}", e),
            }
        });
        gtk4::glib::Propagation::Proceed
    });

    switch_box.append(&switch_label);
    switch_box.append(&power_switch);
    container.append(&switch_box);

    // Search button
    let search_button = Button::builder()
        .label("Cerca Dispositivi")
        .halign(Align::Start)
        .build();

    // Mock list of devices
    let list_box = ListBox::builder()
        .selection_mode(gtk4::SelectionMode::None)
        .css_classes(["boxed-list"])
        .build();

    let list_box_clone = list_box.clone();

    search_button.connect_clicked(move |_| {
        let list_box = list_box_clone.clone();
        
        // Show loading state
        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }
        let loading_label = Label::new(Some("Caricamento..."));
        loading_label.set_margin_top(12);
        loading_label.set_margin_bottom(12);
        list_box.append(&loading_label);
        
        let ctx = gtk4::glib::MainContext::default();
        ctx.spawn_local(async move {
            match crate::get_connection().await {
                Ok(conn) => {
                    match BluetoothProxy::new(&conn).await {
                        Ok(proxy) => {
                            match proxy.get_devices().await {
                                Ok(devices) => {
                                    while let Some(child) = list_box.first_child() {
                                        list_box.remove(&child);
                                    }
                                    for device in devices {
                                        let row_box = Box::builder()
                                            .orientation(Orientation::Horizontal)
                                            .spacing(12)
                                            .margin_top(12)
                                            .margin_bottom(12)
                                            .margin_start(12)
                                            .margin_end(12)
                                            .build();
                                            
                                        let label = Label::new(Some(&device));
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
                                }
                                Err(e) => {
                                    eprintln!("Error getting Bluetooth devices: {:?}", e);
                                    while let Some(child) = list_box.first_child() {
                                        list_box.remove(&child);
                                    }
                                    let error_label = Label::new(Some("Errore durante la ricerca"));
                                    error_label.set_margin_top(12);
                                    error_label.set_margin_bottom(12);
                                    list_box.append(&error_label);
                                }
                            }
                        }
                        Err(e) => eprintln!("Error creating DBus proxy for Bluetooth: {:?}", e),
                    }
                }
                Err(e) => eprintln!("Error connecting to DBus: {:?}", e),
            }
        });
    });

    container.append(&search_button);
    container.append(&list_box);

    container
}
