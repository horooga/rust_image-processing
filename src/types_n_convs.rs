use image::Rgb;

pub fn str_to_pal(hex_str: &str) -> Option<Vec<[u8; 3]>> {
    fn c_val(c: u8) -> Option<u8> {
        match c {
            b'0'..=b'9' => Some(c - b'0'),
            b'a'..=b'f' => Some(c - b'a' + 10),
            b'A'..=b'F' => Some(c - b'A' + 10),
            _ => None,
        }
    }

    fn hex_val(l: u8, h: u8) -> Option<u8> {
        let l_val = c_val(l);
        let h_val = c_val(h);
        if l_val.is_some() && h_val.is_some() {
            Some((h_val.unwrap() << 4) | l_val.unwrap())
        } else {
            None
        }
    }

    hex_str
        .as_bytes()
        .chunks_exact(6)
        .map(|hex| {
            let r_val = hex_val(hex[0], hex[1]);
            let g_val = hex_val(hex[2], hex[3]);
            let b_val = hex_val(hex[4], hex[5]);
            if r_val.is_some() && g_val.is_some() && b_val.is_some() {
                Some([r_val.unwrap(), g_val.unwrap(), b_val.unwrap()])
            } else {
                None
            }
        })
        .collect()
}

pub trait Basic: Sized {
    fn to_u8(val: Self) -> u8;
    fn to_i32(val: Self) -> i32;
    fn to_f32(val: Self) -> f32;
}

impl Basic for u8 {
    fn to_u8(val: Self) -> u8 {
        val
    }
    fn to_i32(val: Self) -> i32 {
        val as i32
    }
    fn to_f32(val: Self) -> f32 {
        val as f32
    }
}

impl Basic for i32 {
    fn to_u8(val: Self) -> u8 {
        if val > 255 {
            255
        } else if val < 0 {
            0
        } else {
            val as u8
        }
    }
    fn to_i32(val: Self) -> i32 {
        val
    }
    fn to_f32(val: Self) -> f32 {
        val as f32
    }
}

impl Basic for f32 {
    fn to_u8(val: Self) -> u8 {
        if val > 255.0 {
            255
        } else if val < 0.0 {
            0
        } else {
            val as u8
        }
    }
    fn to_i32(val: Self) -> i32 {
        val as i32
    }
    fn to_f32(val: Self) -> f32 {
        val
    }
}

pub fn i32_add<T: Basic, Y: Basic>(one: T, two: Y) -> i32 {
    T::to_i32(one) + Y::to_i32(two)
}

pub fn i32_sub<T: Basic, Y: Basic>(one: T, two: Y) -> i32 {
    return T::to_i32(one) - Y::to_i32(two);
}

pub fn i32_div<T: Basic, Y: Basic>(one: T, two: Y) -> i32 {
    return (T::to_f32(one) / Y::to_f32(two)) as i32;
}

pub fn i32_mul<T: Basic, Y: Basic>(one: T, two: Y) -> i32 {
    return (T::to_f32(one) * Y::to_f32(two)) as i32;
}

pub fn u8_add<T: Basic, Y: Basic>(one: T, two: Y) -> u8 {
    let add: i32 = T::to_i32(one) + Y::to_i32(two);
    return if add > 255 {
        255
    } else if add < 0 {
        0
    } else {
        add as u8
    };
}

pub fn u8_sub<T: Basic, Y: Basic>(one: T, two: Y) -> u8 {
    let sub: i32 = T::to_i32(one) - Y::to_i32(two);
    return if sub > 255 {
        255
    } else if sub < 0 {
        0
    } else {
        sub as u8
    };
}

pub fn u8_mul<T: Basic, Y: Basic>(one: T, two: Y) -> u8 {
    let mul: f32 = T::to_f32(one) * Y::to_f32(two);
    return if mul > 255.0 {
        255
    } else if mul < 0.0 {
        0
    } else {
        mul as u8
    };
}

pub fn u8_div<T: Basic, Y: Basic>(one: T, two: Y) -> u8 {
    let div: f32 = T::to_f32(one) / Y::to_f32(two);
    if div > 255.0 {
        255
    } else if div < 0.0 {
        0
    } else {
        div as u8
    }
}

pub fn rgb_to_arr(one: Rgb<u8>) -> [i32; 3] {
    [one[0] as i32, one[1] as i32, one[2] as i32]
}

pub fn u8_to_rgb(n: u8) -> Rgb<u8> {
    Rgb([n, n, n])
}

pub fn rgb_add(one: Rgb<u8>, two: Rgb<u8>) -> Rgb<u8> {
    Rgb([
        u8_add(one[0], two[0]),
        u8_add(one[1], two[1]),
        u8_add(one[2], two[2]),
    ])
}

pub fn rgb_sub(one: Rgb<u8>, two: Rgb<u8>) -> Rgb<u8> {
    Rgb([
        u8_sub(one[0], two[0]),
        u8_sub(one[1], two[1]),
        u8_sub(one[2], two[2]),
    ])
}

pub fn rgb_div(one: Rgb<u8>, two: f32) -> Rgb<u8> {
    Rgb([
        u8_div(one[0], two),
        u8_div(one[1], two),
        u8_div(one[2], two),
    ])
}

pub fn rgb_mul(one: Rgb<u8>, two: f32) -> Rgb<u8> {
    Rgb([
        u8_mul(one[0], two),
        u8_mul(one[1], two),
        u8_mul(one[2], two),
    ])
}
