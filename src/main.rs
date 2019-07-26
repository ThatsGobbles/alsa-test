
use std::fs::File;
use std::io::Write;

use alsa::Direction;
use alsa::ValueOr;
use alsa::pcm::PCM;
use alsa::pcm::HwParams;
use alsa::pcm::Format;
use alsa::pcm::Access;
use alsa::pcm::State;

const NUM_CHANNELS: u32 = 2;
const SAMPLE_RATE: u32 = 44100;
const NUM_SECONDS: u32 = 5;

fn main() {
    let pcm = PCM::new("default", Direction::Capture, false).unwrap();

    let hwp = HwParams::any(&pcm).unwrap();
    hwp.set_channels(NUM_CHANNELS).unwrap();
    hwp.set_rate(SAMPLE_RATE, ValueOr::Nearest).unwrap();
    hwp.set_format(Format::FloatLE).unwrap();
    hwp.set_access(Access::RWInterleaved).unwrap();
    pcm.hw_params(&hwp).unwrap();
    let io = pcm.io_f32().unwrap();

    // Make sure we don't start the stream too early
    let hwp = pcm.hw_params_current().unwrap();
    let swp = pcm.sw_params_current().unwrap();
    swp.set_start_threshold(hwp.get_buffer_size().unwrap() - hwp.get_period_size().unwrap()).unwrap();
    pcm.sw_params(&swp).unwrap();

    // Enough buffer space for number of seconds desired.
    let buf_len = (NUM_CHANNELS * SAMPLE_RATE * NUM_SECONDS) as usize;
    let mut buf = vec![0.0f32; buf_len];

    assert_eq!(io.readi(buf.as_mut_slice()).unwrap(), (SAMPLE_RATE * NUM_SECONDS) as usize);

    // In case the buffer was larger than 2 seconds, start the stream manually.
    if pcm.state() != State::Running { pcm.start().unwrap() };
    // Wait for the stream to finish playback.
    pcm.drain().unwrap();

    let mut output_file = File::create("wav_data.csv").unwrap();

    for sample in buf.as_slice().chunks_exact(NUM_CHANNELS as usize) {
        writeln!(output_file, "{},{}", sample[0], sample[1]).unwrap();
    }
}
