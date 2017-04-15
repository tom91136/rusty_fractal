extern crate image;
extern crate num;
extern crate time;
extern crate rayon;
extern crate arrayvec;

use std::fs::File;
use std::path::Path;

use num::complex::Complex;
use time::PreciseTime;
use rayon::prelude::*;


#[derive(Debug)]
struct Config {
    size: u32,
    iteration: u32,
    zoom: f64,
    poi_x: f64,
    poi_y: f64,
}

struct Step(Complex<f64>, u32);

struct Bound<T>(T, T);

#[inline]
fn iterate(c: Complex<f64>, z: Complex<f64>, iter: u32, limit: u32) -> Step {

    if iter > limit {
        return Step(Complex::new(0.0, 0.0), 0);
    }

    let result: Complex<f64> = z.powf(2.0) + c;
    if z.norm_sqr().sqrt() > 2.0 {
        return Step(result, iter);
    } else {
        return iterate(c, result, iter + 1, limit);
    }
}


#[inline]
fn linear_transform(input: f64,
                    input_min: f64,
                    input_max: f64,
                    output_min: f64,
                    output_max: f64)
                    -> f64 {
    ((output_max - output_min) * (input - input_min) / (input_max - input_min)) + output_min
}

#[inline]
fn colourize(limit: u32, step: Step) -> f64 {
    step.1 as f64 / limit as f64
}

#[inline]
fn compute_discrete(iteration: u32,
                    size: u32,
                    x_bound: &Bound<f64>,
                    y_bound: &Bound<f64>,
                    x: u32,
                    y: u32)
                    -> u8 {
    let scaled_x = linear_transform(x as f64, 0f64, size as f64, x_bound.0, x_bound.1);
    let scaled_y = linear_transform(y as f64, 0f64, size as f64, y_bound.0, y_bound.1);
    let colour = colourize(iteration,
                           iterate(Complex::new(scaled_x, scaled_y),
                                   Complex::new(0.0, 0.0),
                                   0,
                                   iteration));
    (colour * 255.0) as u8
}

fn main() {

    let c = Config {
        size: 2048,
        iteration: 10000,
        zoom: 10000.0,
        poi_x: 0.28693186889504513,
        poi_y: 0.014286693904085048,
    };

    println!("Starting...");


    // let ys: Vec<u32> = (0..c.size).collect();
    // let xs: Vec<u32> = (0..c.size).collect();

    println!("Range created, starting computation...");
    let start = PreciseTime::now();

    let scale = 1.0 / c.zoom;
    let x_bound = Bound(c.poi_x - scale, c.poi_x + scale);
    let y_bound = Bound(c.poi_y - scale, c.poi_y + scale);

    let xys: Vec<Vec<u8>> = (0..c.size)
        .into_par_iter()
        .map(|y| {
                 (0..c.size)
                //  .into_par_iter()
                     .map(|x| compute_discrete(c.iteration, c.size, &x_bound, &y_bound, x, y))
                     .collect()
             })
        .collect();


    let end = PreciseTime::now();

    println!("Fractal computed taking {}s writing to disk...{}",
             start.to(end),
             xys.len());

    let mut imgbuf = image::ImageBuffer::new(c.size, c.size);

    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        *pixel = image::Luma([xys[x as usize][y as usize]])
    }

    let ref mut fout = File::create(&Path::new("fractal.png")).unwrap();

    // We must indicate the image’s color type and what format to save as
    let _ = image::ImageLuma8(imgbuf).save(fout, image::PNG);
    println!("Done.");

}
