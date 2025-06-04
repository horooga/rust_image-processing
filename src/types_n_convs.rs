use image::Rgb;

#[derive(Copy, Clone, Debug)]
pub struct Intersection {
    pub t: f32,
    pub n: Xyz,
    pub hit: Xyz,
    pub hit_obj: Object,
}

pub fn area(x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) -> f32 {
    ((x1 * (y2 - y3) + x2 * (y3 - y1) + x3 * (y1 - y2)) / 2.0).abs()
}

impl Intersection {
    pub fn new(t: f32, n: Xyz, hit: Xyz, hit_obj: Object) -> Self {
        Self { t, n, hit, hit_obj }
    }
    pub fn null() -> Self {
        Self::new(-1.0, Xyz::null(), Xyz::null(), Object::null())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Object {
    pub obj_type: i8,
    pub pos: Xyz,
    pub col: Rgb<u8>,
    pub a: f32,
    pub b: Xyz,
    pub c: Xyz,
    pub d: Xyz,
    pub rfty: f32,
    pub light: bool,
}

impl Object {
    pub fn new(
        obj_type: i8,
        pos: Xyz,
        col: Rgb<u8>,
        a: f32,
        b: Xyz,
        c: Xyz,
        d: Xyz,
        rfty: f32,
        light: bool,
    ) -> Self {
        Self {
            obj_type,
            pos,
            col,
            a,
            b,
            c,
            d,
            rfty,
            light,
        }
    }
    pub fn null() -> Self {
        Self::new(
            -1,
            Xyz::null(),
            Rgb([0, 0, 0]),
            0.0,
            Xyz::null(),
            Xyz::null(),
            Xyz::null(),
            0.0,
            false,
        )
    }
    pub fn new_plane(pos: Xyz, col: Rgb<u8>, a: f32, rfty: f32, light: bool) -> Self {
        Self::new(
            0,
            pos,
            col,
            a,
            Xyz::null(),
            Xyz::null(),
            Xyz::null(),
            rfty,
            light,
        )
    }
    pub fn new_sphere(pos: Xyz, col: Rgb<u8>, a: f32, rfty: f32, light: bool) -> Self {
        Self::new(
            1,
            pos,
            col,
            a,
            Xyz::null(),
            Xyz::null(),
            Xyz::null(),
            rfty,
            light,
        )
    }
    pub fn new_triangle(
        pos: Xyz,
        col: Rgb<u8>,
        b: Xyz,
        c: Xyz,
        d: Xyz,
        rfty: f32,
        light: bool,
    ) -> Self {
        Self::new(2, pos, col, 0.0, b, c, d, rfty, light)
    }
    pub fn check(self: Self, ro: Xyz, rd: Xyz) -> Intersection {
        match self.obj_type {
            0 => {
                let p: f32 = self.a;
                let n: Xyz = self.pos;
                let t: f32 = if rd.dot(n) != 0.0 {
                    -(ro.dot(n) - p) / rd.dot(n)
                } else {
                    -1.0
                };
                if t != -1.0 && t > 0.0 {
                    Intersection::new(t, n, Xyz::xyz_add(ro, Xyz::mul(rd, t)), self)
                } else {
                    Intersection::null()
                }
            }
            1 => {
                let ro: Xyz = ro.xyz_sub(self.pos);
                let s: f32 = self.a;
                let e_c: f32 = ro.dot(ro) - s * s;
                let e_b: f32 = ro.dot(rd) * 2.0;
                let e_a: f32 = rd.dot(rd);
                let d: f32 = e_b.powi(2) - 4.0 * e_a * e_c;
                if d >= 0.0 {
                    let x: f32 =
                        ((-e_b - d.sqrt()) / (e_a * 2.0)).max((-e_b - d.sqrt()) / (e_a * 2.0));
                    if x < 0.0 {
                        return Intersection::null();
                    }
                    let n: Xyz = ro.xyz_add(rd.mul(x)).norm();
                    let hit: Xyz = ro.xyz_add(rd.mul(x));
                    Intersection::new(x, n, hit, self)
                } else {
                    Intersection::null()
                }
            }
            2 => {
                let v0: Xyz = self.b;
                let v1: Xyz = self.c;
                let v2: Xyz = self.d;
                let e1: Xyz = v1.xyz_sub(v0);
                let e2: Xyz = v2.xyz_sub(v0);
                let pvec: Xyz = rd.cross(e2);
                let det: f32 = e1.dot(pvec);
                if det > -f32::EPSILON && det < f32::EPSILON {
                    return Intersection::null();
                }
                let inv_det: f32 = 1.0 / det;
                let tvec: Xyz = ro.xyz_sub(v0);
                let u: f32 = inv_det * tvec.dot(pvec);
                if !(0.0..=1.0).contains(&u) {
                    return Intersection::null();
                }
                let qvec: Xyz = tvec.cross(e1);
                let v: f32 = inv_det * rd.dot(qvec);
                if v < 0.0 || u + v > 1.0 {
                    return Intersection::null();
                }
                let t: f32 = inv_det * e2.dot(qvec);
                if t > f32::EPSILON {
                    Intersection::new(t, self.pos, ro.xyz_add(rd.mul(t)), self)
                } else {
                    Intersection::null()
                }
            }
            _ => Intersection::null(),
        }
    }
}

impl PartialEq for Object {
    fn eq(self: &Self, other: &Self) -> bool {
        self.pos == other.pos && self.obj_type != other.obj_type
    }
    fn ne(self: &Self, other: &Self) -> bool {
        self.pos != other.pos && self.obj_type != other.obj_type
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Xyz {
    pub val: [f32; 3],
}

impl Xyz {
    pub fn new(val: [f32; 3]) -> Self {
        Self { val }
    }
    pub fn null() -> Self {
        Self::new([0.0, 0.0, 0.0])
    }
    pub fn to_xyz(arr: [f32; 3]) -> Xyz {
        Xyz::new([arr[0], arr[1], arr[2]])
    }
    pub fn length(self: Self) -> f32 {
        (self.val[0].powi(2) + self.val[1].powi(2) + self.val[2].powi(2)).sqrt()
    }
    pub fn norm(self: Self) -> Self {
        let l: f32 = self.length();
        if l == 0.0 {
            Self::new([0.0, 0.0, 0.0])
        } else {
            self.div(l)
        }
    }
    pub fn div(self: Self, dr: f32) -> Self {
        let pval: [f32; 3] = self.val;
        Self::new([pval[0] / dr, pval[1] / dr, pval[2] / dr])
    }
    pub fn mul(self: Self, mr: f32) -> Self {
        let pval: [f32; 3] = self.val;
        Self::new([pval[0] * mr, pval[1] * mr, pval[2] * mr])
    }
    pub fn add(self: Self, ad: f32) -> Self {
        let pval: [f32; 3] = self.val;
        Self::new([pval[0] + ad, pval[1] + ad, pval[2] + ad])
    }
    pub fn sub(self: Self, sb: f32) -> Self {
        let pval: [f32; 3] = self.val;
        Self::new([pval[0] - sb, pval[1] - sb, pval[2] - sb])
    }
    pub fn xyz_add(self: Self, other: Self) -> Self {
        let pval: [f32; 3] = self.val;
        let sval: [f32; 3] = other.val;
        Self::new([pval[0] + sval[0], pval[1] + sval[1], pval[2] + sval[2]])
    }
    pub fn xyz_sub(self: Self, other: Self) -> Self {
        let pval: [f32; 3] = self.val;
        let sval: [f32; 3] = other.val;
        Self::new([pval[0] - sval[0], pval[1] - sval[1], pval[2] - sval[2]])
    }
    pub fn xyz_mul(self: Self, other: Self) -> Self {
        let pval: [f32; 3] = self.val;
        let sval: [f32; 3] = other.val;
        Self::new([pval[0] * sval[0], pval[1] * sval[0], pval[2] * sval[2]])
    }
    pub fn reflect(mut self: Self, n: Self) -> Self {
        let pval: [f32; 3] = self.val;
        let sval: [f32; 3] = n.val;
        Xyz::new([
            pval[0] - sval[0] * 2.0 * self.dot(n),
            pval[1] - sval[1] * 2.0 * self.dot(n),
            pval[2] - sval[2] * 2.0 * self.dot(n),
        ])
    }
    pub fn dot(self: Self, other: Self) -> f32 {
        let pval: [f32; 3] = self.val;
        let sval: [f32; 3] = other.val;
        pval[0] * sval[0] + pval[1] * sval[1] + pval[2] * sval[2]
    }
    pub fn cross(self: Self, other: Self) -> Self {
        let pval: [f32; 3] = self.val;
        let sval: [f32; 3] = other.val;
        Xyz::new([
            pval[1] * sval[2] - pval[2] * sval[1],
            pval[2] * sval[0] - pval[0] * sval[2],
            pval[0] * sval[1] - pval[1] * sval[0],
        ])
    }
}

impl PartialEq for Xyz {
    fn eq(self: &Self, eqself: &Self) -> bool {
        self.val == eqself.val
    }
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

pub fn rgb_xyz_mul(one: Rgb<u8>, two: Xyz) -> Rgb<u8> {
    Rgb([
        u8_mul(one[0], two.val[0]),
        u8_mul(one[1], two.val[1]),
        u8_mul(one[2], two.val[2]),
    ])
}
