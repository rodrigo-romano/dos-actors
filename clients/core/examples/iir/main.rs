use gmt_dos_clients::iir::IIRFilter;

// Example usage showing different filter orders
// cargo run -r --example iir
fn main() {
    // Create a simple low-pass filter (example coefficients)
    let b_coeffs = vec![0.0, 0.2, 0.2];  // Feed-forward coefficients
    let a_coeffs = vec![0.1, 0.05];      // Feedback coefficients (excluding a[0]=1.0)
    let n_u = 14;
    
    let mut filter = IIRFilter::new(b_coeffs, a_coeffs, n_u);
    
    // Example input vector (one time sample with 14 dimensions)
    let input = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0];
    
    // Process the input
    let output = filter.process(&input);
    
    println!("Input: {:?}", input);
    println!("Output: {:?}", output);
    
    // Another example: process a sequence of input samples
    println!("\nProcessing a sequence of samples:");
    
    // Reset the filter state
    filter.reset();
    
    // Create some example time-series data
    let time_samples = vec![
        vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0], // t=0
        vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], // t=1
        vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0], // t=2
        vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], // t=3
        vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0], // t=4
    ];
    
    // Process each time sample
    for (i, sample) in time_samples.iter().enumerate() {
        let filtered = filter.process(sample);
        println!("t={}Ts: First dimension output = {}", i, filtered[0]);
    }
}