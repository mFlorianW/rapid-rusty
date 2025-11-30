// SPDX-FileCopyrightText: 2025 All contributors
//
// SPDX-License-Identifier: GPL-2.0-or-later

use common::position::Position;

fn get_position_as_json<'a>() -> &'a str {
    r#"
    {
        "latitude": 52.025833,
        "longitude": 11.279166
    }
    "#
}

fn get_position() -> Position {
    Position {
        latitude: 52.025833,
        longitude: 11.279166,
    }
}

#[test]
pub fn deserialize_position_from_json() {
    let pos = Position::from_json(get_position_as_json())
        .unwrap_or_else(|e| panic!("Failed to deserialize the raw json. Reason: {e}"));
    assert_eq!(pos, get_position());
}
