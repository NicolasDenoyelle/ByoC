use std::cmp::{Ord, Ordering};
use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

/// Unsigned Number holding a very big number
#[derive(Clone, Copy, Debug, Hash)]
#[allow(non_camel_case_types)]
pub struct u256 {
    value: [u64; 4],
}

impl std::convert::From<u64> for u256 {
    fn from(c: u64) -> Self {
        u256 {
            value: [c, 0, 0, 0],
        }
    }
}

impl std::convert::From<u32> for u256 {
    fn from(c: u32) -> Self {
        u256 {
            value: [c as u64, 0, 0, 0],
        }
    }
}

impl std::convert::From<u16> for u256 {
    fn from(c: u16) -> Self {
        u256 {
            value: [c as u64, 0, 0, 0],
        }
    }
}

impl std::convert::From<u8> for u256 {
    fn from(c: u8) -> Self {
        u256 {
            value: [c as u64, 0, 0, 0],
        }
    }
}

impl std::convert::From<i64> for u256 {
    fn from(c: i64) -> Self {
        u256 {
            value: [c as u64, 0, 0, 0],
        }
    }
}

impl std::convert::From<i32> for u256 {
    fn from(c: i32) -> Self {
        u256 {
            value: [c as u64, 0, 0, 0],
        }
    }
}

impl std::convert::From<i16> for u256 {
    fn from(c: i16) -> Self {
        u256 {
            value: [c as u64, 0, 0, 0],
        }
    }
}

impl std::convert::From<i8> for u256 {
    fn from(c: i8) -> Self {
        u256 {
            value: [c as u64, 0, 0, 0],
        }
    }
}

impl std::convert::From<&u64> for u256 {
    fn from(c: &u64) -> Self {
        u256 {
            value: [*c, 0, 0, 0],
        }
    }
}

impl std::convert::From<&u32> for u256 {
    fn from(c: &u32) -> Self {
        u256 {
            value: [*c as u64, 0, 0, 0],
        }
    }
}

impl std::convert::From<&u16> for u256 {
    fn from(c: &u16) -> Self {
        u256 {
            value: [*c as u64, 0, 0, 0],
        }
    }
}

impl std::convert::From<&u8> for u256 {
    fn from(c: &u8) -> Self {
        u256 {
            value: [*c as u64, 0, 0, 0],
        }
    }
}

impl std::convert::From<&i64> for u256 {
    fn from(c: &i64) -> Self {
        u256 {
            value: [*c as u64, 0, 0, 0],
        }
    }
}

impl std::convert::From<&i32> for u256 {
    fn from(c: &i32) -> Self {
        u256 {
            value: [*c as u64, 0, 0, 0],
        }
    }
}

impl std::convert::From<&i16> for u256 {
    fn from(c: &i16) -> Self {
        u256 {
            value: [*c as u64, 0, 0, 0],
        }
    }
}

impl std::convert::From<&i8> for u256 {
    fn from(c: &i8) -> Self {
        u256 {
            value: [*c as u64, 0, 0, 0],
        }
    }
}

impl std::convert::From<f32> for u256 {
    fn from(c: f32) -> Self {
        let mut ret = u256 {
            value: [0, 0, 0, 0],
        };
        let mut p = c.log2();

        if p > 256.0 {
            return u256 {
                value: [
                    u64::max_value(),
                    u64::max_value(),
                    u64::max_value(),
                    u64::max_value(),
                ],
            };
        }

        if p < 63.0 {
            return u256 {
                value: [c as u64, 0, 0, 0],
            };
        }

        for i in (1..4).rev() {
            let r = 64f32 * i as f32;
            if p >= r {
                ret.value[i] = 2f32.powf(p - r) as u64;
                p -= 64f32;
            }
        }

        if p > 0.0 {
            ret.value[0] = 2f32.powf(p) as u64;
        }

        ret
    }
}

impl std::convert::From<&f32> for u256 {
    fn from(c: &f32) -> Self {
        u256::from(*c)
    }
}

impl std::convert::From<f64> for u256 {
    fn from(c: f64) -> Self {
        let mut ret = u256 {
            value: [0, 0, 0, 0],
        };
        let mut p = c.log2();

        if p > 256.0 {
            return u256 {
                value: [
                    u64::max_value(),
                    u64::max_value(),
                    u64::max_value(),
                    u64::max_value(),
                ],
            };
        }

        if p < 63.0 {
            return u256 {
                value: [c as u64, 0, 0, 0],
            };
        }

        for i in (1..4).rev() {
            let r = 64f64 * i as f64;
            if p >= r {
                ret.value[i] = 2f64.powf(p - r) as u64;
                p -= 64f64;
            }
        }

        if p > 0.0 {
            ret.value[0] = 2f64.powf(p) as u64;
        }

        ret
    }
}

