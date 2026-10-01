use std::path::PathBuf;

use tauri_plugin_dialog::FilePath;

use super::{FixedFolderPicker, FolderPicker, chosen_folder};

fn parsed(text: &str) -> FilePath {
    match text.parse() {
        Ok(path) => path,
        Err(never) => match never {},
    }
}

#[test]
fn a_chosen_path_is_the_folder() {
    let folder = PathBuf::from("/home/ana/repos/ramal");
    assert_eq!(
        chosen_folder(Some(FilePath::Path(folder.clone()))),
        Some(folder)
    );
}

#[test]
fn a_file_url_becomes_its_path() {
    assert_eq!(
        chosen_folder(Some(parsed("file:///home/ana/repos/ramal"))),
        Some(PathBuf::from("/home/ana/repos/ramal"))
    );
}

#[test]
fn a_url_that_is_not_a_file_counts_as_cancelling() {
    assert_eq!(
        chosen_folder(Some(parsed("https://example.com/repo"))),
        None
    );
}

#[test]
fn closing_the_dialog_counts_as_cancelling() {
    assert_eq!(chosen_folder(None), None);
}

#[test]
fn the_fixed_picker_answers_its_folder_or_cancels() {
    let folder = PathBuf::from("/tmp/repo");
    let choosing = FixedFolderPicker::choosing(folder.clone()).pick_folder();
    let cancelling = FixedFolderPicker::cancelling().pick_folder();
    assert_eq!(
        tauri::async_runtime::block_on(choosing).ok(),
        Some(Some(folder))
    );
    assert_eq!(tauri::async_runtime::block_on(cancelling).ok(), Some(None));
}
