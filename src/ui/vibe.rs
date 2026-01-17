use crate::ui::protocol::{send_rio, RioAction};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use realfft::RealFftPlanner;
use std::sync::{Arc, Mutex};

pub fn start_vibe_engine() {
    let host = cpal::default_host();
    let device = host.default_input_device().expect("No input device available");
    let config = device.default_input_config().expect("No default input config");

    let _sample_rate = config.sample_rate();
    let fft_size = 1024;
    let mut planner = RealFftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(fft_size);
    
    let spectrum_bins = 8;
    let bins = Arc::new(Mutex::new(vec![0.0f32; spectrum_bins]));
    let bins_clone = bins.clone();

    let stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &_| {
            if data.len() < fft_size { return; }
            
            let mut input = vec![0.0f32; fft_size];
            input.copy_from_slice(&data[0..fft_size]);
            
            let mut output = fft.make_output_vec();
            if fft.process(&mut input, &mut output).is_ok() {
                let mut new_bins = vec![0.0f32; spectrum_bins];
                let bin_size = (fft_size / 2) / spectrum_bins;
                
                for i in 0..spectrum_bins {
                    let start = i * bin_size;
                    let end = (i + 1) * bin_size;
                    let mut sum = 0.0;
                    for j in start..end {
                        sum += output[j].norm();
                    }
                    // Scale and normalize (crude)
                    new_bins[i] = (sum / bin_size as f32 / 5.0).min(1.0);
                }
                
                if let Ok(mut b) = bins_clone.lock() {
                    *b = new_bins;
                }
            }
        },
        move |err| {
            eprintln!("Audio stream error: {}", err);
        },
        None,
    ).expect("Failed to build audio stream");

    stream.play().expect("Failed to play audio stream");

    // Spawn a thread to periodically send spectrum to Rio
    std::thread::spawn(move || {
        loop {
            let data = {
                if let Ok(b) = bins.lock() {
                    b.clone()
                } else {
                    vec![0.0; spectrum_bins]
                }
            };
            
            send_rio(RioAction::Spectrum(data));
            std::thread::sleep(std::time::Duration::from_millis(50)); // ~20fps
        }
    });

    // We keep the stream alive by leaking it or keeping it in a global state
    // For now, let's just leak it to keep the prototype simple
    Box::leak(Box::new(stream));
}
