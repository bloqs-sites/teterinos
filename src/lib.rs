mod utils;

use std::f32::consts::PI;
use std::fmt::Display;

use wasm_bindgen::prelude::*;

use js_sys::Error;
use js_sys::Math::{floor, random};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}

const UP: u8 = 0;
const DOWN: u8 = 1;
const LEFT: u8 = 2;
const RIGHT: u8 = 3;

#[wasm_bindgen]
pub struct Tetromino {
    lvl: u8,
    shape: Vec<bool>,
    stride: u8,
}

#[wasm_bindgen]
impl Tetromino {
    pub fn rotate_deg(&mut self, tetha: Option<f32>) {
        let mut tetha: f32 = tetha.unwrap_or(90.0);

        tetha *= PI / 180f32;

        self.rotate(tetha)
    }

    pub fn rotate_rad(&mut self, tetha: Option<f32>) {
        let tetha: f32 = tetha.unwrap_or(PI/2f32);

        self.rotate(tetha)
    }

    /*
     * Multiplying by the matrix to rotate "a" degrees:
     * | x || cos(a) -sin(a) |   | x cos(a) - y sin(a) |
     * |   ||                | = |                     |
     * | y || sin(a) cos(a)  |   | x sin(a) + y cos(a) |
     */
    fn rotate(&mut self, tetha: f32) {
        let cos = tetha.cos();
        let sin = tetha.sin();

        let n = self.stride;
        let m = self.shape.len() as u8 / n;

        let new_n = ((m as f32 * sin).abs() + (n as f32 * cos).abs()).ceil() as u8;
        let new_m = ((m as f32 * cos).abs() + (n as f32 * sin).abs()).ceil() as u8;

        let mut new_shape = vec![false; (new_n * new_m) as usize];

        for y in 0..m {
            for x in 0..n {
                let i = x + y * n;

                if self.shape[i as usize] == false {
                    continue;
                }

                let new_x = ((x + 1) as f32 * cos - (m - y) as f32 * sin + m as f32).round().abs();
                let new_y = n as f32 - ((x + 1) as f32 * sin + (m - y) as f32 * cos).round().abs();

                let j = (new_x + new_y * new_n as f32).round().abs() as f32;

                alert(&format!("[{x};{y}]\t[{new_x};{new_y}]"));

                alert(&format!("{j}"));
                new_shape[j as usize] = true;
            }
        }

        self.shape = new_shape;
        self.stride = new_n;
    }

    pub fn render(&self) -> String {
        self.to_string()
    }
}

impl Display for Tetromino {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n")?;
        for row in self.shape.as_slice().chunks(self.stride as usize) {
            write!(f, "{}:", self.lvl)?;
            for &bloq in row {
                write!(f, "{}", if bloq { "#" } else { " " })?;
            }
            write!(f, ":\n")?;
        }
        write!(f, "\n")?;

        Ok(())
    }
}

const MIN: u8 = 1;
const MAX: u8 = 8;

#[wasm_bindgen]
pub struct Game {
    stride: u8,
    field: Vec<Option<usize>>,
}

#[wasm_bindgen]
impl Game {
    pub fn new(w: u8, h: u8) -> Self {
        let f = vec![None; (w * h) as usize];

        Self {
            stride: w,
            field: f,
        }
    }

    pub fn rnd_shape(&self, lvl: u8) -> Result<Tetromino, Error> {
        if lvl < MIN || lvl > MAX {
            return Err(Error::new("invalid lvl"));
        }

        let gh = self.field.len() as u8 / self.stride;
        let mw = if lvl > self.stride { self.stride } else { lvl };
        let mh = if lvl > gh { gh } else { lvl };

        let mut area: Vec<bool> = vec![false; (mw * mh) as usize];

        let mut i = 0;

        let mut tx: u8 = 0;
        let mut ty: u8 = 0;
        let mut tw: u8 = 0;
        let mut th: u8 = 0;

        loop {
            let pos: usize = (tx + ty * mw) as usize;

            if area[pos] != true {
                area[pos] = true;
                i += 1;
            }

            if i >= lvl {
                break;
            }

            loop {
                let dir = floor(random() * 4.0) as u8;

                match dir {
                    UP => {
                        if ty == 0 {
                            continue;
                        } else {
                            ty -= 1;
                            break;
                        }
                    }
                    DOWN => {
                        if ty == (mw - 1) {
                            continue;
                        } else {
                            ty += 1;
                            if ty > th {
                                th = ty;
                            }
                            break;
                        }
                    }
                    LEFT => {
                        if tx == 0 {
                            continue;
                        } else {
                            tx -= 1;
                            break;
                        }
                    }
                    RIGHT => {
                        if tx == (mh - 1) {
                            continue;
                        } else {
                            tx += 1;
                            if tx > tw {
                                tw = tx;
                            }
                            break;
                        }
                    }
                    _ => continue,
                };
            }
        }

        let cap = (tw + 1) * (th + 1);

        let mut shape: Vec<bool> = vec![false; cap as usize];

        for j in 0..=th {
            for i in 0..=tw {
                let pos = i + j * (tw + 1);
                let apos = i + j * mw;
                shape[pos as usize] = area[apos as usize]
            }
        }

        Ok(Tetromino {
            lvl,
            shape,
            stride: tw + 1,
        })
    }

    pub fn render(&self) -> String {
        self.to_string()
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.field.as_slice().chunks(self.stride as usize) {
            for &bloq in row {
                write!(f, "{}", bloq.unwrap_or(0))?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}
