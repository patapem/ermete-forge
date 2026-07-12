use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};

pub fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Ermete Greeter Dummy")
        .default_width(800)
        .default_height(600)
        .build();
    window.present();
}
