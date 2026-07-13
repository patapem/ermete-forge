use gtk4::prelude::*;
use gtk4::{glib, Application, ApplicationWindow, Box, Button, HeaderBar, Label, ListBox, Orientation, ScrolledWindow, SearchEntry, Stack, StackSidebar};
use crate::backend;

fn create_app_row(app: &backend::AppInfo) -> Box {
    let row_box = Box::new(Orientation::Horizontal, 12);
    row_box.set_margin_top(12);
    row_box.set_margin_bottom(12);
    row_box.set_margin_start(12);
    row_box.set_margin_end(12);

    let text_box = Box::new(Orientation::Vertical, 4);
    text_box.set_hexpand(true);

    let title = Label::builder()
        .label(&app.name)
        .halign(gtk4::Align::Start)
        .css_classes(vec!["heading"])
        .build();
    
    let subtitle = Label::builder()
        .label(&app.description)
        .halign(gtk4::Align::Start)
        .css_classes(vec!["dim-label"])
        .wrap(true)
        .build();

    text_box.append(&title);
    text_box.append(&subtitle);

    let install_btn = Button::with_label("Installa");
    install_btn.set_valign(gtk4::Align::Center);
    
    let app_id = app.id.clone();
    let btn_clone = install_btn.clone();
    install_btn.connect_clicked(move |_| {
        let app_id_clone = app_id.clone();
        let btn = btn_clone.clone();
        btn.set_sensitive(false);
        btn.set_label("Installando...");
        glib::spawn_future_local(async move {
            let _ = backend::install_app(&app_id_clone).await;
            btn.set_label("Fatto");
        });
    });

    row_box.append(&text_box);
    row_box.append(&install_btn);

    row_box
}

pub fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Ermete Store")
        .default_width(900)
        .default_height(600)
        .build();

    let content_box = Box::new(Orientation::Vertical, 0);
    
    let header = HeaderBar::new();
    content_box.append(&header);

    let hbox = Box::new(Orientation::Horizontal, 0);
    let stack = Stack::new();
    stack.set_vexpand(true);
    stack.set_hexpand(true);
    stack.set_transition_type(gtk4::StackTransitionType::Crossfade);

    let sidebar = StackSidebar::new();
    sidebar.set_stack(&stack);
    sidebar.set_size_request(200, -1);
    
    hbox.append(&sidebar);
    hbox.append(&stack);
    content_box.append(&hbox);
    window.set_child(Some(&content_box));

    // Featured
    let featured_box = Box::new(Orientation::Vertical, 12);
    featured_box.set_margin_top(24);
    featured_box.set_margin_bottom(24);
    featured_box.set_margin_start(24);
    featured_box.set_margin_end(24);
    
    let featured_label = Label::builder()
        .label("In primo piano")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["title-1"])
        .build();
    featured_box.append(&featured_label);
    
    let featured_list = ListBox::new();
    featured_list.set_selection_mode(gtk4::SelectionMode::None);
    featured_box.append(&featured_list);
    
    let featured_scroll = ScrolledWindow::new();
    featured_scroll.set_child(Some(&featured_box));
    stack.add_titled(&featured_scroll, Some("featured"), "Primo Piano");

    let (tx, rx) = async_channel::unbounded();
    let featured_list_clone = featured_list.clone();
    
    glib::spawn_future_local(async move {
        if let Ok(apps) = backend::get_featured_apps().await {
            let _ = tx.send(apps).await;
        }
    });

    let rx_clone = rx.clone();
    glib::spawn_future_local(async move {
        while let Ok(apps) = rx_clone.recv().await {
            for app in apps {
                let row = create_app_row(&app);
                featured_list_clone.append(&row);
            }
        }
    });

    // Categories / Search
    let search_box = Box::new(Orientation::Vertical, 12);
    search_box.set_margin_top(24);
    search_box.set_margin_bottom(24);
    search_box.set_margin_start(24);
    search_box.set_margin_end(24);
    
    let search_entry = SearchEntry::new();
    search_box.append(&search_entry);
    
    let search_list = ListBox::new();
    search_list.set_selection_mode(gtk4::SelectionMode::None);
    let search_scroll = ScrolledWindow::new();
    search_scroll.set_child(Some(&search_list));
    search_scroll.set_vexpand(true);
    search_box.append(&search_scroll);
    
    stack.add_titled(&search_box, Some("categories"), "Cerca");

    let search_list_clone = search_list.clone();
    search_entry.connect_search_changed(move |entry| {
        let query = entry.text().to_string();
        let search_list = search_list_clone.clone();
        if query.len() < 3 { return; }
        
        glib::spawn_future_local(async move {
            if let Ok(apps) = backend::search_apps(&query).await {
                while let Some(child) = search_list.first_child() {
                    search_list.remove(&child);
                }
                
                for app in apps {
                    let row = create_app_row(&app);
                    search_list.append(&row);
                }
            }
        });
    });

    // Updates
    let updates_box = Box::new(Orientation::Vertical, 12);
    updates_box.set_margin_top(24);
    updates_box.set_margin_bottom(24);
    updates_box.set_margin_start(24);
    updates_box.set_margin_end(24);
    
    let update_title = Label::builder()
        .label("Aggiornamenti di Sistema")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["title-1"])
        .build();
    updates_box.append(&update_title);
    
    let sys_update_btn = Button::builder()
        .label("Aggiorna Sistema (rpm-ostree)")
        .css_classes(vec!["suggested-action"])
        .halign(gtk4::Align::Center)
        .build();
        
    let update_status = Label::new(Some("Sistema pronto."));
    update_status.set_wrap(true);
    
    updates_box.append(&sys_update_btn);
    updates_box.append(&update_status);
    
    let status_clone = update_status.clone();
    let sys_update_btn_clone = sys_update_btn.clone();
    sys_update_btn.connect_clicked(move |_| {
        let status = status_clone.clone();
        let btn = sys_update_btn_clone.clone();
        btn.set_sensitive(false);
        status.set_label("Aggiornamento in corso. L'operazione potrebbe richiedere alcuni minuti...");
        
        glib::spawn_future_local(async move {
            match backend::update_system().await {
                Ok(out) => status.set_label(&format!("Aggiornamento completato:\n{}", out)),
                Err(e) => status.set_label(&format!("Errore durante l'aggiornamento:\n{}", e)),
            }
            btn.set_sensitive(true);
        });
    });
    
    let updates_scroll = ScrolledWindow::new();
    updates_scroll.set_child(Some(&updates_box));
    stack.add_titled(&updates_scroll, Some("updates"), "Aggiornamenti");

    window.present();
}
