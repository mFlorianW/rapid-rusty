use crate::{position::Position, track::Track};

pub fn get_track_as_json<'a>() -> &'a str {
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

pub fn get_track() -> Track {
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