impl std::convert::From<&f64> for u256 {
    fn from(c: &f64) -> Self {
        u256::from(*c)
    }
}

impl std::convert::From<u256> for f32 {
    fn from(c: u256) -> Self {
        let mut sum = 0f32;

        if c == u256::ZERO {
            return 0.0;
        };
        for i in 0..4 {
            if c.value[i] > 0 {
                sum += f32::log2(c.value[i] as f32) + (i * 64) as f32;
            }
        }
        2f32.powf(sum)
    }
}

impl std::convert::From<u256> for f64 {
    fn from(c: u256) -> Self {
        let mut sum = 0f64;

        if c == u256::ZERO {
            return 0.0;
        };
        for i in 0..4 {
            if c.value[i] > 0 {
                sum += f64::log2(c.value[i] as f64) + (i * 64) as f64;
            }
        }
        2f64.powf(sum)
    }
}

impl Ord for u256 {
    fn cmp(&self, other: &Self) -> Ordering {
        for (i, j) in self.value.iter().rev().zip(other.value.iter().rev())
        {
            if i < j {
                return Ordering::Less;
            }
            if i > j {
                return Ordering::Greater;
            }
        }
        Ordering::Equal
    }
}

impl PartialOrd for u256 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for u256 {
    fn eq(&self, other: &Self) -> bool {
        for i in 0..4 {
            if self.value[i] != other.value[i] {
                return false;
            }
        }
        true
    }
}

impl Eq for u256 {}

impl u256 {
    pub const ZERO: u256 = u256 {
        value: [0, 0, 0, 0],
    };
    pub const ONE: u256 = u256 {
        value: [1, 0, 0, 0],
    };

    fn add_i(&mut self, i: usize, x: u64) {
        if i > 3 {
            self.value[0] = u64::max_value();
            self.value[1] = u64::max_value();
            self.value[2] = u64::max_value();
            self.value[3] = u64::max_value();
            return;
        }

        let (y, b) = self.value[i].overflowing_add(x);
        self.value[i] = y;
        if b && i == 3 {
            self.value[0] = u64::max_value();
            self.value[1] = u64::max_value();
            self.value[2] = u64::max_value();
            self.value[3] = u64::max_value();
            return;
        }

        if b {
            self.add_i(i + 1, 1)
        }
    }

    fn sub_i(&mut self, i: usize, x: u64) -> bool {
        if i > 3 {
            self.value[0] = 0;
            self.value[1] = 0;
            self.value[2] = 0;
            self.value[3] = 0;
            return true;
        };

        let (y, b) = self.value[i].overflowing_sub(x);
        if b && self.sub_i(i + 1, 1) {
            return true;
        };
        self.value[i] = y;
        false
    }
}

impl AddAssign for u256 {
    fn add_assign(&mut self, other: Self) {
        for i in 0..4 {
            if other.value[i] != 0 {
                self.add_i(i, other.value[i])
            }
        }
    }
}

impl Add for u256 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        let mut r = self;
        for i in 0..4 {
            if other.value[i] != 0 {
                r.add_i(i, other.value[i])
            }
        }
        r
    }
}

impl Add for &u256 {
    type Output = u256;

    fn add(self, other: Self) -> Self::Output {
        let mut r = self.clone();
        for i in 0..4 {
            if other.value[i] != 0 {
                r.add_i(i, other.value[i])
            }
        }
        r
    }
}

impl SubAssign for u256 {
    fn sub_assign(&mut self, other: Self) {
        for i in 0..4 {
            if other.value[i] != 0 {
                self.sub_i(i, other.value[i]);
            }
        }
    }
}

impl Sub for u256 {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        let mut r = self.clone();
        for i in 0..4 {
            if other.value[i] != 0 {
                r.sub_i(i, other.value[i]);
            }
        }
        r
    }
}

impl Sub for &u256 {
    type Output = u256;

    fn sub(self, other: Self) -> Self::Output {
        let mut r = self.clone();
        for i in 0..4 {
            if other.value[i] != 0 {
                r.sub_i(i, other.value[i]);
            }
        }
        r
    }
}

