use common::position::Position;
use common::track::Track;
use core::f64;

/// Returns a list of references to tracks whose start line is within a specified detection radius of a given position.
///
/// Iterates through the provided collection of tracks, calculates the distance from each track’s start line to the specified position,
/// and collects references to tracks that are within the given detection radius.
///
/// # Parameters
/// - `tracks`: A reference to a vector of `Track` instances to check.
/// - `pos`: The `Position` from which distances are measured.
/// - `detection_radius`: The maximum distance (in meters) between `pos` and a track’s start line to consider it detected.
///
/// # Returns
/// A vector containing references to tracks whose start line is within the specified detection radius.
pub fn is_on_track<'a>(
    tracks: &'a Vec<Track>,
    pos: &Position,
    detection_radius: u16,
) -> Vec<&'a Track> {
    let mut detected_tracks = Vec::<&Track>::new();
    for track in tracks {
        let distance = calculate_distance(&track.startline, pos);
        if distance <= detection_radius as f64 {
            detected_tracks.push(track);
        }
    }
    detected_tracks
}

/// Calculates the approximate distance in meters between two geographic positions.
///
/// This function uses a simplified equirectangular approximation to determine  
/// the distance between two latitude/longitude points. It assumes that the  
/// Earth's surface is locally flat and therefore does not account for  
/// large-scale curvature or ellipsoidal effects, making it suitable only  
/// for relatively short distances.
///
/// # Parameters
/// - `pos1`: Reference to the first geographic position.
/// - `pos2`: Reference to the second geographic position.
///
/// # Returns
/// The calculated distance between `pos1` and `pos2` in meters as a `f64`.
///
/// # Notes
/// - The function expects latitude and longitude values in **degrees**.
/// - Accuracy decreases over long distances or near the poles.
/// - This method is more efficient than more precise formulas (e.g., Haversine)  
///   but trades some accuracy for performance.
pub fn calculate_distance(pos1: &Position, pos2: &Position) -> f64 {
    let lat = (pos1.latitude + pos2.latitude) / 2.0 * 0.01745;
    let dx = 111300.0 * lat.cos() * (pos1.longitude - pos2.longitude);
    let dy = 111300.0 * (pos1.latitude - pos2.latitude);
    (dx * dx + dy * dy).sqrt()
}
