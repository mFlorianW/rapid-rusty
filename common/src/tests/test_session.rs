use crate::{
    session::Session,
    test_helper::session::{get_session, get_session_as_json},
};

#[test]
pub fn deserialize_session_from_json() {
    let session = Session::from_json(get_session_as_json())
        .unwrap_or_else(|e| panic!("Failed to deserialize the raw json. Reason: {e}"));
    assert_eq!(session, get_session());
}

#[test]
pub fn serialize_session_to_json() {
    let session = Session::to_json(&get_session())
        .unwrap_or_else(|e| panic!("Failed to serialize session to json. Reason {e}"));
    assert_eq!(
        serde_json::from_str::<Session>(&session).unwrap(),
        serde_json::from_str(get_session_as_json()).unwrap()
    );
}
