pub type Value = i64;

// bit 0 = tag
// 0 => int
// 1 => bool

#[inline(always)]
pub fn int(v: i64) -> Value {
    v << 1
}

#[inline(always)]
pub fn boolv(v: bool) -> Value {
    if v { 3 } else { 1 }
}

#[inline(always)]
pub fn is_bool(v: Value) -> bool {
    (v & 1) == 1
}

#[inline(always)]
pub fn as_int(v: Value) -> i64 {
    v >> 1
}

#[inline(always)]
pub fn as_bool(v: Value) -> bool {
    v == 3
}
