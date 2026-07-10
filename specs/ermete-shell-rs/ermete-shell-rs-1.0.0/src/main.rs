use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Label, Box, Orientation, Align};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use chrono::Local;

const APP_ID: &str = "os.ermete.Shell";

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Ermete Top Bar")
        .build();

    // Initialize Layer Shell
    window.init_layer_shell();
    window.set_layer(Layer::Top);
    window.set_namespace("topbar");
    
    // Anchor to top, left, right
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);
    
    // UI Layout
    let hbox = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .margin_top(4)
        .margin_bottom(4)
        .margin_start(12)
        .margin_end(12)
        .build();
        
    let time_label = Label::new(Some(&Local::now().format("%H:%M - %A %d %b").to_string()));
    time_label.set_halign(Align::Center);
    time_label.set_hexpand(true);
    
    hbox.append(&time_label);
    window.set_child(Some(&hbox));
    
    // Auto-update clock every minute
    glib::timeout_add_seconds_local(60, glib::clone!(@weak time_label => @default-return glib::ControlFlow::Break, move || {
        time_label.set_label(&Local::now().format("%H:%M - %A %d %b").to_string());
        glib::ControlFlow::Continue
    }));

    window.present();
}
