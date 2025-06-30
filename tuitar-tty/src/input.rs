use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};

pub struct Recorder {
    stream: Stream,
    config: StreamConfig,
}

impl Recorder {
    pub fn init<D>(device_name: &str, callback: D) -> Self
    where
        D: FnMut(&[i16], &cpal::InputCallbackInfo) + Send + 'static,
    {
        let host = cpal::default_host();

        let device = host
            .input_devices()
            .unwrap()
            .find(|d| d.name().unwrap().contains(device_name))
            .expect("No pipewire input device found");

        println!("Using input device: {:?}", device.name().unwrap());

        let mut supported_configs_range = device.supported_input_configs().unwrap();
        let supported_config = supported_configs_range
            .next()
            .expect("no supported config")
            .with_max_sample_rate()
            .config();

        println!("Using config: {:?}", supported_config,);

        let stream = device
            .build_input_stream(
                &supported_config,
                callback,
                |err| {
                    panic!("Error: {err}");
                },
                None,
            )
            .unwrap();

        Self {
            stream,
            config: supported_config,
        }
    }

    pub fn start(&self) {
        self.stream.play().unwrap();
    }

    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate.0
    }
}
