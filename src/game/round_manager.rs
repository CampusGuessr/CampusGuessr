use crate::data::{LatLng, Location};
use crate::game::scoring::{self, GuessResults};
use fastrand::Rng;

pub struct Round<'a> {
    location: &'a Location,
}

impl Round<'_> {
    fn evaluate_guess(&self, guessed_coords: LatLng) -> GuessResults {
        scoring::evaluate_guess(self.location.coords, guessed_coords)
    }
}

pub struct RoundManager {
    locations: Vec<Location>,
    location_index: usize,
    rand: Rng,
}

impl RoundManager {
    pub fn new(locations: Vec<Location>) -> Self {
        RoundManager {
            locations,
            location_index: 0,
            rand: Rng::with_seed(80085),
        }
    }

    pub fn get_next_round(&mut self) -> Option<Round<'_>> {
        if self.locations.is_empty() {
            return None;
        }

        let round = Some(Round {
            location: &self.locations[self.location_index % self.locations.len()],
        });
        self.location_index += 1;

        round
    }

    pub fn get_random_round(&mut self) -> Option<Round<'_>> {
        if self.locations.is_empty() {
            return None;
        }

        Some(Round {
            location: &self.locations[self.rand.usize(0..self.locations.len())],
        })
    }
}
