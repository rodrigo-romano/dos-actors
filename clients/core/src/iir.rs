//! # Infinite Impulse Response (IIR) filter client
//!
//! 

use std::collections::VecDeque;

#[derive(Debug, Clone)]

/// A multi-dimensional IIR filter where the same coefficients are applied to all dimensions
pub struct IIRFilter {
    /// Feed-forward coefficients (b values)
    b_coeffs: Vec<f64>,
    /// Feedback coefficients (a values excluding a[0] which is assumed to be 1.0)
    a_coeffs: Vec<f64>,
    /// Number of dimensions to filter
    filter_dim: usize,
    /// Input history for each dimension [dimension][sample]
    x_history: Vec<VecDeque<f64>>,
    /// Output history for each dimension [dimension][sample]
    y_history: Vec<VecDeque<f64>>,
}

// impl<T, U> IIRFilter<U>
// where 
//     T: Default + Clone,
//     U: UniqueIdentifier<Datatype = Vec<T>>,
impl IIRFilter {
    /// Create a new multi-dimensional IIR filter with the given coefficients
    pub fn new(b_coeffs: Vec<f64>, a_coeffs: Vec<f64>, filter_dim: usize) -> Self {
        // Initialize history buffers with zeros
        let x_history = vec![VecDeque::from(vec![0.0; b_coeffs.len()]); filter_dim];
        let y_history = vec![VecDeque::from(vec![0.0; a_coeffs.len()]); filter_dim];
        
        IIRFilter {
            b_coeffs,
            a_coeffs,
            filter_dim,
            x_history,
            y_history,
        }
    }
    
   /// Process a new multi-dimensional input sample using iterators
   pub fn process(&mut self, input: &[f64]) -> Vec<f64> {
    if input.len() != self.filter_dim {
        panic!("Input dimension {} does not match filter dimension {}", 
               input.len(), self.filter_dim);
    }
    
    // Process each dimension using iterators
    (0..self.filter_dim)
        .map(|dim| {
            // Update input history
            self.x_history[dim].pop_back();
            self.x_history[dim].push_front(input[dim]);
            
            // Apply feed-forward coefficients using zip and sum
            let feed_forward = self.b_coeffs.iter()
                .zip(self.x_history[dim].iter())
                .map(|(&b, &x)| b * x)
                .sum::<f64>();
            
            // Apply feedback coefficients using zip and sum
            let feedback = self.a_coeffs.iter()
                .zip(self.y_history[dim].iter())
                .map(|(&a, &y)| a * y)
                .sum::<f64>();
            
            // Calculate output
            let y = feed_forward - feedback;
            
            // Update output history
            self.y_history[dim].pop_back();
            self.y_history[dim].push_front(y);
            
            y
        })
        .collect()
}

/// Reset the filter state using iterators
pub fn reset(&mut self) {
    // Flatten both histories into a single iterator of mutable references and set all to zero
    self.x_history.iter_mut()
        .flat_map(|queue| queue.iter_mut())
        .chain(self.y_history.iter_mut().flat_map(|queue| queue.iter_mut()))
        .for_each(|val| *val = 0.0);
}

/// Change the filter input dimension
pub fn resize(&mut self, new_filter_dim: usize) {
    if new_filter_dim == self.filter_dim {
        return;
    }
    
    self.filter_dim = new_filter_dim;
    self.x_history = vec![VecDeque::from(vec![0.0; self.b_coeffs.len()]); new_filter_dim];
    self.y_history = vec![VecDeque::from(vec![0.0; self.a_coeffs.len()]); new_filter_dim];
}
}
