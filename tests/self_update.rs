use serde_json::json;
use skylab_rust::self_update;
#[test]
fn update_session_accepts_three_documents_and_keeps_the_active_one() {
    let session = json!({"active":2,"documents":[{"id":1,"state":{"project":null}},{"id":2,"state":{"project":null}},{"id":3,"state":{"project":null}}]});
    self_update::validate_session(&session).unwrap();
}
#[test]
fn invalid_update_sessions_are_refused() {
    for session in [
        json!({}),
        json!({"active":1,"documents":[]}),
        json!({"active":2,"documents":[{"id":1,"state":{"project":null}}]}),
        json!({"active":1,"documents":[{"id":1,"state":{}},{"id":1,"state":{}}]}),
        json!({"active":1,"documents":[{"id":1,"state":{"project":{"lines":[]}}}]}),
    ] {
        assert!(self_update::validate_session(&session).is_err());
    }
}

#[test]
fn home_screen_can_restart_after_update_without_creating_a_project() {
    self_update::validate_session(&json!({"active":null,"documents":[]})).unwrap();
}
