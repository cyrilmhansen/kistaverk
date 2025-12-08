use std::f64::consts::TAU;

/// Low-pass filter for scalar values.
/// Returns `None` when the sample is non-finite.
pub fn low_pass_scalar(previous: Option<f64>, sample: f64, alpha: f64) -> Option<f64> {
    if !sample.is_finite() {
        return None;
    }
    let a = alpha.clamp(0.0, 1.0);
    match (previous, a) {
        (_, a) if (a - 1.0).abs() < f64::EPSILON => Some(sample),
        (Some(prev), a) if a > 0.0 => Some(prev + a * (sample - prev)),
        (Some(prev), _) => Some(prev),
        (None, _) => Some(sample),
    }
}

/// Low-pass filter for angles in radians (wraps around TAU).
/// Returns `None` when the sample is non-finite.
pub fn low_pass_angle(previous: Option<f64>, sample: f64, alpha: f64) -> Option<f64> {
    if !sample.is_finite() {
        return None;
    }
    let a = alpha.clamp(0.0, 1.0);
    let target = normalize_angle(sample);
    match (previous, a) {
        (None, _) => Some(target),
        (Some(prev), a) if a <= 0.0 => Some(prev),
        (Some(prev), _) => {
            let delta = shortest_delta(prev, target);
            Some(normalize_angle(prev + a * delta))
        }
    }
}

fn normalize_angle(angle: f64) -> f64 {
    let mut wrapped = angle % TAU;
    if wrapped < 0.0 {
        wrapped += TAU;
    }
    wrapped
}

fn shortest_delta(from: f64, to: f64) -> f64 {
    let mut delta = to - from;
    if delta > std::f64::consts::PI {
        delta -= TAU;
    } else if delta < -std::f64::consts::PI {
        delta += TAU;
    }
    delta
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_low_pass_progresses_toward_sample() {
        let next = low_pass_scalar(Some(0.0), 10.0, 0.25).unwrap();
        assert!((next - 2.5).abs() < 1e-9);
    }

    #[test]
    fn scalar_low_pass_starts_from_sample_when_empty() {
        let next = low_pass_scalar(None, 5.0, 0.5).unwrap();
        assert!((next - 5.0).abs() < 1e-9);
    }

    #[test]
    fn angle_low_pass_wraps_across_zero() {
        let prev = 350f64.to_radians();
        let sample = 10f64.to_radians();
        let next = low_pass_angle(Some(prev), sample, 0.5).unwrap();
        let expected = normalize_angle(prev + 0.5 * shortest_delta(prev, normalize_angle(sample)));
        assert!((next - expected).abs() < 1e-9);
    }

    #[test]
    fn angle_low_pass_handles_half_turn_delta() {
        let prev = 0.0;
        let sample = std::f64::consts::PI;
        let next = low_pass_angle(Some(prev), sample, 0.5).unwrap();
        assert!((next - (std::f64::consts::PI / 2.0)).abs() < 1e-9);
    }

    #[test]
    fn invalid_scalar_sample_returns_none() {
        assert!(low_pass_scalar(Some(1.0), f64::NAN, 0.2).is_none());
    }

    #[test]
    fn invalid_angle_sample_returns_none() {
        assert!(low_pass_angle(Some(1.0), f64::INFINITY, 0.2).is_none());
    }
}
