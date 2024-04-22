use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
use gtk::{Box, Button, Entry};
use gtk4 as gtk;
use reqwest;
use serde_json::{json, Value};

fn get_video_info(id: &str) -> String {
    // https://tyrrrz.me/blog/reverse-engineering-youtube-revisited
    // https://github.com/Tyrrrz/YoutubeExplode

    let client = reqwest::blocking::Client::new();

    client
        .post("https://www.youtube.com/youtubei/v1/player")
        .header(
            "User-Agent",
            "com.google.android.youtube/17.36.4 (Linux; U; Android 12; GB) gzip",
        )
        .json(&json!({
            "videoId": id,
            "context": {
                "client": {
                    "clientName": "ANDROID_TESTSUITE",
                    "clientVersion": "1.9",
                    "androidSdkVersion": 30,
                    "hl": "en",
                    "gl": "US",
                    "utcOffsetMinutes": 0
                }
            }
        }))
        .send()
        .unwrap()
        .text()
        .unwrap()
}

fn on_activate(app: &Application) {
    let win = ApplicationWindow::builder()
        .application(app)
        .default_width(1280)
        .default_height(800)
        .title("Hello, World!")
        .build();

    let input_row = Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .hexpand(true)
        .valign(gtk::Align::Start)
        .spacing(10)
        .margin_top(10)
        .margin_start(10)
        .margin_end(10)
        .build();

    let button = Button::builder().label("Fetch").width_request(160).build();
    let input = Entry::builder().hexpand(true).text("dQw4w9WgXcQ").build();

    input_row.append(&input);
    input_row.append(&button);

    button.connect_clicked(move |_btn: &Button| {
        let id: String = input.text().into();
        println!("{}", id);

        let info = get_video_info(id.as_str());
        let json: Value = serde_json::from_str(&info).unwrap();

        println!("{}", json)
    });

    win.set_child(Some(&input_row));

    win.present();
}

fn main() {
    let app = Application::builder()
        .application_id("org.example.ytp")
        .build();

    app.connect_activate(on_activate);
    app.run();
}
