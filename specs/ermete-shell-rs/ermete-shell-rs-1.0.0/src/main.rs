use clap::Parser;
use gtk4::prelude::*;
use gtk4::Application;

// Dummy modules for now
mod ui;
mod greeter;
mod core;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    topbar: bool,
    #[arg(long)]
    greeter: bool,
    #[arg(long)]
    spotlight: bool,
}

const APP_ID: &str = "os.ermete.Shell";

fn main() -> glib::ExitCode {
    let args = Args::parse();
    
    // We need a unique app_id if we want spotlight to run concurrently without DBus collision for now
    let app_id = if args.spotlight { "os.ermete.Spotlight" } else { APP_ID };
    let app = Application::builder().application_id(app_id).build();
    
    app.connect_activate(move |app| {
        if args.topbar {
            ui::topbar::build_ui(app);
            crate::ui::osd::spawn_osd(app);
        } else if args.greeter {
            greeter::build_ui(app);
        } else if args.spotlight {
            ui::spotlight::show_spotlight_modal(app);
        } else {
            eprintln!("Error: specify --topbar, --greeter, or --spotlight");
        }
    });
    
    app.run_with_args(&Vec::<String>::new())
}
