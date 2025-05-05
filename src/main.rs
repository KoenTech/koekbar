use async_channel::{Receiver, Sender, bounded};
use background::BackgroundService;
use glib::clone;
use gtk::gdk::Display;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, CenterBox, CssProvider, Label, glib};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::time::Duration;

use crate::types::StatusUpdate;

pub mod background;
pub mod types;

const APP_ID: &str = "dev.koentech.koekbar";

fn main() -> glib::ExitCode {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_startup(|_| load_css());
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_string(include_str!("./style/style.css"));

    gtk::style_context_add_provider_for_display(
        &Display::default().expect("no display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn build_ui(app: &Application) {
    let (tx, rx): (Sender<StatusUpdate>, Receiver<StatusUpdate>) = bounded(1);

    let background_service = BackgroundService {
        update_period: Duration::from_secs(1),
    };

    background_service.spawn(tx);

    let time_text = Label::builder()
        .label("00:00")
        .css_classes(vec!["time"])
        .build();
    let date_text = Label::builder()
        .label("Mon 1 Jan")
        .css_classes(vec!["pill"])
        .build();
    let media_text = Label::builder()
        .label("test")
        .css_classes(vec!["pill"])
        .build();

    let layout_box = CenterBox::builder()
        .center_widget(&time_text)
        .start_widget(&media_text)
        .end_widget(&date_text)
        .build();

    glib::spawn_future_local(clone!(
        #[strong]
        rx,
        #[weak]
        time_text,
        #[weak]
        media_text,
        #[weak]
        date_text,
        #[weak]
        layout_box,
        async move {
            while let Ok(status) = rx.recv().await {
                time_text.set_label(status.time.format("%H:%M").to_string().as_str());
                date_text.set_label(status.time.format("%a %b %e").to_string().as_str());

                match status.media {
                    Some(media) => {
                        if layout_box.start_widget().is_none() {
                            layout_box.set_start_widget(Some(&media_text));
                        }
                        match media.author {
                            None => {
                                media_text.set_text(format!("♫ {}", media.title).as_str());
                            }
                            Some(author) => {
                                media_text
                                    .set_label(format!("♫ {} - {}", media.title, author).as_str());
                            }
                        }
                    }
                    None => {
                        media_text.set_label("");
                        layout_box.set_start_widget(None::<&gtk::Widget>);
                    }
                }
            }
        }
    ));

    // Create a window and set the title
    let window = ApplicationWindow::builder()
        .application(app)
        .title("koekbar")
        .default_height(30)
        .child(&layout_box)
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Top);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);
    window.set_decorated(false);
    window.auto_exclusive_zone_enable();
    window.set_namespace(Some("koekbar"));

    // Present window
    window.present();
}
