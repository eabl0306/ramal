use pretty_assertions::assert_eq;

use super::neutralize;

#[test]
fn every_filter_driver_is_left_without_a_command_once() {
    let names = "filter.lfs.clean\0filter.lfs.smudge\0filter.My.Driver.process\0filter..x\0";

    assert_eq!(
        neutralize(names),
        vec![
            "filter.My.Driver.clean=",
            "filter.My.Driver.smudge=",
            "filter.My.Driver.process=",
            "filter.My.Driver.required=false",
            "filter.lfs.clean=",
            "filter.lfs.smudge=",
            "filter.lfs.process=",
            "filter.lfs.required=false",
        ]
    );
    assert_eq!(neutralize(""), Vec::<String>::new());
}