impl Mul for u256 {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        let mut r = self;
        for i in 0..4 {
            if other.value[i] != 1 {
                let (y, b) = r.value[i].overflowing_mul(other.value[i]);
                r.value[i] = y;
                if b {
                    r.add_i(i + 1, 1)
                }
            }
        }
        r
    }
}

#[cfg(test)]
mod tests {
    use super::u256;

    #[test]
    fn test_eq() {
        let mut v = u256 {
            value: [8200, 0, 0, 0],
        };
        for _ in 0..1000 {
            v += u256::ONE;
            assert_eq!(v, v);
        }
        v = u256 {
            value: [std::u64::MAX, 0, 0, 0],
        };
        for _ in 0..1000 {
            v += u256::ONE;
            assert_eq!(v, v);
        }
    }

    #[test]
    fn test_from_f64() {
        assert_eq!(
            u256::from(2f64.powf(64.0)),
            u256 {
                value: [0, 1, 0, 0]
            }
        );
        assert_eq!(
            u256::from(3f64),
            u256 {
                value: [3u64, 0, 0, 0]
            }
        );
        assert_eq!(
            u256::from(2f64),
            u256 {
                value: [2u64, 0, 0, 0]
            }
        );
        assert_eq!(
            u256::from(1f64),
            u256 {
                value: [1u64, 0, 0, 0]
            }
        );
        assert_eq!(
            u256::from(0f64),
            u256 {
                value: [0, 0, 0, 0]
            }
        );
    }

    #[test]
    fn test_from_f32() {
        assert_eq!(
            u256::from(2f32.powf(64.0)),
            u256 {
                value: [0, 1, 0, 0]
            }
        );
        assert_eq!(
            u256::from(3f32),
            u256 {
                value: [3u64, 0, 0, 0]
            }
        );
        assert_eq!(
            u256::from(2f32),
            u256 {
                value: [2u64, 0, 0, 0]
            }
        );
        assert_eq!(
            u256::from(1f32),
            u256 {
                value: [1u64, 0, 0, 0]
            }
        );
        assert_eq!(
            u256::from(0f32),
            u256 {
                value: [0, 0, 0, 0]
            }
        );
    }

    #[test]
    fn test_to_f32() {
        assert_eq!(f32::from(u256::ZERO), 0.0);
        assert_eq!(f32::from(u256::ONE), 1.0);

        let a = f32::from(u256 {
            value: [988, 0, 0, 0],
        });
        let b = 988f32;
        assert!((a - b) * (a - b) / (a + b) * (a + b) < 1e9);

        let c = u256 {
            value: [0, 1, 0, 0],
        };
        assert_eq!(f32::from(c), 2f32.powf(64.0));
    }

    #[test]
    fn test_to_f64() {
        assert_eq!(f64::from(u256::ZERO), 0.0);
        assert_eq!(f64::from(u256::ONE), 1.0);

        let a = f64::from(u256 {
            value: [988, 0, 0, 0],
        });
        let b = 988f64;
        assert!((a - b) * (a - b) / (a + b) * (a + b) < 1e9);

        let c = u256 {
            value: [0, 1, 0, 0],
        };
        assert_eq!(f64::from(c), 2f64.powf(64.0));
    }

    #[test]
    fn test_sub() {
        let a = u256 {
            value: [0, 1, 0, 0],
        };
        let b = u256 {
            value: [u64::max_value(), 0, 0, 0],
        };
        assert_eq!(a - u256::ONE, b);

        let c = u256 {
            value: [0, 100000, 0, 0],
        };
        let d = u256 {
            value: [0, 10000, 0, 0],
        };
        let e = u256 {
            value: [0, 100000 - 10000, 0, 0],
        };
        assert_eq!(c - d, e);
    }

    #[test]
    fn test_add() {
        let a = u256 {
            value: [0, 1, 0, 0],
        };
        let b = u256 {
            value: [u64::max_value(), 0, 0, 0],
        };
        assert_eq!(b + u256::ONE, a);

        let c = u256 {
            value: [0, 100000, 0, 0],
        };
        let d = u256 {
            value: [0, 10000, 0, 0],
        };
        let e = u256 {
            value: [0, 100000 + 10000, 0, 0],
        };
        assert_eq!(c + d, e);
    }

    #[test]
    fn test_inc() {
        let mut a = u256 {
            value: [u64::max_value() - 10u64, 0, 0, 0],
        };
        let mut b = a.value[0];

        for _ in 0..100 {
            a = a + u256::from(1u64);
            if a.value[0] != 0 {
                assert!(a.value[0] == 0 || a.value[0] == b + 1);
            }
            b = a.value[0]
        }
    }
}
