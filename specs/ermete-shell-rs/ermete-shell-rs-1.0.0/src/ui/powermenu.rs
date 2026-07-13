use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Label};
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};

pub fn show_powermenu_modal(app: &Application) {
    let window = ApplicationWindow::new(app);
    window.init_layer_shell();
    window.set_layer(Layer::Top);
    window.set_keyboard_mode(KeyboardMode::OnDemand);
    
    let container = GtkBox::new(gtk4::Orientation::Vertical, 10);
    container.append(&Label::new(Some("Powermenu Placeholder")));
    window.set_child(Some(&container));
    window.present();
}
