extern crate framing;
extern crate x264_framing;

use framing::{Chunky, Function, Bgra};
use std::io::Write;
use std::fs::File;
use x264_framing::{Setup, Preset, Tune};

fn main() {
    let mut encoder =
        Setup::preset(Preset::Veryfast, Tune::None, false, false)
            .width(1280)
            .height(720)
            .fps(60, 1)
            .build()
            .unwrap();

    let mut file = File::create("fade.h264").unwrap();
    let mut canvas = Chunky::new(
        Function::new(1280, 720, |_, _| Bgra(0, 0, 0, 0))
    );

    file.write_all(encoder.headers().unwrap().entirety()).unwrap();

    for i in 0..255 {
        canvas.copy_from(
            Function::new(1280, 720, |_, _| Bgra(0, i, 0, 255))
        );

        let (data, _) = encoder.encode(i as _, &canvas).unwrap();
        file.write_all(data.entirety()).unwrap();
    }

    while !encoder.done() {
        let (data, _) = encoder.work().unwrap();
        file.write_all(data.entirety()).unwrap();
    }

    println!("Done! The output is at `fade.h264`.");
    println!("Try playing it with VLC, and prepare to be underwhelmed! ;)");
}
