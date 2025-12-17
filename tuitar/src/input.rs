use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, SizedSample, Stream, StreamConfig};

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

        let requested = device_name.to_lowercase();
        let mut matching_device = None;

        match host.input_devices() {
            Ok(devices) => {
                for device in devices {
                    match device.name() {
                        Ok(name) => {
                            if name.to_lowercase().contains(&requested) {
                                matching_device = Some((device, name));
                                break;
                            }
                        }
                        Err(err) => {
                            eprintln!("Skipping input device with unreadable name: {err}");
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("Failed to enumerate input devices: {err}");
            }
        }

        let (device, device_name) = if let Some(device) = matching_device {
            device
        } else {
            let fallback = host
                .default_input_device()
                .expect("No audio input devices detected");

            let name = fallback
                .name()
                .unwrap_or_else(|err| format!("unknown device ({err})"));
            println!("Falling back to default input device: {name}");
            (fallback, name)
        };

        println!("Using input device: {device_name}");

        let supported_config = match device.default_input_config() {
            Ok(config) => config,
            Err(err) => {
                eprintln!("Failed to fetch default input config: {err}");

                let mut chosen_config = None;
                match device.supported_input_configs() {
                    Ok(configs) => {
                        for range in configs {
                            let config = range.with_max_sample_rate();

                            if config.sample_format() == SampleFormat::I16 {
                                chosen_config = Some(config);
                                break;
                            }

                            if chosen_config.is_none() {
                                chosen_config = Some(config);
                            }
                        }
                    }
                    Err(list_err) => {
                        eprintln!("Unable to read supported input configs: {list_err}");
                    }
                }

                chosen_config.expect("Device has no supported audio configs")
            }
        };

        let sample_format = supported_config.sample_format();
        let stream_config = supported_config.config();

        println!("Using config: {stream_config:?}");
        println!("Using sample format: {sample_format:?}");

        let mut device = Some(device);
        let mut stream_config = Some(stream_config);
        let mut callback = Some(callback);

        match sample_format {
            SampleFormat::I16 => Self::build_passthrough(
                device.take().unwrap(),
                stream_config.take().unwrap(),
                callback.take().unwrap(),
            ),
            SampleFormat::U16 => Self::build_stream_with_conversion::<u16, _>(
                device.take().unwrap(),
                stream_config.take().unwrap(),
                callback.take().unwrap(),
            ),
            SampleFormat::I32 => Self::build_stream_with_conversion::<i32, _>(
                device.take().unwrap(),
                stream_config.take().unwrap(),
                callback.take().unwrap(),
            ),
            SampleFormat::U32 => Self::build_stream_with_conversion::<u32, _>(
                device.take().unwrap(),
                stream_config.take().unwrap(),
                callback.take().unwrap(),
            ),
            SampleFormat::F32 => Self::build_stream_with_conversion::<f32, _>(
                device.take().unwrap(),
                stream_config.take().unwrap(),
                callback.take().unwrap(),
            ),
            SampleFormat::F64 => Self::build_stream_with_conversion::<f64, _>(
                device.take().unwrap(),
                stream_config.take().unwrap(),
                callback.take().unwrap(),
            ),
            SampleFormat::I8 => Self::build_stream_with_conversion::<i8, _>(
                device.take().unwrap(),
                stream_config.take().unwrap(),
                callback.take().unwrap(),
            ),
            SampleFormat::U8 => Self::build_stream_with_conversion::<u8, _>(
                device.take().unwrap(),
                stream_config.take().unwrap(),
                callback.take().unwrap(),
            ),
            SampleFormat::I64 => Self::build_stream_with_conversion::<i64, _>(
                device.take().unwrap(),
                stream_config.take().unwrap(),
                callback.take().unwrap(),
            ),
            SampleFormat::U64 => Self::build_stream_with_conversion::<u64, _>(
                device.take().unwrap(),
                stream_config.take().unwrap(),
                callback.take().unwrap(),
            ),
            _ => panic!("Unsupported sample format: {sample_format:?}"),
        }
    }

    pub fn start(&self) {
        self.stream.play().unwrap();
    }

    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate.0
    }
}

impl Recorder {
    fn build_passthrough<D>(device: cpal::Device, config: StreamConfig, mut callback: D) -> Self
    where
        D: FnMut(&[i16], &cpal::InputCallbackInfo) + Send + 'static,
    {
        let stream = device
            .build_input_stream(
                &config,
                move |data: &[i16], info| {
                    callback(data, info);
                },
                |err| {
                    panic!("Error: {err}");
                },
                None,
            )
            .unwrap_or_else(|err| panic!("Failed to build input stream: {err}"));

        Self { stream, config }
    }

    fn build_stream_with_conversion<T, D>(
        device: cpal::Device,
        config: StreamConfig,
        mut callback: D,
    ) -> Self
    where
        T: Sample + SizedSample,
        i16: FromSample<T>,
        D: FnMut(&[i16], &cpal::InputCallbackInfo) + Send + 'static,
    {
        let mut scratch: Vec<i16> = Vec::new();

        let stream = device
            .build_input_stream(
                &config,
                move |data: &[T], info| {
                    scratch.clear();
                    scratch.reserve(data.len());
                    scratch.extend(data.iter().map(|sample| (*sample).to_sample::<i16>()));
                    callback(&scratch, info);
                },
                |err| {
                    panic!("Error: {err}");
                },
                None,
            )
            .unwrap_or_else(|err| panic!("Failed to build input stream: {err}"));

        Self { stream, config }
    }
}
