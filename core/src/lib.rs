#![forbid(unsafe_op_in_unsafe_fn)]

use std::cmp::{max, min};

#[derive(Debug, Clone)]
pub struct FrutigerDrop;

fn clamp_u8(v: i32) -> u8 {
    v.clamp(0, 255) as u8
}

fn idx(x: u32, y: u32, width: u32) -> usize {
    ((y * width + x) as usize) * 4
}

fn validate_rgba_len(buf: &[u8], width: u32, height: u32) {
    let expected = (width as usize)
        .checked_mul(height as usize)
        .and_then(|px| px.checked_mul(4))
        .expect("width/height overflow");
    assert!(
        buf.len() == expected,
        "RGBA buffer length mismatch: got {}, expected {}",
        buf.len(),
        expected
    );
}

fn box_blur_1d_u8(src: &[u8], dst: &mut [u8], radius: i32) {
    debug_assert_eq!(src.len(), dst.len());
    let n = src.len() as i32;
    let r = radius;
    let window = 2 * r + 1;

    let mut sum: i32 = 0;
    for i in -r..=r {
        let ii = min(max(i, 0), n - 1);
        sum += src[ii as usize] as i32;
    }

    for x in 0..n {
        dst[x as usize] = clamp_u8((sum + window / 2) / window);

        let out_i = x - r;
        let in_i = x + r + 1;
        let out_clamped = min(max(out_i, 0), n - 1);
        let in_clamped = min(max(in_i, 0), n - 1);
        sum += src[in_clamped as usize] as i32 - src[out_clamped as usize] as i32;
    }
}

fn blur_rgba_box(input: &[u8], width: u32, height: u32, radius: i32) -> Vec<u8> {
    validate_rgba_len(input, width, height);
    let w = width as usize;
    let h = height as usize;

    let mut tmp = vec![0u8; input.len()];
    let mut out = vec![0u8; input.len()];

    let mut row_src = vec![0u8; w];
    let mut row_dst = vec![0u8; w];
    for y in 0..h {
        for c in 0..4 {
            for x in 0..w {
                row_src[x] = input[(y * w + x) * 4 + c];
            }
            box_blur_1d_u8(&row_src, &mut row_dst, radius);
            for x in 0..w {
                tmp[(y * w + x) * 4 + c] = row_dst[x];
            }
        }
    }

    let mut col_src = vec![0u8; h];
    let mut col_dst = vec![0u8; h];
    for x in 0..w {
        for c in 0..4 {
            for y in 0..h {
                col_src[y] = tmp[(y * w + x) * 4 + c];
            }
            box_blur_1d_u8(&col_src, &mut col_dst, radius);
            for y in 0..h {
                out[(y * w + x) * 4 + c] = col_dst[y];
            }
        }
    }

    out
}

pub fn apply_blur(input: &[u8], width: u32, height: u32) -> Vec<u8> {
    blur_rgba_box(input, width, height, 3)
}

pub fn apply_tint(rgba: &[u8], tint: (u8, u8, u8, u8)) -> Vec<u8> {
    assert!(rgba.len() % 4 == 0, "RGBA length must be multiple of 4");
    let (tr, tg, tb, ta) = tint;
    let mut out = vec![0u8; rgba.len()];
    for (i, px) in rgba.chunks_exact(4).enumerate() {
        let r = px[0] as u16;
        let g = px[1] as u16;
        let b = px[2] as u16;
        let a = px[3] as u16;
        out[i * 4 + 0] = ((r * tr as u16 + 127) / 255) as u8;
        out[i * 4 + 1] = ((g * tg as u16 + 127) / 255) as u8;
        out[i * 4 + 2] = ((b * tb as u16 + 127) / 255) as u8;
        out[i * 4 + 3] = ((a * ta as u16 + 127) / 255) as u8;
    }
    out
}

