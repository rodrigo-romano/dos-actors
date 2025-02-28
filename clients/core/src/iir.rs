//! # Infinite Impulse Response (IIR) filter client
//!
//! 

use std::collections::VecDeque;

#[derive(Debug, Clone)]

/// An IIR (Infinite Impulse Response) filter implementation with variable order
pub struct IIRFilter {
    /// Feed-forward coefficients (b values)
    b_coeffs: Vec<f64>,
    /// Feed-back coefficients (a values, a[0] is assumed to be 1.0)
    a_coeffs: Vec<f64>,
    /// Buffer for previous inputs
    x_buffer: VecDeque<f64>,
    /// Buffer for previous outputs
    y_buffer: VecDeque<f64>,
}

// impl<T, U> IIRFilter<U>
// where 
//     T: Default + Clone,
//     U: UniqueIdentifier<Datatype = Vec<T>>,
impl IIRFilter{
    /// Create a new IIR filter with order determined by the coefficient lengths
    ///
    /// # Arguments
    /// * `b_coeffs` - Feed-forward coefficients (b0, b1, b2, ...)
    /// * `a_coeffs` - Feed-back coefficients (a1, a2, ...), note a0 is assumed to be 1.0
    pub fn new(b_coeffs: Vec<f64>, a_coeffs: Vec<f64>) -> Self {
        // The filter order is determined by the maximum length of coefficient arrays
        let b_len = b_coeffs.len();
        let a_len = a_coeffs.len();
        
        println!("Creating IIR filter with {} feed-forward and {} feed-back coefficients", b_len, a_len);
        
        // Initialize input buffer (needs to hold b_len samples)
        let mut x_buffer = VecDeque::with_capacity(b_len);
        for _ in 0..b_len {
            x_buffer.push_front(0.0);
        }
        
        // Initialize output buffer (needs to hold a_len samples)
        let mut y_buffer = VecDeque::with_capacity(a_len);
        for _ in 0..a_len {
            y_buffer.push_front(0.0);
        }
        
        Self {
            b_coeffs,
            a_coeffs,
            x_buffer,
            y_buffer,
        }
    }
    
    /// Process a single sample through the filter
    ///
    /// # Arguments
    /// * `input` - The current input sample
    ///
    /// # Returns
    /// The filtered output sample
    pub fn process(&mut self, input: f64) -> f64 {
        // Update input buffer
        self.x_buffer.pop_back();
        self.x_buffer.push_front(input);
        
        // Calculate output using the difference equation:
        // y[n] = b0*x[n] + b1*x[n-1] + ... - a1*y[n-1] - ... 
        let mut output = 0.0;
        
        // Add the feed-forward terms (b coefficients * input samples)
        for i in 0..self.b_coeffs.len() {
            output += self.b_coeffs[i] * self.x_buffer[i];
        }
        
        // Subtract the feed-back terms (a coefficients * previous outputs)
        for i in 0..self.a_coeffs.len() {
            output -= self.a_coeffs[i] * self.y_buffer[i];
        }
        
        // Update output buffer
        self.y_buffer.pop_back();
        self.y_buffer.push_front(output);
        
        output
    }
    
    /// Reset the filter state
    pub fn reset(&mut self) {
        for i in 0..self.x_buffer.len() {
            self.x_buffer[i] = 0.0;
        }
        
        for i in 0..self.y_buffer.len() {
            self.y_buffer[i] = 0.0;
        }
    }
    
    /// Process a block of samples through the filter
    ///
    /// # Arguments
    /// * `input` - The input samples
    ///
    /// # Returns
    /// The filtered output samples
    pub fn process_block(&mut self, input: &[f64]) -> Vec<f64> {
        let mut output = Vec::with_capacity(input.len());
        
        for &sample in input {
            output.push(self.process(sample));
        }
        
        output
    }
    
    /// Get the order of the filter
    pub fn order(&self) -> usize {
        // The order is determined by the maximum of the number of poles or zeros
        // Number of poles = a_coeffs.len(), number of zeros = b_coeffs.len() - 1
        self.a_coeffs.len().max(self.b_coeffs.len() - 1)
    }
}
