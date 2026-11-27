use crate::data::LatLng;

// Approximations for conversion; gathered from Google Earth in Munich
const METERS_PER_DEGREE_LAT: f64 = 114527.0;
const METERS_PER_DEGREE_LNG: f64 = 74383.0;

const MAX_POINTS: usize = 5000;
const MAX_POINTS_DISTANCE: f64 = 15.0;
const ZERO_POINTS_DISTANCE: f64 = 5000.0;

pub struct GuessResults {
    distance_in_meters: f64,
    points: usize,
}

pub fn evaluate_guess(real_coords: LatLng, guessed_coords: LatLng) -> GuessResults {
    let distance_in_meters = calculate_distance_in_meters(real_coords, guessed_coords);
    let points = calculate_points(distance_in_meters);

    GuessResults {
        distance_in_meters,
        points,
    }
}

fn calculate_distance_in_meters(real_coords: LatLng, guessed_coords: LatLng) -> f64 {
    let distance_lat = (real_coords.lat - guessed_coords.lat) * METERS_PER_DEGREE_LAT;
    let distance_lng = (real_coords.lng - guessed_coords.lng) * METERS_PER_DEGREE_LNG;

    let distance_in_meters = (distance_lat.powi(2) + distance_lng.powi(2)).sqrt();

    distance_in_meters
}

fn calculate_points(distance_in_meters: f64) -> usize {
    match distance_in_meters {
        0.0..MAX_POINTS_DISTANCE => MAX_POINTS,
        _ => point_curve(distance_in_meters - MAX_POINTS_DISTANCE),
    }
}

// Formula from: https://latb.io/geoguessr/articles/the-maths
fn point_curve(distance: f64) -> usize {
    // A value so guesses farther than ZERO_POINTS_DISTANCE will be less than 1
    let reference_distance: f64 = -10.0 * ZERO_POINTS_DISTANCE / (1.0 / MAX_POINTS as f64).ln();

    (MAX_POINTS as f64 * (-10.0 * distance / reference_distance).exp()).floor() as usize
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_distance_calculation() {
        let point_a = LatLng {
            lat: 48.148547,
            lng: 11.58615,
        };
        let point_b = LatLng {
            lat: 48.148553,
            lng: 11.586172,
        };
        let a_to_b = 1.73;

        let point_c = LatLng {
            lat: 48.151206,
            lng: 11.579219,
        };
        let point_d = LatLng {
            lat: 48.151061,
            lng: 11.579125,
        };
        let c_to_d = 17.0;

        let point_e = LatLng {
            lat: 48.149953,
            lng: 11.594317,
        };
        let point_f = LatLng {
            lat: 48.132158,
            lng: 11.668464,
        };
        let e_to_f = 5862.0;

        assert!((calculate_distance_in_meters(point_a, point_b) - a_to_b).abs() < a_to_b * 0.1);
        assert!((calculate_distance_in_meters(point_c, point_d) - c_to_d).abs() < c_to_d * 0.1);
        assert!((calculate_distance_in_meters(point_e, point_f) - e_to_f).abs() < e_to_f * 0.1);
    }

    #[test]
    fn test_point_calculation() {
        let well_within_max_dist = calculate_points(MAX_POINTS_DISTANCE / 2.0);
        dbg!(well_within_max_dist);
        let just_within_max_dist = calculate_points(MAX_POINTS_DISTANCE * 0.99);
        dbg!(just_within_max_dist);
        let just_outside_max_dist = calculate_points(MAX_POINTS_DISTANCE * 1.01);
        dbg!(just_outside_max_dist);
        let well_outside_max_dist = calculate_points(
            MAX_POINTS_DISTANCE + ((MAX_POINTS_DISTANCE + ZERO_POINTS_DISTANCE) / 2.0),
        );
        dbg!(well_outside_max_dist);
        let just_within_zero_dist = calculate_points(ZERO_POINTS_DISTANCE * 0.99);
        dbg!(just_within_zero_dist);
        let just_outside_zero_dist = calculate_points(ZERO_POINTS_DISTANCE * 1.01);
        dbg!(just_outside_zero_dist);
        let well_outside_zero_dist = calculate_points(ZERO_POINTS_DISTANCE * 2.0);
        dbg!(well_outside_zero_dist);

        assert!(well_within_max_dist == MAX_POINTS);
        assert!(just_within_max_dist == MAX_POINTS);
        assert!(just_outside_max_dist < MAX_POINTS && just_outside_max_dist > 0);
        assert!(well_outside_max_dist < MAX_POINTS && well_outside_max_dist > 0);
        assert!(just_within_zero_dist < MAX_POINTS && just_within_zero_dist > 0);
        assert!(just_outside_zero_dist == 0);
        assert!(well_outside_zero_dist == 0);
    }
}
