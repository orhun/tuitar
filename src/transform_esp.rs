use microfft::real::rfft_512;
use num_complex::Complex32;
use pitchy::Note;

const FFT_SIZE: usize = 512;

pub struct Transform {
    fft_samples: [Complex32; FFT_SIZE / 2],
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            fft_samples: [Complex32::new(0.0, 0.0); FFT_SIZE / 2],
        }
    }
}

impl Transform {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn process(&mut self, samples: &[i16]) {
        assert_eq!(
            samples.len(),
            FFT_SIZE,
            "microfft only supports fixed sizes; got {}",
            samples.len()
        );

        // Convert i16 samples to f32
        let mut buffer = [0.0_f32; FFT_SIZE];
        // This will fill buffer with as many samples as possible (up to FFT_SIZE).
        for (i, s) in samples.iter().take(FFT_SIZE).enumerate() {
            buffer[i] = *s as f32;
        }

        // Compute FFT in-place
        let spectrum = rfft_512(&mut buffer);
        self.fft_samples.copy_from_slice(spectrum);
        // microfft packs Nyquist freq in spectrum[0].im, so clear for amplitude
        self.fft_samples[0].im = 0.0;
    }

    fn find_fundamental_frequency(&self, sample_rate: f32) -> f32 {
        let mut max_magnitude = 0.0;
        let mut fundamental_freq = 0.0;

        for (i, sample) in self.fft_samples.iter().enumerate() {
            let magnitude = sample.norm();
            if magnitude > max_magnitude {
                max_magnitude = magnitude;
                fundamental_freq = i as f32 * sample_rate / FFT_SIZE as f32;
            }
        }

        fundamental_freq
    }

    pub fn note(&self, sample_rate: u32) -> Note {
        let fundamental_freq = self.find_fundamental_frequency(sample_rate as f32);
        Note::new(fundamental_freq as f64)
    }

    pub fn fft_data(&self) -> Vec<f64> {
        self.fft_samples.iter().map(|c| c.norm() as f64).collect()
    }

    pub fn normalized_fft_data(&self) -> Vec<f64> {
        let len = FFT_SIZE as f32;
        self.fft_data()
            .iter()
            .map(|&m| m / len.sqrt() as f64)
            .collect()
    }
}
