mod utils;

use std::f32::consts::PI;
use std::fmt::Display;

extern crate fixedbitset;
use fixedbitset::FixedBitSet;
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

struct Pos {
    x: u8,
    y: u8,
}

#[wasm_bindgen]
pub struct Tetromino {
    lvl: u8,
    shape: FixedBitSet,
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
        let tetha: f32 = tetha.unwrap_or(PI / 2f32);

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

        let new_n = ((m as f32 * sin).abs() + (n as f32 * cos).abs()).round() as u8;
        let new_m = ((m as f32 * cos).abs() + (n as f32 * sin).abs()).round() as u8;

        let mut new_shape = FixedBitSet::with_capacity((new_n * new_m) as usize);

        for y in 0..m {
            for x in 0..n {
                let i = x + y * n;

                if self.shape[i as usize] == false {
                    continue;
                }

                let new_x = ((x + 1) as f32 * cos - (m - y) as f32 * sin + m as f32).round();
                let new_y = n as f32 - ((x + 1) as f32 * sin + (m - y) as f32 * cos).round();

                let j = (new_x + new_y * new_n as f32).round().abs() as f32;

                //alert(&format!("[{x};{y}]\t[{new_x};{new_y}]"));

                //alert(&format!("{j}"));
                new_shape.set(j as usize, true);
            }
        }

        self.shape = new_shape;
        self.stride = new_n;
    }

    pub fn width(&self) -> u8 {
        self.stride
    }

    pub fn height(&self) -> u8 {
        self.shape.len() as u8 / self.stride
    }

    pub fn test_pos(&self, pos: usize) -> bool {
        self.shape[pos]
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
                write!(f, "{bloq:b}\n")?;
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
    id: char,
    stride: u8,
    field: Vec<Option<char>>,
}

#[wasm_bindgen]
impl Game {
    pub fn new(w: u8, h: u8) -> Self {
        let f = vec![None; (w * h) as usize];

        Self {
            id: 'A',
            stride: w,
            field: f,
        }
    }

    pub fn put_tetrino(&mut self, lvl: Option<u8>) -> Result<bool, Error> {
        let lvl = lvl.unwrap_or(rnd());

        let mut shape = self.rnd_tetrino(lvl)?;

        for _ in 0..floor(random() * 4.0) as u8 {
            shape.rotate_deg(Some(90.0));
        }

        alert(&format!(
            "i={}\tj={}\tk={}\nl={}",
            self.height() - shape.height(),
            self.width() - shape.width(),
            shape.height(),
            shape.width()
        ));
        alert(&format!(
            "i={}\tj={}\tk={}\nl={}",
            self.height(),
            self.width(),
            shape.height(),
            shape.width()
        ));
        alert(&shape.render());

        for _ in 0..4 {
            for i in 0..self.height()
            /* - shape.height()*/
            {
                'next: for j in 0..self.width()
                /* - shape.width()*/
                {
                    for k in 0..shape.height() {
                        for l in 0..shape.width() {
                            let pos: usize = (j + l + (i + k) * self.width()) as usize;

                            if self.field.len() <= pos
                                || (self.field[pos].is_some()
                                    && shape.test_pos((l + k * shape.width()) as usize))
                            {
                                continue 'next;
                            }

                            if k == shape.height() - 1 && l == shape.width() - 1 {
                                self.put_shape(Pos { x: j, y: i }, shape);

                                return Ok(true);
                            }
                        }
                    }
                }
            }

            shape.rotate_deg(Some(90.0));
        }

        Err(Error::new(&format!("could not insert tetrino lvl {lvl}")))
    }

    fn put_shape(&mut self, coords: Pos, shape: Tetromino) {
        for i in 0..shape.height() {
            for j in 0..shape.width() {
                let pos = j + i * shape.width();

                if shape.test_pos(pos as usize) {
                    let pos = j + coords.x + (i + coords.y) * self.width();
                    self.field[pos as usize] = Some(self.id);
                }
            }
        }

        self.id = std::char::from_u32(self.id as u32 + 1).unwrap_or(self.id);
    }

    pub fn rnd_tetrino(&self, lvl: u8) -> Result<Tetromino, Error> {
        if lvl < MIN || lvl > MAX {
            return Err(Error::new("invalid lvl"));
        }

        let gh = self.height();
        let mw = if lvl > self.stride { self.stride } else { lvl };
        let mh = if lvl > gh { gh } else { lvl };

        let mut area = FixedBitSet::with_capacity((mw * mh) as usize);

        let mut i = 0;

        let mut tx: u8 = 0;
        let mut ty: u8 = 0;
        let mut tw: u8 = 0;
        let mut th: u8 = 0;

        loop {
            let pos: usize = (tx + ty * mw) as usize;

            if area[pos] != true {
                area.set(pos, true);
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

        let mut shape = FixedBitSet::with_capacity(cap as usize);

        for j in 0..=th {
            for i in 0..=tw {
                let pos = i + j * (tw + 1);
                let apos = i + j * mw;
                shape.set(pos as usize, area[apos as usize]);
            }
        }

        Ok(Tetromino {
            lvl,
            shape,
            stride: tw + 1,
        })
    }

    pub fn width(&self) -> u8 {
        self.stride
    }

    pub fn height(&self) -> u8 {
        self.field.len() as u8 / self.stride
    }

    pub fn render(&self) -> String {
        self.to_string()
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.field.as_slice().chunks(self.stride as usize) {
            for &bloq in row {
                write!(f, "{}", bloq.unwrap_or(' '))?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

fn rnd() -> u8 {
    floor(random() * (MAX - MIN - 1) as f64 + 1.0) as u8
}
