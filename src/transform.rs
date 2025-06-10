use pitchy::Note;
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;

// TODO: Use <https://docs.rs/microfft> instead
pub struct Transform {
    fft_planner: FftPlanner<f64>,
    fft_samples: Vec<Complex<f64>>,
}

impl Transform {
    pub fn new() -> Self {
        Transform {
            fft_planner: FftPlanner::new(),
            fft_samples: Vec::new(),
        }
    }

    pub fn process(&mut self, samples: &[i16]) {
        let samples_f64: Vec<f64> = samples.iter().map(|&s| s as f64).collect();
        let fft = self.fft_planner.plan_fft_forward(samples.len());
        let mut fft_input: Vec<Complex<f64>> =
            samples_f64.iter().map(|&s| Complex::new(s, 0.0)).collect();
        fft.process(&mut fft_input);
        self.fft_samples = fft_input;
    }

    fn find_fundamental_frequency(&self, sample_rate: f64) -> f64 {
        let mut max_magnitude = 0.0;
        let mut fundamental_freq = 0.0;

        for (i, &sample) in self.fft_samples.iter().enumerate() {
            let magnitude = sample.norm();
            if magnitude > max_magnitude {
                max_magnitude = magnitude;
                fundamental_freq = i as f64 * sample_rate / self.fft_samples.len() as f64;
            }
        }

        fundamental_freq
    }

    pub fn note(&self, sample_rate: u32) -> Note {
        let fundamental_freq = self.find_fundamental_frequency(sample_rate as f64);
        Note::new(fundamental_freq)
    }

    pub fn fft_data(&self) -> Vec<f64> {
        // Only take the first half of the FFT output (the positive frequencies)
        let half_len = self.fft_samples.len() / 2;
        let magnitude_spectrum: Vec<f64> = self
            .fft_samples
            .iter()
            .take(half_len)
            .map(|c| c.norm()) // magnitude = sqrt(re^2 + im^2)
            .collect();

        magnitude_spectrum
    }

    /// Returns the FFT data normalized by the square root of the number of samples.
    ///
    /// See <https://docs.rs/rustfft/latest/rustfft/#normalization>
    pub fn normalized_fft_data(&self) -> Vec<f64> {
        let len = self.fft_samples.len() as f64;
        self.fft_data().iter().map(|&m| m / len.sqrt()).collect()
    }
}
