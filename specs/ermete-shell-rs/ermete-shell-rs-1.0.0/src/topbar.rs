use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};

pub fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Ermete Topbar Dummy")
        .default_width(320)
        .default_height(24)
        .build();
    window.present();
}
