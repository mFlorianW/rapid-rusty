use crate::is_on_track;
use common::position::Position;
use common::test_helper::track::get_track;

#[test]
fn position_is_in_radius() {
    let detection_radius = 500_u16;
    let tracks = vec![get_track()];
    let test_pos = Position {
        latitude: 52.0258333,
        longitude: 11.279166666,
    };
    let detected_tracks = is_on_track(&tracks, &test_pos, detection_radius);
    assert_eq!(1, detected_tracks.len());
    assert_eq!(tracks[0], *detected_tracks[0]);
}

#[test]
fn position_is_not_in_radius() {
    let detection_radius = 500_u16;
    let tracks = vec![get_track()];
    let test_pos = Position {
        latitude: 52.0225,
        longitude: 11.29,
    };
    let detected_tracks = is_on_track(&tracks, &test_pos, detection_radius);
    assert_eq!(0, detected_tracks.len());
}
