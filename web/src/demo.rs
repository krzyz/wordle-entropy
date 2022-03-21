use hsl::HSL;
use num_complex::Complex64;
use rand::Rng;
use wasm_bindgen::{prelude::*, Clamped};
use rayon::prelude::*;
pub use wasm_bindgen_rayon::init_thread_pool;

type RGBA = [u8; 4];

struct Generator {
    width: u32,
    height: u32,
    palette: Box<[RGBA]>,
}

impl Generator {
    fn new(width: u32, height: u32, max_iterations: u32) -> Self {
        let mut rng = rand::thread_rng();

        Self {
            width,
            height,
            palette: (0..max_iterations)
                .map(move |_| {
                    let (r, g, b) = HSL {
                        h: rng.gen_range(0.0..360.0),
                        s: 0.5,
                        l: 0.6,
                    }
                    .to_rgb();
                    [r, g, b, 255]
                })
                .collect(),
        }
    }

    #[allow(clippy::many_single_char_names)]
    fn get_color(&self, x: u32, y: u32) -> &RGBA {
        let c = Complex64::new(
            (f64::from(x) - f64::from(self.width) / 2.0) * 4.0 / f64::from(self.width),
            (f64::from(y) - f64::from(self.height) / 2.0) * 4.0 / f64::from(self.height),
        );
        let mut z = Complex64::new(0.0, 0.0);
        let mut i = 0;
        while z.norm_sqr() < 4.0 {
            if i == self.palette.len() {
                return &self.palette[0];
            }
            z = z.powi(2) + c;
            i += 1;
        }
        &self.palette[i]
    }

    fn iter_row_bytes(&self, y: u32) -> impl '_ + Iterator<Item = u8> {
        (0..self.width)
            .flat_map(move |x| self.get_color(x, y))
            .copied()
    }

    fn iter_bytes(&self) -> impl '_ + ParallelIterator<Item = u8> {
        (0..self.height)
            .into_par_iter()
            .flat_map_iter(move |y| self.iter_row_bytes(y))
    }

    /*
    fn iter_bytes(&self) -> impl '_ + Iterator<Item = u8> {
        (0..self.height).flat_map(move |y| self.iter_row_bytes(y))
    }
    */
}

#[wasm_bindgen]
pub fn generate(width: u32, height: u32, max_iterations: u32) -> Vec<u8> {
    Generator::new(width, height, max_iterations)
        .iter_bytes()
        .collect()
}