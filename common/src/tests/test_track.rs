use crate::{position::Position, track::Track};

fn get_track_as_json<'a>() -> &'a str {
    r#"
    {
        "name": "Oschersleben",
        "startline": {
            "latitude": 52.025833,
            "longitude": 11.279166
        },
        "finishline": {
            "latitude": 52.025833,
            "longitude": 11.279166
        },
        "sectors": [
            {
                "latitude": 52.025833,
                "longitude": 11.279166
            },
            {
                "latitude": 52.025833,
                "longitude": 11.279166
            }
        ]
    }
    "#
}

fn get_track() -> Track {
    Track {
        name: "Oschersleben".to_string(),
        startline: Position {
            latitude: 52.025833,
            longitude: 11.279166,
        },
        finishline: Some(Position {
            latitude: 52.025833,
            longitude: 11.279166,
        }),
        sectors: vec![
            Position {
                latitude: 52.025833,
                longitude: 11.279166,
            },
            Position {
                latitude: 52.025833,
                longitude: 11.279166,
            },
        ],
    }
}

#[test]
pub fn deserialize_track_from_json() {
    let track = Track::from_json(get_track_as_json())
        .unwrap_or_else(|e| panic!("Failed to deserialize the raw json. Reason: {e}"));
    assert_eq!(track, get_track());
}
