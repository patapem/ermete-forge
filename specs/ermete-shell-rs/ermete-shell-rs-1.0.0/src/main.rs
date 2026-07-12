use clap::Parser;
use gtk4::prelude::*;
use gtk4::Application;

// Dummy modules for now
mod topbar;
mod greeter;
mod core;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    topbar: bool,
    #[arg(long)]
    greeter: bool,
}

const APP_ID: &str = "os.ermete.Shell";

fn main() -> glib::ExitCode {
    let args = Args::parse();
    
    let app = Application::builder().application_id(APP_ID).build();
    
    app.connect_activate(move |app| {
        if args.topbar {
            topbar::build_ui(app);
        } else if args.greeter {
            greeter::build_ui(app);
        } else {
            eprintln!("Error: specify --topbar or --greeter");
        }
    });
    
    app.run()
}
