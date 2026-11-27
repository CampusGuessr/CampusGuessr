use std::path::Path;

#[derive(Clone, Copy)]
pub struct LatLng {
    pub lat: f64,
    pub lng: f64,
}

pub struct Location {
    pub file_path: Box<Path>,
    pub coords: LatLng,
}

// Placeholder function, interaction with database required
pub fn get_locations() -> Vec<Location> {
    Vec::new()
}
