use gstreamer as gst;
use gtk4 as gtk;

use std::sync::OnceLock;
use tokio::runtime::{Runtime, Builder};
use gst::prelude::*;
use gtk::gdk;
use gtk::{prelude::*, Application, ApplicationWindow, Picture};
use std::env;
mod gstlib;
mod ytlib;

fn tokio_rt() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        return Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()
        .expect("Setting up tokio runtime needs to succeed.");
    })
}

fn create_ui(app: &Application) {
    let win = ApplicationWindow::builder()
        .application(app)
        .default_width(1280)
        .default_height(800)
        .build();

    let stack = gtk::Stack::builder().build();

    let pic = Picture::builder().hexpand(true).vexpand(true).build();
    let btn = gtk::Button::builder().label("Test").hexpand(false).vexpand(false).build();

    stack.add_child(&btn);
    stack.add_child(&pic);

    win.set_child(Some(&stack));
    win.present();

    let api_key = env::var("YOUTUBE_API_KEY").unwrap_or("".into());

    btn.connect_clicked(move |_| {
        let info = tokio_rt().block_on(async{
            let list = ytlib::video_chart("UA", api_key.as_str()).await;
            ytlib::get_video_info(list.items[0].id.as_str()).await
        });

        win.set_title(Some(&info.video_details.title));

        let url = if let Some(formats) = info.streaming_data.formats {
            if let Some(best) = ytlib::find_best_format(formats) {
                Some(best.url.clone())
            } else {
                None
            }
        // todo: pick and process best audio/video pair
        } else if let Some(formats) = info.streaming_data.adaptive_formats {
            Some(formats[0].url.clone())
        } else if let Some(hls) = info.streaming_data.hls_manifest_url {
            Some(hls.clone())
        } else if let Some(dash) = info.streaming_data.dash_manifest_url {
            Some(dash.clone())
        } else {
            None
        };

        match url {
            Some(url) => {
                let picsink = gst::ElementFactory::make("gtk4paintablesink")
                    .property("sync", true)
                    .build()
                    .unwrap();
                pic.set_paintable(Some(&picsink.property::<gdk::Paintable>("paintable")));

                let pipe = gstlib::make_pipeline(url, &picsink);

                pipe.set_state(gst::State::Playing).unwrap();
                stack.set_visible_child(&pic);
            }
            None => {
                eprintln!("No url to play found");
                return;
            }
        }
    });
}

fn main() { 
    gst::init().unwrap();
    gtk::init().unwrap();

    gstgtk4::plugin_register_static().expect("Failed to register gstgtk4 plugin");

    let app = Application::builder()
        .application_id("org.example.ytp")
        .build();

    app.connect_activate(create_ui);
    app.run();
}
