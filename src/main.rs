#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use native_windows_gui as nwg;
use nwg::{error_message, simple_message, ControlHandle};
use std::path::PathBuf;

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    let settings = if let Ok(txt) = std::fs::read_to_string("settings.txt") {
        let mut settings = txt.split('\n');
        let Some(source) = settings.next().map(PathBuf::from) else {
            error_message("Error", "No source path in settings.txt");
            return;
        };
        let Some(target) = settings.next().map(PathBuf::from) else {
            error_message("Error", "No target path in settings.txt");
            return;
        };

        (source, target)
    } else {
        let mut dialog = nwg::FileDialog::default();
        nwg::FileDialog::builder()
            .title("Select soruce folder")
            .action(nwg::FileDialogAction::OpenDirectory)
            .build(&mut dialog)
            .unwrap();
        dialog.run::<ControlHandle>(None);
        let Ok(source) = dialog.get_selected_item().map(PathBuf::from) else {
            error_message("Error", "No source folder selected");
            return
        };

        nwg::FileDialog::builder()
            .title("Select target folder")
            .action(nwg::FileDialogAction::OpenDirectory)
            .build(&mut dialog)
            .unwrap();
        dialog.run::<ControlHandle>(None);
        let Ok(target) = dialog.get_selected_item().map(PathBuf::from) else {
            error_message("Error", "No source folder selected");
            return
        };
        std::fs::write(
            "settings.txt",
            format!(
                "{}\n{ }",
                source.to_str().unwrap(),
                target.to_str().unwrap()
            ),
        )
        .unwrap();

        (source, target)
    };
    let source = settings.0;
    let target = settings.1;

    let mut cnt = 0;
    let mut skipped_exist = 0;
    let mut skipped_no_image = 0;

    for entry in walkdir::WalkDir::new(&source).into_iter().flatten() {
        if entry.file_type().is_file() {
            let source_file_path = entry.path();
            let relative_path = source_file_path.strip_prefix(&source).unwrap();
            let target_file_name = relative_path.iter().fold(String::new(), |a, b| {
                if a.is_empty() {
                    return b.to_string_lossy().to_string();
                }
                format!("{a} - {}", b.to_string_lossy())
            });
            let target_file_path = target.clone().join(target_file_name);

            if !source_file_path
                .extension()
                .map(|ext| ext == "png")
                .unwrap_or(false)
            {
                skipped_no_image += 1;
                continue;
            }

            if target_file_path.exists() {
                skipped_exist += 1;
                continue;
            }

            if let Err(e) = std::os::windows::fs::symlink_file(source_file_path, target_file_path) {
                println!("{:?}", e);
                error_message("Error", &format!("Failed to create symlink:\n\n{:?}", e));
                return;
            }

            cnt += 1;
        }
    }

    simple_message(
        "Success",
        &format!(
            "{:?}\n↓\n{:?}:\n\nSuccessfully created {} symlinks\nSkipped {} non-image files\nSkipped {} existing files",
            source, target, cnt, skipped_no_image, skipped_exist
        ),
    );
}
