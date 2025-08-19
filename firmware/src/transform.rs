use microfft::real;
use num_complex::Complex32;
use tuitar_core::transform::Transformer;

pub const FFT_SIZE: usize = 512;

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
}

impl Transformer for Transform {
    fn process(&mut self, samples: &[i16]) {
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
        let spectrum = real::rfft_512(&mut buffer);
        self.fft_samples.copy_from_slice(spectrum);
        // microfft packs Nyquist freq in spectrum[0].im, so clear for amplitude
        self.fft_samples[0].im = 0.0;
    }

    fn find_fundamental_frequency(&self, sample_rate: f64) -> f64 {
        let freq_nyquist = sample_rate as f32 / 2.0;
        let len = FFT_SIZE as f32;

        // Iterate once, track max index directly
        let mut max_index = 0;
        let mut max_val = f32::MIN;

        for (i, c) in self
            .fft_samples
            .iter()
            .enumerate()
            .skip(1)
            .take(FFT_SIZE - 2)
        {
            let mag = (c.norm() / len.sqrt()) as f32;
            if mag > max_val {
                max_val = mag;
                max_index = i;
            }
        }

        let bin_width = freq_nyquist / FFT_SIZE as f32;

        if max_index == 0 || max_index + 1 >= FFT_SIZE {
            return max_index as f64 * bin_width as f64;
        }

        // compute 3-bin interpolation *without* allocating whole magnitudes vec
        let y0 = (self.fft_samples[max_index - 1].norm() / len.sqrt())
            .max(1e-12)
            .ln();
        let y1 = (self.fft_samples[max_index].norm() / len.sqrt())
            .max(1e-12)
            .ln();
        let y2 = (self.fft_samples[max_index + 1].norm() / len.sqrt())
            .max(1e-12)
            .ln();

        let delta = 0.5 * (y0 - y2) / (y0 - 2.0 * y1 + y2);
        let bin = max_index as f32 + delta;
        (bin * bin_width) as f64
    }

    fn fft_data(&self) -> Vec<f64> {
        self.fft_samples.iter().map(|c| c.norm() as f64).collect()
    }

    fn normalized_fft_data(&self) -> Vec<f64> {
        let len = FFT_SIZE as f32;
        self.fft_data()
            .iter()
            .map(|&m| m / len.sqrt() as f64)
            .collect()
    }
}
