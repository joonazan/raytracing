use num_complex::Complex;

pub const IRON_REFRACTIVE_INDEX: Complex<f32> = Complex::new(2.9304, 2.9996);

pub fn amount_reflected(angle_cosine: f32, refractive_index: Complex<f32>) -> f32 {
    let refraction_angle_cosine =
        (1. - (1. - angle_cosine * angle_cosine) / (refractive_index * refractive_index)).sqrt();

    let r_parl = (refractive_index * angle_cosine - refraction_angle_cosine)
        / (refractive_index * angle_cosine + refraction_angle_cosine);
    let r_perp = (angle_cosine - refractive_index * refraction_angle_cosine)
        / (angle_cosine + refractive_index * refraction_angle_cosine);
    return (r_parl.norm() + r_perp.norm()) / 2.;
}
