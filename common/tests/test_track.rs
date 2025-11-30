// SPDX-FileCopyrightText: 2025 All contributors
//
// SPDX-License-Identifier: GPL-2.0-or-later

use common::{test_helper::track::get_track, test_helper::track::get_track_as_json, track::Track};

#[test]
pub fn deserialize_track_from_json() {
    let track = Track::from_json(get_track_as_json())
        .unwrap_or_else(|e| panic!("Failed to deserialize the raw json. Reason: {e}"));
    assert_eq!(track, get_track());
}
