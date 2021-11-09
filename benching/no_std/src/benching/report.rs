use alloc::format;
use defmt::info;
use embedded_time::TimeInt;

use super::{runner::Samples, Summary};

pub fn report<T: TimeInt + Into<u32>>(s: &Samples<T>) {
    // let mut total = 0f64;
    // for i in 0..s.data.len() {
    //     total += interpreter.as_micros(s.data[i]);
    // }
    // let n = s.data.len() as f64;
    // let avg = total / n;
    let data = s
        .data
        .iter()
        .map(|d| {
            let d: u32 = d.0.into();
            d as f64
        })
        .collect::<heapless::Vec<f64, 100>>();

    // info!("{:?}", data.as_slice());

    let summary = Summary::new::<100>(&data.into_array().unwrap());

    let median = (summary.median) as usize;
    let deviation = (summary.max - summary.min) as usize;

    info!(
        "function: {}    {} ns/iter (+/- {})",
        s.name,
        fmt_thousands_sep(median, '_').as_str(),
        fmt_thousands_sep(deviation, '_').as_str()
    );
}

// Format a number with thousands separators
fn fmt_thousands_sep(mut n: usize, sep: char) -> alloc::string::String {
    let mut output = alloc::string::String::new();
    let mut trailing = false;
    for &pow in &[9, 6, 3, 0] {
        let base = 10_usize.pow(pow);
        if pow == 0 || trailing || n / base != 0 {
            if !trailing {
                output.extend(format!("{}", n / base).as_str().chars());
            } else {
                output.extend(format!("{:03}", n / base).as_str().chars());
            }
            if pow != 0 {
                output.push(sep);
            }
            trailing = true;
        }
        n %= base;
    }

    output
}
