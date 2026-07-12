use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Entry, Label, Orientation, Align};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::os::unix::net::UnixStream;
use std::io::{Read, Write};
use greetd_ipc::{Request, Response};

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

    let username = std::env::var("USER").unwrap_or_else(|_| "ermete".to_string());
    let req = Request::CreateSession { username };
    let resp = send_request(&mut stream, &req)?;

    match resp {
        Response::AuthMessage { .. } => {
            let req = Request::PostAuthMessageResponse { response: Some(password.to_string()) };
            let resp = send_request(&mut stream, &req)?;
            match resp {
                Response::Success => {
                    let req = Request::StartSession {
                        cmd: vec!["ermete-session".to_string()],
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
                 cmd: vec!["ermete-session".to_string()],
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
    
    let vbox = Box::builder()
        .orientation(Orientation::Vertical)
        .valign(Align::Center)
        .halign(Align::Center)
        .spacing(16)
        .build();
        
    let user_label = Label::new(Some("Ermete"));
    
    let password_entry = Entry::builder()
        .placeholder_text("Password...")
        .visibility(false)
        .build();
        
    password_entry.connect_activate(move |entry| {
        let password = entry.text().to_string();
        entry.set_sensitive(false);
        
        let (sender, receiver) = glib::MainContext::channel::<Result<(), String>>(glib::Priority::DEFAULT);
        
        std::thread::spawn(move || {
            let res = authenticate(&password);
            let _ = sender.send(res);
        });
        
        let entry_clone = entry.clone();
        receiver.attach(None, move |res| {
            match res {
                Ok(_) => {
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("Login failed: {}", e);
                    entry_clone.set_text("");
                    entry_clone.set_sensitive(true);
                    entry_clone.grab_focus();
                }
            }
            glib::ControlFlow::Break
        });
    });

    vbox.append(&user_label);
    vbox.append(&password_entry);
    
    window.set_child(Some(&vbox));
    window.present();
}