pub fn composite_layers(bottom_rgba: &[u8], top_rgba: &[u8]) -> Vec<u8> {
    assert!(
        bottom_rgba.len() == top_rgba.len(),
        "Layer buffers must match in length"
    );
    assert!(bottom_rgba.len() % 4 == 0, "RGBA length must be multiple of 4");

    let mut out = vec![0u8; bottom_rgba.len()];
    for i in 0..(bottom_rgba.len() / 4) {
        let bi = i * 4;
        let br = bottom_rgba[bi + 0] as f32 / 255.0;
        let bg = bottom_rgba[bi + 1] as f32 / 255.0;
        let bb = bottom_rgba[bi + 2] as f32 / 255.0;
        let ba = bottom_rgba[bi + 3] as f32 / 255.0;

        let tr = top_rgba[bi + 0] as f32 / 255.0;
        let tg = top_rgba[bi + 1] as f32 / 255.0;
        let tb = top_rgba[bi + 2] as f32 / 255.0;
        let ta = top_rgba[bi + 3] as f32 / 255.0;

        let out_a = ta + ba * (1.0 - ta);
        let (out_r, out_g, out_b) = if out_a <= 0.0 {
            (0.0, 0.0, 0.0)
        } else {
            let r = (tr * ta + br * ba * (1.0 - ta)) / out_a;
            let g = (tg * ta + bg * ba * (1.0 - ta)) / out_a;
            let b = (tb * ta + bb * ba * (1.0 - ta)) / out_a;
            (r, g, b)
        };

        out[bi + 0] = clamp_u8((out_r * 255.0).round() as i32);
        out[bi + 1] = clamp_u8((out_g * 255.0).round() as i32);
        out[bi + 2] = clamp_u8((out_b * 255.0).round() as i32);
        out[bi + 3] = clamp_u8((out_a * 255.0).round() as i32);
    }
    out
}

#[cfg(feature = "ffi")]
pub mod ffi {
    use super::{apply_blur, apply_tint, composite_layers, validate_rgba_len};
    use std::slice;

    #[repr(C)]
    pub struct FrutigerDropBuffer {
        pub ptr: *mut u8,
        pub len: usize,
    }

    fn vec_to_buffer(mut v: Vec<u8>) -> FrutigerDropBuffer {
        let buf = FrutigerDropBuffer {
            ptr: v.as_mut_ptr(),
            len: v.len(),
        };
        std::mem::forget(v);
        buf
    }

    #[no_mangle]
    pub extern "C" fn frutiger_drop_free(buf: FrutigerDropBuffer) {
        if buf.ptr.is_null() || buf.len == 0 {
            return;
        }
        unsafe {
            drop(Vec::from_raw_parts(buf.ptr, buf.len, buf.len));
        }
    }

    #[no_mangle]
    pub extern "C" fn frutiger_drop_apply_blur(
        input_ptr: *const u8,
        input_len: usize,
        width: u32,
        height: u32,
    ) -> FrutigerDropBuffer {
        if input_ptr.is_null() {
            return FrutigerDropBuffer {
                ptr: std::ptr::null_mut(),
                len: 0,
            };
        }
        let input = unsafe { slice::from_raw_parts(input_ptr, input_len) };
        validate_rgba_len(input, width, height);
        vec_to_buffer(apply_blur(input, width, height))
    }

    #[no_mangle]
    pub extern "C" fn frutiger_drop_apply_tint(
        rgba_ptr: *const u8,
        rgba_len: usize,
        tr: u8,
        tg: u8,
        tb: u8,
        ta: u8,
    ) -> FrutigerDropBuffer {
        if rgba_ptr.is_null() {
            return FrutigerDropBuffer {
                ptr: std::ptr::null_mut(),
                len: 0,
            };
        }
        let rgba = unsafe { slice::from_raw_parts(rgba_ptr, rgba_len) };
        vec_to_buffer(apply_tint(rgba, (tr, tg, tb, ta)))
    }

    #[no_mangle]
    pub extern "C" fn frutiger_drop_composite_layers(
        bottom_ptr: *const u8,
        bottom_len: usize,
        top_ptr: *const u8,
        top_len: usize,
    ) -> FrutigerDropBuffer {
        if bottom_ptr.is_null() || top_ptr.is_null() {
            return FrutigerDropBuffer {
                ptr: std::ptr::null_mut(),
                len: 0,
            };
        }
        let bottom = unsafe { slice::from_raw_parts(bottom_ptr, bottom_len) };
        let top = unsafe { slice::from_raw_parts(top_ptr, top_len) };
        vec_to_buffer(composite_layers(bottom, top))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tint_identity_is_noop() {
        let rgba = vec![10u8, 20, 30, 40, 200, 150, 100, 50];
        let out = apply_tint(&rgba, (255, 255, 255, 255));
        assert_eq!(out, rgba);
    }

    #[test]
    fn composite_over_basic_alpha() {
        let bottom = vec![0u8, 0, 0, 255];
        let top = vec![255u8, 0, 0, 128];
        let out = composite_layers(&bottom, &top);
        assert_eq!(out.len(), 4);
        assert!(out[0] > 0);
        assert_eq!(out[3], 255);
    }

    #[test]
    fn blur_preserves_length() {
        let width = 4;
        let height = 4;
        let rgba = vec![255u8; (width * height * 4) as usize];
        let out = apply_blur(&rgba, width, height);
        assert_eq!(out.len(), rgba.len());
    }
}

