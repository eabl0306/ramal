use git_rudder_protocol::METHODS;

use super::HANDLERS;

#[test]
fn every_declared_method_has_exactly_one_handler() {
    let declared: Vec<&str> = METHODS.iter().map(|method| method.name).collect();
    let handled: Vec<&str> = HANDLERS.iter().map(|(name, _)| *name).collect();

    assert_eq!(
        handled, declared,
        "the table follows define_methods! in order"
    );
}
