//use iir::IIRFilter;
use gmt_dos_clients::iir::IIRFilter;

// Example usage showing different filter orders
fn main() {
    // Example 1: 4th order filter (same as before)
    let b_coeffs = vec![0.0048, 0.0193, 0.0289, 0.0193, 0.0048]; // 5 coefficients
    let a_coeffs = vec![-2.3695, 2.3139, -1.0546, 0.1873];       // 4 coefficients
    
    let mut filter = IIRFilter::new(b_coeffs, a_coeffs);
    println!("Filter order: {}", filter.order());
    
    // Example 2: 2nd order filter
    let b_coeffs_2 = vec![0.1, 0.2, 0.1];  // 3 coefficients
    let a_coeffs_2 = vec![-1.2, 0.5];      // 2 coefficients
    
    let mut filter_2 = IIRFilter::new(b_coeffs_2, a_coeffs_2);
    println!("Filter order: {}", filter_2.order());
    
    // Example 3: Different number of b and a coefficients
    let b_coeffs_3 = vec![0.05, 0.1, 0.1, 0.1, 0.05];  // 5 coefficients
    let a_coeffs_3 = vec![-1.2, 0.5];                  // 2 coefficients
    
    let mut filter_3 = IIRFilter::new(b_coeffs_3, a_coeffs_3);
    println!("Filter order: {}", filter_3.order());
    
    // Process some data with one of the filters
    let input = vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let output = filter.process_block(&input);
    
    println!("Impulse response of 4th order filter:");
    for (i, sample) in output.iter().enumerate() {
        println!("y[{}] = {}", i, sample);
    }
}