use crate::core::*;
use glib::clone;
use gtk4::prelude::*;
use gtk4::{Align, Application, ApplicationWindow, Box as GtkBox, Label, Orientation};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

pub fn show_toast_popup(app: &Application, notif: &NotificationData) {
    let toast = ApplicationWindow::builder()
        .application(app)
        .css_classes(["popup-window"])
        .build();

    toast.init_layer_shell();
    toast.set_layer(Layer::Overlay);
    toast.set_anchor(Edge::Top, true);
    toast.set_anchor(Edge::Right, true);
    toast.set_margin(Edge::Top, 40);
    toast.set_margin(Edge::Right, 10);

    let vbox = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .css_classes(["cc-card"])
        .build();
    
    let title = Label::builder().label(&notif.summary).css_classes(["cc-title"]).halign(Align::Start).build();
    let body = Label::builder().label(&notif.body).css_classes(["cc-label-sub"]).halign(Align::Start).wrap(true).max_width_chars(30).build();
    
    vbox.append(&title);
    vbox.append(&body);
    toast.set_child(Some(&vbox));
    toast.present();

    glib::timeout_add_seconds_local(5, clone!(@weak toast => @default-return glib::ControlFlow::Break, move || {
        toast.close();
        glib::ControlFlow::Break
    }));
}

pub fn spawn_notification_daemon(app: &Application) {
    let (sender, receiver) = glib::MainContext::channel::<NotificationData>(glib::Priority::DEFAULT);
    
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let server = NotificationServer {
                sender,
                counter: std::sync::atomic::AtomicU32::new(1),
            };
            let _conn = zbus::connection::Builder::session()
                .unwrap()
                .name("org.freedesktop.Notifications")
                .unwrap()
                .serve_at("/org/freedesktop/Notifications", server)
                .unwrap()
                .build()
                .await
                .unwrap();
            std::future::pending::<()>().await;
        });
    });

    let app_clone = app.clone();
    receiver.attach(None, move |notif| {
        NOTIFICATIONS.with(|n| {
            let mut list = n.borrow_mut();
            if let Some(pos) = list.iter().position(|x| x.id == notif.id) {
                list[pos] = notif.clone();
            } else {
                list.insert(0, notif.clone());
            }
        });
        show_toast_popup(&app_clone, &notif);
        glib::ControlFlow::Continue
    });
}
