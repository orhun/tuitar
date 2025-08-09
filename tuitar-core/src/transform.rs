/// A trait for Fast Fourier Transform (FFT) processing.
// I was not able to find a better name, I like transformers
pub trait Transformer {
    /// Processes the input audio samples and prepares the FFT data.
    fn process(&mut self, samples: &[i16]);

    /// Finds the fundamental frequency from the processed FFT data.
    fn find_fundamental_frequency(&self, sample_rate: f64) -> f64;

    /// Returns the FFT data.
    fn fft_data(&self) -> Vec<f64>;

    /// Returns the normalized FFT data.
    fn normalized_fft_data(&self) -> Vec<f64>;
}
