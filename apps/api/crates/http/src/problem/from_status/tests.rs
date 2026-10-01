use salvo::prelude::StatusCode;

use super::problem_for_status;

fn code(status: StatusCode) -> (u16, String) {
    let problem = problem_for_status(status);
    (problem.status, problem.code)
}

#[test]
fn a_missing_route_is_a_404_with_its_own_code() {
    let problem = problem_for_status(StatusCode::NOT_FOUND);
    assert_eq!(problem.status, 404);
    assert_eq!(problem.code, "route_not_found");
    assert_eq!(problem.kind, "https://ramal.dev/problems/not-found");
    assert_eq!(problem.detail, "La ruta solicitada no existe en esta API.");
}

#[test]
fn each_transport_status_has_a_stable_code() {
    assert_eq!(
        code(StatusCode::METHOD_NOT_ALLOWED),
        (405, "method_not_allowed".to_owned())
    );
    assert_eq!(
        code(StatusCode::PAYLOAD_TOO_LARGE),
        (413, "payload_too_large".to_owned())
    );
    assert_eq!(
        code(StatusCode::UNSUPPORTED_MEDIA_TYPE),
        (415, "unsupported_media_type".to_owned())
    );
    assert_eq!(
        code(StatusCode::TOO_MANY_REQUESTS),
        (429, "too_many_requests".to_owned())
    );
    assert_eq!(
        code(StatusCode::SERVICE_UNAVAILABLE),
        (503, "service_unavailable".to_owned())
    );
}

#[test]
fn other_statuses_keep_their_number_with_a_generic_type() {
    assert_eq!(code(StatusCode::GONE), (410, "bad_request".to_owned()));
    assert_eq!(
        code(StatusCode::BAD_GATEWAY),
        (502, "internal_error".to_owned())
    );
}
