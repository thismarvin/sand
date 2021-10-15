use wasm_bindgen::prelude::*;

// #[wasm_bindgen]
// extern "C" {
//     #[wasm_bindgen(js_namespace = console)]
//     fn log(s: &str);
// }
//
// macro_rules! console_log {
//     ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
// }

fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

fn distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    (((x2 - x1).powi(2) + (y2 - y1).powi(2)) as f32).sqrt()
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Material {
    Air = 0,
    Rock = 1,
    Sand = 2,
    Water = 3,
    Smoke = 4,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Solid,
    Liquid,
    Gas,
}

impl From<Material> for State {
    fn from(material: Material) -> Self {
        match material {
            Material::Rock => State::Solid,
            Material::Sand => State::Solid,
            Material::Water => State::Liquid,
            Material::Smoke => State::Gas,
            Material::Air => State::Gas,
        }
    }
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Tint {
    None = 0,
    Dark = 1,
    Darker = 2,
    Darkest = 3,
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

#[wasm_bindgen]
pub struct World {
    size: Size,
    chunk_size: usize,
    chunk_columns: usize,
    hot: bool,
    active_chunks: Vec<bool>,
    forecast: Vec<bool>,
    materials: Vec<Material>,
    dirty: Vec<bool>,
    tints: Vec<Tint>,
    spreads: Vec<u8>,
}

#[wasm_bindgen]
impl World {
    pub fn create(width: usize, height: usize, chunk_size: usize) -> Self {
        set_panic_hook();

        let size = Size { width, height };

        let columns = (width as f32 / chunk_size as f32).ceil() as usize;
        let rows = (height as f32 / chunk_size as f32).ceil() as usize;

        World {
            size,
            chunk_size,
            chunk_columns: columns,
            hot: false,
            active_chunks: vec![false; columns * rows],
            forecast: vec![false; columns * rows],
            materials: vec![Material::Air; size.width * size.height],
            tints: vec![Tint::None; size.width * size.height],
            spreads: vec![0; size.width * size.height],
            dirty: vec![false; size.width * size.height],
        }
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn materials(&self) -> *const Material {
        self.materials.as_ptr()
    }

    pub fn tints(&self) -> *const Tint {
        self.tints.as_ptr()
    }

    fn get(&self, x: usize, y: usize) -> Option<&Material> {
        self.materials.get(y * self.size.width + x)
    }

    fn get_chunk_index(&self, x: usize, y: usize) -> Option<usize> {
        let x = x / self.chunk_size;
        let y = y / self.chunk_size;

        let index = y * self.chunk_columns + x;

        match self.active_chunks.get(index) {
            Some(_) => Some(index),
            None => None,
        }
    }

    pub fn reset(&mut self) {
        for i in 0..self.materials.len() {
            self.materials[i] = Material::Air;
            self.tints[i] = Tint::None;
            self.spreads[i] = 0;
        }

        self.hot = false;

        for i in 0..self.active_chunks.len() {
            self.active_chunks[i] = false;
            self.forecast[i] = false;
        }
    }

    pub fn place(&mut self, x: usize, y: usize, material: Material, tint: Tint, spread: u8) {
        let index = y * self.size.width + x;

        if index >= self.materials.len() {
            return;
        }

        self.materials[index] = material;
        self.tints[index] = tint;
        self.spreads[index] = spread;

        self.dirty[index] = true;

        self.warm_up(x, y);

        // Instead of queuing chunks to be active, immediately set the chunks as active.
        for (i, entry) in self.forecast.iter_mut().enumerate() {
            if *entry {
                self.active_chunks[i] = true;

                *entry = false;
            }
        }

        self.hot = true;
    }

    fn warm_up(&mut self, x: usize, y: usize) {
        let x = x / self.chunk_size;
        let y = y / self.chunk_size;

        let base = y * self.chunk_columns + x;

        let index = base;

        if let Some(target) = self.forecast.get_mut(index) {
            *target = true;
        }

        if x > 0 {
            let index = base - 1;

            if let Some(target) = self.forecast.get_mut(index) {
                *target = true;
            }
        }

        if x < self.chunk_columns - 1 {
            let index = base + 1;

            if let Some(target) = self.forecast.get_mut(index) {
                *target = true;
            }
        }

        if y > 0 {
            let index = base - self.chunk_columns;

            if let Some(target) = self.forecast.get_mut(index) {
                *target = true;
            }

            if x > 0 {
                let index = base - self.chunk_columns - 1;

                if let Some(target) = self.forecast.get_mut(index) {
                    *target = true;
                }
            }

            if x < self.chunk_columns - 1 {
                let index = base - self.chunk_columns + 1;

                if let Some(target) = self.forecast.get_mut(index) {
                    *target = true;
                }
            }
        }

        if y < self.chunk_columns - 1 {
            let index = base + self.chunk_columns;

            if let Some(target) = self.forecast.get_mut(index) {
                *target = true;
            }

            if x > 0 {
                let index = base + self.chunk_columns - 1;

                if let Some(target) = self.forecast.get_mut(index) {
                    *target = true;
                }
            }

            if x < self.chunk_columns - 1 {
                let index = base + self.chunk_columns + 1;

                if let Some(target) = self.forecast.get_mut(index) {
                    *target = true;
                }
            }
        }
    }

    pub fn paint(
        &mut self,
        x1: usize,
        y1: usize,
        x2: usize,
        y2: usize,
        radius: usize,
        material: Material,
        tint: Tint,
        spread: u8,
    ) {
        let x1 = x1 as isize;
        let y1 = y1 as isize;
        let x2 = x2 as isize;
        let y2 = y2 as isize;
        let radius = radius as isize;

        let dx = x2 - x1;
        let dy = y2 - y1;

        const LEEWAY: isize = 1;

        if -LEEWAY <= dx && dx <= LEEWAY {
            let range = dy.abs() as usize;
            let range = range.max(1);
            let range = range.min(self.size.height);

            let mut x = x1;
            let mut y = y1;

            if y2 < y1 {
                x = x2;
                y = y2;
            }

            for i in 0..range {
                let y = y + i as isize;

                for j in (y - radius)..(y + radius + 1) {
                    if j < 0 || j > self.size.height as isize - 1 {
                        continue;
                    }

                    for i in (x - radius)..(x + radius + 1) {
                        if i < 0 || i > self.size.width as isize - 1 {
                            continue;
                        }

                        let distance = distance(x as f32, y as f32, i as f32, j as f32).ceil();

                        if distance <= radius as f32 {
                            self.place(i as usize, j as usize, material, tint, spread);
                        }
                    }
                }
            }

            return;
        }

        let slope = dy as f32 / dx as f32;
        let y_intercept = y1 as f32 - slope * x1 as f32;

        let domain = dx.abs() as usize;
        let domain = domain.max(1);
        let domain = domain.min(self.size.width);

        let leftmost = x1.min(x2) as f32;

        const STEP: f32 = 0.5;
        let domain = (domain as f32 / STEP).ceil() as usize;

        for i in 0..domain {
            let x = leftmost + i as f32 * STEP;
            let y = (((slope * x).ceil()) + y_intercept) as isize;
            let x = x as isize;

            for j in (y - radius)..(y + radius + 1) {
                if j < 0 || j > self.size.height as isize - 1 {
                    continue;
                }

                for i in (x - radius)..(x + radius + 1) {
                    if i < 0 || i > self.size.width as isize - 1 {
                        continue;
                    }

                    let distance = distance(x as f32, y as f32, i as f32, j as f32).ceil();

                    if distance <= radius as f32 {
                        self.place(i as usize, j as usize, material, tint, spread);
                    }
                }
            }
        }
    }

    fn swap(&mut self, x1: usize, y1: usize, x2: usize, y2: usize) -> bool {
        let a = y1 * self.size.width + x1;
        let b = y2 * self.size.width + x2;

        if a > self.materials.len() || b > self.materials.len() {
            return false;
        }

        match (
            State::from(self.materials[a]),
            State::from(self.materials[b]),
        ) {
            (State::Solid, State::Liquid)
            | (State::Solid, State::Gas)
            | (State::Liquid, State::Gas) => {
                let temp_a = self.materials[a];
                let temp_b = self.materials[b];

                self.materials[a] = temp_b;
                self.materials[b] = temp_a;

                let temp_a = self.tints[a];
                let temp_b = self.tints[b];

                self.tints[a] = temp_b;
                self.tints[b] = temp_a;

                let temp_a = self.spreads[a];
                let temp_b = self.spreads[b];

                self.spreads[a] = temp_b;
                self.spreads[b] = temp_a;

                return true;
            }
            _ => (),
        }

        if self.dirty[a] || self.dirty[b] {
            return false;
        }

        let temp_a = self.materials[a];
        let temp_b = self.materials[b];

        self.materials[a] = temp_b;
        self.materials[b] = temp_a;

        if temp_b != Material::Air {
            self.dirty[a] = true;
        }
        if temp_a != Material::Air {
            self.dirty[b] = true;
        }

        let temp_a = self.tints[a];
        let temp_b = self.tints[b];

        self.tints[a] = temp_b;
        self.tints[b] = temp_a;

        let temp_a = self.spreads[a];
        let temp_b = self.spreads[b];

        self.spreads[a] = temp_b;
        self.spreads[b] = temp_a;

        true
    }

    pub fn simulate(&mut self) {
        if !self.hot {
            return;
        }

        for entry in self.dirty.iter_mut() {
            *entry = false;
        }

        for y in (0..self.size.height).rev() {
            let preference: isize = if y % 2 == 0 { 1 } else { -1 };

            for x in 0..self.size.width {
                let x = if preference < 0 {
                    self.size.width - 1 - x
                } else {
                    x
                };

                if self.dirty[y * self.size.width + x] {
                    continue;
                }

                if let Some(index) = self.get_chunk_index(x, y) {
                    if let Some(chunk) = self.active_chunks.get(index) {
                        if !chunk {
                            continue;
                        }
                    }
                } else {
                    // I do not think this will ever be reached, but you can never be too safe!
                    continue;
                }

                let material = self.materials[y * self.size.width + x];

                (|| match material {
                    Material::Sand => {
                        if let Some(material) = self.get(x, y + 1) {
                            match State::from(*material) {
                                State::Gas | State::Liquid => {
                                    if self.swap(x, y, x, y + 1) {
                                        self.warm_up(x, y + 1);

                                        return;
                                    }
                                }
                                _ => (),
                            }
                        }

                        let spread = self.spreads[y * self.size.width + x];

                        let mut left_blocked = false;
                        let mut right_blocked = false;

                        let mut dir = -preference;

                        for i in 1..(spread + 1) {
                            for _ in 0..2 {
                                dir = -dir;

                                let swapped = (|| {
                                    if (dir < 0 && left_blocked) || (dir > 0 && right_blocked) {
                                        return false;
                                    }

                                    let index = (x as isize) + (i as isize) * dir;

                                    if index < 0 || index >= self.size.width as isize {
                                        return false;
                                    }

                                    let index = index as usize;

                                    let blocked = match self.get(index, y) {
                                        Some(Material::Sand) => false,
                                        Some(material)
                                            if matches!(State::from(*material), State::Gas) =>
                                        {
                                            false
                                        }
                                        _ => true,
                                    };

                                    let mut update_blockade = || {
                                        if dir < 0 {
                                            left_blocked = true;
                                        } else {
                                            right_blocked = true;
                                        }
                                    };

                                    if blocked {
                                        update_blockade();

                                        return false;
                                    }

                                    match self.get(index, y + 1) {
                                        Some(Material::Sand) => false,
                                        Some(material)
                                            if matches!(State::from(*material), State::Gas) =>
                                        {
                                            if self.swap(x, y, index, y + 1) {
                                                self.warm_up(x, y + 1);

                                                return true;
                                            }

                                            false
                                        }
                                        _ => {
                                            update_blockade();

                                            false
                                        }
                                    }
                                })();

                                if swapped {
                                    return;
                                }
                            }

                            if left_blocked && right_blocked {
                                break;
                            }
                        }
                    }

                    Material::Water => {
                        if let Some(material) = self.get(x, y + 1) {
                            match State::from(*material) {
                                State::Gas => {
                                    if self.swap(x, y, x, y + 1) {
                                        self.warm_up(x, y + 1);

                                        return;
                                    }
                                }
                                _ => (),
                            }
                        }

                        let spread = self.spreads[y * self.size.width + x];

                        let mut dir = -preference;
                        let mut left_blocked = false;
                        let mut right_blocked = false;

                        for i in 1..(spread + 1) {
                            for _ in 0..2 {
                                dir = -dir;

                                let swapped = (|| {
                                    if (dir < 0 && left_blocked) || (dir > 0 && right_blocked) {
                                        return false;
                                    }

                                    let index = (x as isize) + (i as isize) * dir;

                                    if index < 0 || index >= self.size.width as isize {
                                        return false;
                                    }

                                    let index = index as usize;

                                    let blocked = match self.get(index, y) {
                                        Some(Material::Water) => false,
                                        Some(material)
                                            if matches!(State::from(*material), State::Gas) =>
                                        {
                                            false
                                        }
                                        _ => true,
                                    };

                                    let mut update_blockade = || {
                                        if dir < 0 {
                                            left_blocked = true;
                                        } else {
                                            right_blocked = true;
                                        }
                                    };

                                    if blocked {
                                        update_blockade();

                                        return false;
                                    }

                                    match self.get(index, y + 1) {
                                        Some(Material::Water) => false,
                                        Some(material)
                                            if matches!(State::from(*material), State::Gas) =>
                                        {
                                            if self.swap(x, y, index, y + 1) {
                                                self.warm_up(x, y + 1);

                                                return true;
                                            }

                                            false
                                        }
                                        _ => {
                                            update_blockade();

                                            false
                                        }
                                    }
                                })();

                                if swapped {
                                    return;
                                }
                            }

                            if left_blocked && right_blocked {
                                break;
                            }
                        }

                        let mut dir = -preference;
                        let mut left_blocked = false;
                        let mut right_blocked = false;

                        for i in 1..(spread + 1) {
                            for _ in 0..2 {
                                dir = -dir;

                                let swapped = (|| {
                                    if (dir < 0 && left_blocked) || (dir > 0 && right_blocked) {
                                        return false;
                                    }

                                    let index = (x as isize) + (i as isize) * dir;

                                    if index < 0 || index >= self.size.width as isize {
                                        return false;
                                    }

                                    let index = index as usize;

                                    let mut update_blockade = || {
                                        if dir < 0 {
                                            left_blocked = true;
                                        } else {
                                            right_blocked = true;
                                        }
                                    };

                                    match self.get(index, y) {
                                        Some(Material::Water) => false,
                                        Some(material)
                                            if matches!(State::from(*material), State::Gas) =>
                                        {
                                            if self.swap(x, y, index, y) {
                                                self.warm_up(x, y);

                                                return true;
                                            }

                                            false
                                        }
                                        _ => {
                                            update_blockade();

                                            false
                                        }
                                    }
                                })();

                                if swapped {
                                    return;
                                }
                            }

                            if left_blocked && right_blocked {
                                break;
                            }
                        }
                    }

                    Material::Smoke => {
                        if let Some(Material::Air) = self.get(x, y - 1) {
                            if self.swap(x, y, x, y - 1) {
                                self.warm_up(x, y - 1);
                                return;
                            }
                        }

                        let spread = self.spreads[y * self.size.width + x];

                        let mut dir = -preference;
                        let mut left_blocked = false;
                        let mut right_blocked = false;

                        for i in 1..(spread + 1) {
                            for _ in 0..2 {
                                dir = -dir;

                                let swapped = (|| {
                                    if (dir < 0 && left_blocked) || (dir > 0 && right_blocked) {
                                        return false;
                                    }

                                    let index = (x as isize) + (i as isize) * dir;

                                    if index < 0 || index >= self.size.width as isize {
                                        return false;
                                    }

                                    let index = index as usize;

                                    let blocked = match self.get(index, y) {
                                        Some(Material::Smoke | Material::Air) => false,
                                        _ => true,
                                    };

                                    let mut update_blockade = || {
                                        if dir < 0 {
                                            left_blocked = true;
                                        } else {
                                            right_blocked = true;
                                        }
                                    };

                                    if blocked {
                                        update_blockade();

                                        return false;
                                    }

                                    match self.get(index, y - 1) {
                                        Some(Material::Smoke) => false,
                                        Some(Material::Air) => {
                                            if self.swap(x, y, index, y - 1) {
                                                self.warm_up(x, y - 1);

                                                return true;
                                            }

                                            false
                                        }
                                        _ => {
                                            update_blockade();

                                            false
                                        }
                                    }
                                })();

                                if swapped {
                                    return;
                                }
                            }

                            if left_blocked && right_blocked {
                                break;
                            }
                        }

                        let mut dir = -preference;
                        let mut left_blocked = false;
                        let mut right_blocked = false;

                        for i in 1..(spread + 1) {
                            for _ in 0..2 {
                                dir = -dir;

                                let swapped = (|| {
                                    if (dir < 0 && left_blocked) || (dir > 0 && right_blocked) {
                                        return false;
                                    }

                                    let index = (x as isize) + (i as isize) * dir;

                                    if index < 0 || index >= self.size.width as isize {
                                        return false;
                                    }

                                    let index = index as usize;

                                    match self.get(index, y) {
                                        Some(Material::Smoke) => false,
                                        Some(Material::Air) => {
                                            if self.swap(x, y, index, y) {
                                                self.warm_up(x, y);

                                                return true;
                                            }

                                            false
                                        }
                                        _ => {
                                            if dir < 0 {
                                                left_blocked = true;
                                            } else {
                                                right_blocked = true;
                                            }

                                            false
                                        }
                                    }
                                })();

                                if swapped {
                                    return;
                                }
                            }

                            if left_blocked && right_blocked {
                                break;
                            }
                        }
                    }
                    _ => (),
                })();
            }
        }

        self.hot = false;

        for (i, entry) in self.forecast.iter_mut().enumerate() {
            if *entry {
                self.hot = true;
            }

            self.active_chunks[i] = *entry;

            *entry = false;
        }
    }
}
