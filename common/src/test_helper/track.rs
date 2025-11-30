// SPDX-FileCopyrightText: 2025 All contributors
//
// SPDX-License-Identifier: GPL-2.0-or-later

use crate::{position::Position, track::Track};

pub fn get_track_as_json<'a>() -> &'a str {
    include_str!("../../../assets/tracks/Oschersleben.json")
}

pub fn get_track() -> Track {
    Track {
        name: "Oschersleben".to_string(),
        startline: Position {
            latitude: 52.0270889,
            longitude: 11.2803483,
        },
        finishline: Some(Position {
            latitude: 52.0270889,
            longitude: 11.2803483,
        }),
        sectors: vec![
            Position {
                latitude: 52.0298205,
                longitude: 11.2741851,
            },
            Position {
                latitude: 52.0299681,
                longitude: 11.2772076,
            },
        ],
    }
}
