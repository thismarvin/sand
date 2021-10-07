use wasm_bindgen::prelude::*;

fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
#[repr(u32)]
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
#[derive(Clone, Copy)]
pub struct Size {
    width: usize,
    height: usize,
}

#[wasm_bindgen]
pub struct World {
    size: Size,
    data: Vec<Material>,
    dirty: Vec<bool>,
}

#[wasm_bindgen]
impl World {
    pub fn with_size(width: usize, height: usize) -> Self {
        set_panic_hook();

        let size = Size { width, height };

        World {
            size,
            data: vec![Material::Air; size.width * size.height],
            dirty: vec![false; size.width * size.height],
        }
    }

    pub fn width(&self) -> usize {
        self.size.width
    }

    pub fn height(&self) -> usize {
        self.size.height
    }

    pub fn data(&self) -> *const Material {
        self.data.as_ptr()
    }

    fn get(&self, x: usize, y: usize) -> Option<&Material> {
        self.data.get(y * self.size.width + x)
    }

    pub fn place(&mut self, x: usize, y: usize, material: Material) {
        let index = y * self.size.width + x;

        if let Some(cell) = self.data.get_mut(index) {
            self.dirty[index] = true;
            *cell = material;
        }
    }

    pub fn paint(&mut self, x: usize, y: usize, radius: usize, material: Material) {
        for j in (y as isize - radius as isize)..(y as isize + radius as isize + 1) {
            if j < 0 || j > self.size.height as isize - 1 {
                continue;
            }

            for i in (x as isize - radius as isize)..(x as isize + radius as isize + 1) {
                if i < 0 || i > self.size.width as isize - 1 {
                    continue;
                }

                let distance = (((x as isize - i).pow(2) + (y as isize - j).pow(2)) as f32).sqrt();

                if distance.ceil() <= radius as f32 {
                    self.place(i as usize, j as usize, material);
                }
            }
        }
    }

    fn swap(&mut self, x1: usize, y1: usize, x2: usize, y2: usize) -> bool {
        let a = y1 * self.size.width + x1;
        let b = y2 * self.size.width + x2;

        if a > self.data.len() || b > self.data.len() {
            return false;
        }

        match (State::from(self.data[a]), State::from(self.data[b])) {
            (State::Solid, State::Liquid) | (State::Solid, State::Gas) => {
                let temp_a = self.data[a];
                let temp_b = self.data[b];

                self.data[a] = temp_b;
                self.data[b] = temp_a;

                return true;
            }
            _ => (),
        }

        if self.dirty[a] || self.dirty[b] {
            return false;
        }

        let temp_a = self.data[a];
        let temp_b = self.data[b];

        self.data[a] = temp_b;
        self.data[b] = temp_a;

        if temp_b != Material::Air {
            self.dirty[a] = true;
        }
        if temp_a != Material::Air {
            self.dirty[b] = true;
        }

        true
    }

    pub fn simulate(&mut self) {
        for flag in self.dirty.iter_mut() {
            *flag = false;
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

                let data = self.data[y * self.size.width + x];

                (|| match data {
                    Material::Sand => {
                        if let Some(material) = self.get(x, y + 1) {
                            match State::from(*material) {
                                State::Gas | State::Liquid => {
                                    if self.swap(x, y, x, y + 1) {
                                        return;
                                    }
                                }
                                _ => (),
                            }
                        }

                        let spread = 2;

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
                                            self.swap(x, y, index, y + 1)
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
                                        return;
                                    }
                                }
                                _ => (),
                            }
                        }

                        let spread = 5;

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
                                            self.swap(x, y, index, y + 1)
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
                                            self.swap(x, y, index, y)
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
                                return;
                            }
                        }

                        let spread = 2;

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
                                        Some(Material::Air) => self.swap(x, y, index, y - 1),
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
                                        Some(Material::Air) => self.swap(x, y, index, y),
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
    }
}
