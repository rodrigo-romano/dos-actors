use tai_time::MonotonicTime;

fn main() {
    let t = MonotonicTime::now();
    println!("{:?}", t.as_secs());
    println!("{:?}", t.subsec_nanos());
}
