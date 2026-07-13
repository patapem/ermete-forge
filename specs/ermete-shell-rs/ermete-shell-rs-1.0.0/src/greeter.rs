use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Entry, Label, Orientation, Align};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::os::unix::net::UnixStream;
use std::io::{Read, Write};
use greetd_ipc::{Request, Response};

const GREETER_CSS: &str = r#"
window.background {
    background-color: rgba(10, 12, 16, 0.45);
}

.greeter-card {
    background-color: rgba(22, 25, 33, 0.90);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 24px;
    padding: 42px 48px;
    min-width: 340px;
    box-shadow: 0 24px 60px rgba(0, 0, 0, 0.65);
}

.greeter-os-title {
    font-size: 11px;
    font-weight: 800;
    letter-spacing: 3px;
    color: rgba(255, 255, 255, 0.45);
}

.greeter-user-name {
    font-size: 26px;
    font-weight: 700;
    color: #ffffff;
    margin-bottom: 8px;
}

.greeter-entry {
    background-color: rgba(255, 255, 255, 0.07);
    border: 1px solid rgba(255, 255, 255, 0.18);
    border-radius: 12px;
    color: #ffffff;
    caret-color: #6ea8fe;
    font-size: 15px;
    padding: 12px 16px;
    min-height: 44px;
}

.greeter-entry:focus {
    border-color: #6ea8fe;
    background-color: rgba(255, 255, 255, 0.11);
}

.greeter-error {
    color: #ff6b6b;
    font-size: 13px;
    font-weight: 600;
    margin-top: 4px;
}
"#;

fn send_request(stream: &mut UnixStream, req: &Request) -> Result<Response, String> {
    let json = serde_json::to_string(req).map_err(|e| e.to_string())?;
    let len = (json.len() as u32).to_ne_bytes();
    stream.write_all(&len).map_err(|e| e.to_string())?;
    stream.write_all(json.as_bytes()).map_err(|e| e.to_string())?;

    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).map_err(|e| e.to_string())?;
    let reply_len = u32::from_ne_bytes(len_buf);

    let mut reply_buf = vec![0u8; reply_len as usize];
    stream.read_exact(&mut reply_buf).map_err(|e| e.to_string())?;

    serde_json::from_slice(&reply_buf).map_err(|e| e.to_string())
}

fn authenticate(password: &str) -> Result<(), String> {
    let path = std::env::var("GREETD_SOCK").unwrap_or_else(|_| "/run/greetd.sock".to_string());
    let mut stream = UnixStream::connect(path).map_err(|e| e.to_string())?;

    let env_user = std::env::var("USER").unwrap_or_default();
    let username = if env_user == "greeter" || env_user.is_empty() {
        std::env::var("ERMETE_LOGIN_USER").unwrap_or_else(|_| "ermete".to_string())
    } else {
        env_user
    };

    let session_cmd = if std::path::Path::new("/etc/greetd/ermete-session").exists() {
        "/etc/greetd/ermete-session".to_string()
    } else if std::path::Path::new("/usr/local/bin/ermete-session").exists() {
        "/usr/local/bin/ermete-session".to_string()
    } else {
        "ermete-session".to_string()
    };

    let req = Request::CreateSession { username };
    let resp = send_request(&mut stream, &req)?;

    match resp {
        Response::AuthMessage { .. } => {
            let req = Request::PostAuthMessageResponse { response: Some(password.to_string()) };
            let resp = send_request(&mut stream, &req)?;
            match resp {
                Response::Success => {
                    let req = Request::StartSession {
                        cmd: vec![session_cmd],
                        env: vec![],
                    };
                    let resp = send_request(&mut stream, &req)?;
                    match resp {
                        Response::Success => Ok(()),
                        Response::Error { description, .. } => Err(description),
                        _ => Err("Unexpected response to StartSession".to_string()),
                    }
                },
                Response::Error { description, .. } => Err(description),
                _ => Err("Unexpected response to PostAuthMessageResponse".to_string()),
            }
        },
        Response::Success => {
             let req = Request::StartSession {
                 cmd: vec![session_cmd],
                 env: vec![],
             };
             let resp = send_request(&mut stream, &req)?;
             match resp {
                 Response::Success => Ok(()),
                 Response::Error { description, .. } => Err(description),
                 _ => Err("Unexpected response to StartSession".to_string()),
             }
        },
        Response::Error { description, .. } => Err(description),
    }
}

pub fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Ermete Greeter")
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
    
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);

    if let Some(display) = gtk4::gdk::Display::default() {
        let provider = gtk4::CssProvider::new();
        provider.load_from_data(GREETER_CSS);
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
    
    let vbox = Box::builder()
        .orientation(Orientation::Vertical)
        .valign(Align::Center)
        .halign(Align::Center)
        .spacing(12)
        .css_classes(["greeter-card"])
        .build();
        
    let os_label = Label::builder()
        .label("ERMETE OS")
        .css_classes(["greeter-os-title"])
        .build();

    let user_label = Label::builder()
        .label("Ermete")
        .css_classes(["greeter-user-name"])
        .build();
    
    let password_entry = Entry::builder()
        .placeholder_text("Password di accesso...")
        .visibility(false)
        .css_classes(["greeter-entry"])
        .build();

    let error_label = Label::builder()
        .label("")
        .css_classes(["greeter-error"])
        .visible(false)
        .wrap(true)
        .build();
        
    let error_label_clone = error_label.clone();
    password_entry.connect_changed(move |_| {
        error_label_clone.set_visible(false);
    });

    let error_label_activate = error_label.clone();
    password_entry.connect_activate(move |entry| {
        let password = entry.text().to_string();
        entry.set_sensitive(false);
        error_label_activate.set_visible(false);
        
        let (sender, receiver) = glib::MainContext::channel::<Result<(), String>>(glib::Priority::DEFAULT);
        
        std::thread::spawn(move || {
            let res = authenticate(&password);
            let _ = sender.send(res);
        });
        
        let entry_clone = entry.clone();
        let err_clone = error_label_activate.clone();
        receiver.attach(None, move |res| {
            match res {
                Ok(_) => {
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("Login failed: {}", e);
                    err_clone.set_text(&format!("Accesso non riuscito: {}", e));
                    err_clone.set_visible(true);
                    entry_clone.set_text("");
                    entry_clone.set_sensitive(true);
                    entry_clone.grab_focus();
                }
            }
            glib::ControlFlow::Break
        });
    });

    vbox.append(&os_label);
    vbox.append(&user_label);
    vbox.append(&password_entry);
    vbox.append(&error_label);
    
    window.set_child(Some(&vbox));
    window.present();
}
