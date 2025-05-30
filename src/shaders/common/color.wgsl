

// https://github.com/KhronosGroup/ToneMapping/blob/main/PBR_Neutral/README.md#pbr-neutral-specification
fn to_khronos_pbr_neutral(color: vec3<f32>) -> vec3<f32> {
    let k_s = 0.8 - F90;
    let k_d = 0.15;
    
    let x = min(min(color.r, color.g), color.b);

    let f = 
        (x - ((x*x)/(4.0 * F90))) * f32(x <= (2.0 * F90)) 
        + 
        F90 * f32(x > (2.0 * F90));

    let p = max(max(color.r - f, color.g - f), color.b - f);

    let p_n = 1.0 - (((1.0 - k_s) * (1.0 - k_s)) / (p + 1 - 2.0 * k_s));

    let g = (1.0 / (k_d * (p - p_n) + 1.0));

    let c_out = 
        (color - f) * f32(p <= k_s)
        + 
        ((color - f) * (p_n / p) * g + vec3<f32>(p_n * (1.0 - g))) * f32(p > k_s);

    return c_out;
}

// https://github.com/KhronosGroup/ToneMapping/blob/b5a2eed5ddf6c2227090449399de9c7affb9e4c9/PBR_Neutral/lut-writer.mjs#L84
fn from_khronos_pbr_neutral(in_color: vec3<f32>) -> vec3<f32> {
    let start_compression = 0.8 - F90;
    let desaturation = 0.15;

    var color = in_color;

    let peak = max(max(color.r, color.g), color.b);

    if (peak > start_compression) {
        let d = 1.0 - start_compression;
        let old_peak = d * d / (1.0 - peak) - d + start_compression;
        let fInv = desaturation * (old_peak - peak) + 1;
        let f = 1.0 / fInv;
        color.r = (color.r + (f - 1.0) * peak) * fInv;
        color.g = (color.g + (f - 1.0) * peak) * fInv;
        color.b = (color.b + (f - 1.0) * peak) * fInv;
        let scale = old_peak / peak;
        color *= scale;
    }

    let y = min(min(color.r, color.g), color.b);
    
    var offset = F90;
    if (y < F90) {
        let x = sqrt(y / 6.25);
        offset = x - 6.25 * x * x;
    }
    color += offset;

    return color;
}