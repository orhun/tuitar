use microfft::real;
use num_complex::Complex32;

const FFT_SIZE: usize = 1024;

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
        let spectrum = real::rfft_1024(&mut buffer);
        self.fft_samples.copy_from_slice(spectrum);
        // microfft packs Nyquist freq in spectrum[0].im, so clear for amplitude
        self.fft_samples[0].im = 0.0;
    }

    pub fn find_fundamental_frequency(&self, sample_rate: f32) -> f32 {
        let freq_nyquist = sample_rate / 2.0;

        let magnitudes = self.normalized_fft_data();

        let (max_index, _) = magnitudes
            .iter()
            .enumerate()
            .skip(1)
            .take(magnitudes.len() - 2) // Ensure we don't hit the end bins
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap();

        let bin_width = freq_nyquist / magnitudes.len() as f32;

        // Clamp indices to valid range
        if max_index == 0 || max_index + 1 >= magnitudes.len() {
            return max_index as f32 * bin_width;
        }

        // Use log-magnitudes for interpolation
        let y0 = magnitudes[max_index - 1].max(1e-12).ln();
        let y1 = magnitudes[max_index].max(1e-12).ln();
        let y2 = magnitudes[max_index + 1].max(1e-12).ln();

        let delta = 0.5 * (y0 - y2) / (y0 - 2.0 * y1 + y2);
        let bin = max_index as f32 + delta as f32;
        let freq = bin * bin_width;
        freq
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
