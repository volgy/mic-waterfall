use std::sync::{Arc, Mutex};

use easyfft::prelude::*;
use nannou::prelude::*;
use nannou_audio as audio;

const N_FFT: usize = 512;
const TIME_SCALE: f32 = 5.0;
const SAMPLE_RATE: u32 = 44_100;

struct AudioModel {
    spectrogram: Arc<Mutex<Vec<Vec<f32>>>>,
}

struct Model {
    stream: audio::Stream<AudioModel>,
    spectrogram: Arc<Mutex<Vec<Vec<f32>>>>,
}

fn model(_app: &App) -> Model {
    let audio_host = audio::Host::new();

    let spectrogram = Arc::new(Mutex::new(vec![vec![]]));
    let audio_model = AudioModel {
        spectrogram: Arc::clone(&spectrogram),
    };
    let stream = audio_host
        .new_input_stream(audio_model)
        .sample_rate(SAMPLE_RATE)
        .channels(1)
        .frames_per_buffer(N_FFT)
        .capture(mic_receiver)
        .build()
        .expect("Failed to open microphone input stream with desired settings");

    stream
        .play()
        .expect("Failed to start microphone input stream");

    Model {
        stream,
        spectrogram,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    // Hack: it seems that stream.play() does not work in the model() function
    if model.stream.is_paused() {
        model.stream.play().unwrap();
    }

    let height = app.window_rect().h() / TIME_SCALE;
    let mut spectrogram = model.spectrogram.lock().unwrap();
    let excess = spectrogram.len() as f32 - height;
    if excess > 0.0 {
        spectrogram.drain(0..excess as usize);
    }

    // eprintln!("len = {:#?}", spectrogram.len());
}

fn view(app: &App, _model: &Model, frame: Frame) {
    let draw = app
        .draw()
        .x_y(-app.window_rect().w() / 2.0, -app.window_rect().h() / 2.0);
    draw.background().color(BLACK);

    let spectrogram = _model.spectrogram.lock().unwrap();
    let width = app.window_rect().w() / N_FFT as f32 * 2.0;
    for (i, spectrum) in spectrogram.iter().enumerate() {
        let y = i as f32 * TIME_SCALE;
        for (j, &dbfs) in spectrum.iter().enumerate() {
            let x = j as f32 * width;
            let h = TIME_SCALE;
            let color = hsl(1.0 + dbfs / 100.0, 1.0, 0.5);
            draw.rect().x_y(x, y).w_h(width, h).color(color);
        }
    }

    draw.to_frame(app, &frame).unwrap();
}

fn mic_receiver(audio_model: &mut AudioModel, buffer: &audio::Buffer) {
    assert_eq!(buffer.len(), N_FFT);
    assert_eq!(buffer.channels(), 1);
    let spectrum = buffer
        .fft()
        .iter()
        .take(N_FFT / 2)
        .map(|c| c.norm_sqr().log10() * 10.0)
        .collect::<Vec<f32>>();
    audio_model.spectrogram.lock().unwrap().push(spectrum);
}

fn main() {
    nannou::app(model).update(update).simple_window(view).run();
}
