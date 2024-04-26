use gstreamer as gst;

use gst::prelude::*;

pub fn make_pipeline(url: String, picsink: &gst::Element) -> gst::Pipeline {
    let pipe = gst::Pipeline::new();

    pipe.add(picsink).unwrap();

    let decodebin = gst::ElementFactory::make("uridecodebin3")
        .property("uri", url)
        .property("use-buffering", true)
        .build()
        .unwrap();

    pipe.add(&decodebin).unwrap();

    let vidqueue = gst::ElementFactory::make("queue").build().unwrap();
    pipe.add(&vidqueue).unwrap();

    let vidconvert = gst::ElementFactory::make("videoconvert").build().unwrap();
    pipe.add(&vidconvert).unwrap();

    let sndqueue = gst::ElementFactory::make("queue").build().unwrap();
    pipe.add(&sndqueue).unwrap();

    let sndconvert = gst::ElementFactory::make("audioconvert").build().unwrap();
    pipe.add(&sndconvert).unwrap();

    let sndsink = gst::ElementFactory::make("autoaudiosink").build().unwrap();
    pipe.add(&sndsink).unwrap();

    sndqueue.link(&sndconvert).unwrap();
    sndconvert.link(&sndsink).unwrap();

    vidqueue.link(&vidconvert).unwrap();
    vidconvert.link(picsink).unwrap();

    decodebin.connect_pad_added(move |_, new_pad| {
        let vid_pad = vidqueue.static_pad("sink").unwrap();
        if new_pad.can_link(&vid_pad) {
            if let Err(err) = new_pad.link(&vid_pad) {
                eprintln!("Link error: {:?}", err);
            }
        }

        let snd_pad = sndqueue.static_pad("sink").unwrap();
        if new_pad.can_link(&snd_pad) {
            if let Err(err) = new_pad.link(&snd_pad) {
                eprintln!("Link error: {:?}", err);
            }
        }
    });

    pipe
}
