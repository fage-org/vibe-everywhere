use base64::{Engine as _, engine::general_purpose::STANDARD};

pub fn compute_thumbhash(width: u32, height: u32, rgba: &[u8]) -> String {
    let hash = encode_thumbhash(width, height, rgba);
    STANDARD.encode(hash)
}

fn encode_thumbhash(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    assert!(
        width <= 100 && height <= 100,
        "{width}x{height} doesn't fit in 100x100"
    );

    let w = width as usize;
    let h = height as usize;
    let pixel_count = w * h;

    let mut avg_r = 0.0_f64;
    let mut avg_g = 0.0_f64;
    let mut avg_b = 0.0_f64;
    let mut avg_a = 0.0_f64;
    for pixel in rgba.chunks_exact(4) {
        let alpha = pixel[3] as f64 / 255.0;
        avg_r += alpha * pixel[0] as f64 / 255.0;
        avg_g += alpha * pixel[1] as f64 / 255.0;
        avg_b += alpha * pixel[2] as f64 / 255.0;
        avg_a += alpha;
    }
    if avg_a > 0.0 {
        avg_r /= avg_a;
        avg_g /= avg_a;
        avg_b /= avg_a;
    }

    let has_alpha = avg_a < pixel_count as f64;
    let l_limit = if has_alpha { 5 } else { 7 };
    let max_side = width.max(height) as f64;
    let lx = ((l_limit as f64 * width as f64 / max_side).round() as usize).max(1);
    let ly = ((l_limit as f64 * height as f64 / max_side).round() as usize).max(1);

    let mut luminance = vec![0.0_f64; pixel_count];
    let mut yellow_blue = vec![0.0_f64; pixel_count];
    let mut red_green = vec![0.0_f64; pixel_count];
    let mut alpha_channel = vec![0.0_f64; pixel_count];

    for (i, pixel) in rgba.chunks_exact(4).enumerate() {
        let alpha = pixel[3] as f64 / 255.0;
        let r = avg_r * (1.0 - alpha) + alpha * pixel[0] as f64 / 255.0;
        let g = avg_g * (1.0 - alpha) + alpha * pixel[1] as f64 / 255.0;
        let b = avg_b * (1.0 - alpha) + alpha * pixel[2] as f64 / 255.0;
        luminance[i] = (r + g + b) / 3.0;
        yellow_blue[i] = (r + g) / 2.0 - b;
        red_green[i] = r - g;
        alpha_channel[i] = alpha;
    }

    let (l_dc, l_ac, l_scale) = encode_channel(&luminance, w, h, 3.max(lx), 3.max(ly));
    let (p_dc, p_ac, p_scale) = encode_channel(&yellow_blue, w, h, 3, 3);
    let (q_dc, q_ac, q_scale) = encode_channel(&red_green, w, h, 3, 3);
    let (a_dc, a_ac, a_scale) = if has_alpha {
        encode_channel(&alpha_channel, w, h, 5, 5)
    } else {
        (0.0, vec![0.0], 0.0)
    };

    let is_landscape = width > height;
    let header24 = ((63.0 * l_dc).round() as u32)
        | (((31.5 + 31.5 * p_dc).round() as u32) << 6)
        | (((31.5 + 31.5 * q_dc).round() as u32) << 12)
        | (((31.0 * l_scale).round() as u32) << 18)
        | ((has_alpha as u32) << 23);
    let header16 = ((if is_landscape { ly } else { lx }) as u32)
        | (((63.0 * p_scale).round() as u32) << 3)
        | (((63.0 * q_scale).round() as u32) << 9)
        | ((is_landscape as u32) << 15);

    let mut hash = vec![
        (header24 & 255) as u8,
        ((header24 >> 8) & 255) as u8,
        (header24 >> 16) as u8,
        (header16 & 255) as u8,
        (header16 >> 8) as u8,
    ];
    let ac_start = if has_alpha { 6 } else { 5 };
    let mut ac_index = 0usize;
    if has_alpha {
        hash.push((15.0 * a_dc).round() as u8 | (((15.0 * a_scale).round() as u8) << 4));
    }

    for ac_terms in [&l_ac, &p_ac, &q_ac] {
        write_ac_terms(&mut hash, ac_terms, ac_start, &mut ac_index);
    }
    if has_alpha {
        write_ac_terms(&mut hash, &a_ac, ac_start, &mut ac_index);
    }

    hash
}

fn encode_channel(
    channel: &[f64],
    width: usize,
    height: usize,
    nx: usize,
    ny: usize,
) -> (f64, Vec<f64>, f64) {
    let mut dc = 0.0_f64;
    let mut ac = Vec::new();
    let mut scale = 0.0_f64;
    let mut fx = vec![0.0_f64; width];

    for cy in 0..ny {
        for cx in 0..nx {
            if cx * ny >= nx * (ny - cy) {
                continue;
            }
            for (x, value) in fx.iter_mut().enumerate() {
                *value = (std::f64::consts::PI / width as f64 * cx as f64 * (x as f64 + 0.5)).cos();
            }
            let mut f = 0.0_f64;
            for y in 0..height {
                let fy =
                    (std::f64::consts::PI / height as f64 * cy as f64 * (y as f64 + 0.5)).cos();
                for x in 0..width {
                    f += channel[x + y * width] * fx[x] * fy;
                }
            }
            f /= (width * height) as f64;
            if cx != 0 || cy != 0 {
                scale = scale.max(f.abs());
                ac.push(f);
            } else {
                dc = f;
            }
        }
    }

    if scale > 0.0 {
        for value in &mut ac {
            *value = 0.5 + 0.5 / scale * *value;
        }
    }

    (dc, ac, scale)
}

fn write_ac_terms(hash: &mut Vec<u8>, ac_terms: &[f64], ac_start: usize, ac_index: &mut usize) {
    for &value in ac_terms {
        let index = ac_start + (*ac_index >> 1);
        if hash.len() <= index {
            hash.push(0);
        }
        hash[index] |= ((15.0 * value).round() as u8) << (((*ac_index & 1) as u8) << 2);
        *ac_index += 1;
    }
}

#[cfg(test)]
mod tests {
    use image::{ImageBuffer, Rgba};

    use super::compute_thumbhash;

    #[test]
    fn thumbhash_is_deterministic() {
        let image = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_pixel(2, 1, Rgba([255, 0, 0, 255]));
        let hash1 = compute_thumbhash(2, 1, image.as_raw());
        let hash2 = compute_thumbhash(2, 1, image.as_raw());
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn thumbhash_matches_happy_fixture_for_simple_red_image() {
        let image = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_pixel(2, 1, Rgba([255, 0, 0, 255]));
        let hash = compute_thumbhash(2, 1, image.as_raw());
        assert_eq!(hash, "1fsrBP94CIeIiIeAeId4iICHCA==");
    }
}
