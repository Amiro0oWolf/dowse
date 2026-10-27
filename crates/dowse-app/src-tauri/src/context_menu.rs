use std::path::PathBuf;
use std::sync::Mutex;

use tauri::menu::{Menu, MenuEvent, MenuItemBuilder};
use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};
use tauri_plugin_clipboard_manager::ClipboardExt;

const ITEM_OPEN: &str = "ctxmenu-open";
const ITEM_REVEAL: &str = "ctxmenu-reveal";
const ITEM_COPY: &str = "ctxmenu-copy";
const ITEM_CUT: &str = "ctxmenu-cut";
const ITEM_DELETE: &str = "ctxmenu-delete";
const ITEM_COPY_PATHS: &str = "ctxmenu-copy-paths";
const ITEM_COPY_NAMES: &str = "ctxmenu-copy-names";

struct ContextMenuSelection {
    paths: Vec<PathBuf>,
    primary_path: PathBuf,
}

fn emit_file_operation_failure(app: &AppHandle, error: String) {
    if let Err(emit_error) = app.emit("dowse://file-operation-failed", error.clone()) {
        eprintln!("文件操作失败通知界面失败: {emit_error}");
    }
}

#[derive(Default)]
pub struct ContextMenuTarget(Mutex<Option<ContextMenuSelection>>);

impl ContextMenuTarget {
    pub fn new() -> Self {
        Self::default()
    }

    fn set(&self, paths: Vec<PathBuf>, primary_path: PathBuf) {
        *self.0.lock().expect("context menu target mutex poisoned") = Some(ContextMenuSelection {
            paths,
            primary_path,
        });
    }

    fn take(&self) -> Option<ContextMenuSelection> {
        self.0
            .lock()
            .expect("context menu target mutex poisoned")
            .take()
    }
}

#[tauri::command]
pub fn show_result_context_menu(
    app: AppHandle,
    window: WebviewWindow,
    target: State<ContextMenuTarget>,
    paths: Vec<String>,
    primary_path: String,
) -> Result<(), String> {
    if paths.is_empty() || !paths.iter().any(|path| path == &primary_path) {
        return Err("上下文菜单目标无效".to_string());
    }

    let s = crate::i18n::strings();
    let multiple = paths.len() > 1;
    let open_item = MenuItemBuilder::with_id(ITEM_OPEN, s.ctx_open)
        .build(&app)
        .map_err(|e| e.to_string())?;
    let reveal_item = MenuItemBuilder::with_id(ITEM_REVEAL, s.ctx_reveal)
        .build(&app)
        .map_err(|e| e.to_string())?;
    let copy_item = MenuItemBuilder::with_id(ITEM_COPY, s.ctx_copy)
        .build(&app)
        .map_err(|e| e.to_string())?;
    let cut_item = MenuItemBuilder::with_id(ITEM_CUT, s.ctx_cut)
        .build(&app)
        .map_err(|e| e.to_string())?;
    let delete_item = MenuItemBuilder::with_id(ITEM_DELETE, s.ctx_delete)
        .build(&app)
        .map_err(|e| e.to_string())?;
    let copy_paths_label = if multiple {
        s.ctx_copy_paths
    } else {
        s.ctx_copy_path
    };
    let copy_names_label = if multiple {
        s.ctx_copy_names
    } else {
        s.ctx_copy_name
    };
    let copy_paths_item = MenuItemBuilder::with_id(ITEM_COPY_PATHS, copy_paths_label)
        .build(&app)
        .map_err(|e| e.to_string())?;
    let copy_names_item = MenuItemBuilder::with_id(ITEM_COPY_NAMES, copy_names_label)
        .build(&app)
        .map_err(|e| e.to_string())?;

    let menu = if multiple {
        Menu::with_items(
            &app,
            &[
                &open_item,
                &reveal_item,
                &copy_item,
                &cut_item,
                &delete_item,
                &copy_paths_item,
                &copy_names_item,
            ],
        )
    } else {
        Menu::with_items(
            &app,
            &[&open_item, &reveal_item, &copy_paths_item, &copy_names_item],
        )
    }
    .map_err(|e| e.to_string())?;

    target.set(
        paths.into_iter().map(PathBuf::from).collect(),
        PathBuf::from(primary_path),
    );
    if let Err(error) = window.popup_menu(&menu) {
        target.take();
        return Err(error.to_string());
    }
    Ok(())
}

pub fn handle_context_menu_event(window: &tauri::Window, event: MenuEvent) {
    let id = event.id().as_ref();
    if ![
        ITEM_OPEN,
        ITEM_REVEAL,
        ITEM_COPY,
        ITEM_CUT,
        ITEM_DELETE,
        ITEM_COPY_PATHS,
        ITEM_COPY_NAMES,
    ]
    .contains(&id)
    {
        return;
    }

    let app = window.app_handle();
    let Some(selection) = app.state::<ContextMenuTarget>().take() else {
        return;
    };
    let paths: Vec<String> = selection
        .paths
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect();
    let file_ops_window = app.get_webview_window("main");

    match id {
        ITEM_OPEN => {
            for path in paths {
                if let Err(err) = crate::commands::open_file(app.clone(), path) {
                    eprintln!("右键菜单打开文件失败: {err}");
                }
            }
        }
        ITEM_REVEAL => {
            if let Err(err) = crate::commands::reveal_in_folder(
                selection.primary_path.to_string_lossy().into_owned(),
            ) {
                eprintln!("右键菜单定位文件夹失败: {err}");
            }
        }
        ITEM_COPY => {
            let count = paths.len();
            let result = file_ops_window
                .clone()
                .ok_or_else(|| "主窗口不存在".to_string())
                .and_then(|window| crate::commands::copy_files(window, paths));
            match result {
                Ok(()) => {
                    if let Err(err) = app.emit("dowse://files-copied", count) {
                        eprintln!("右键菜单复制文件后通知界面失败: {err}");
                    }
                }
                Err(err) => {
                    eprintln!("右键菜单复制文件失败: {err}");
                    emit_file_operation_failure(&app, err);
                }
            }
        }
        ITEM_CUT => {
            let count = paths.len();
            let result = file_ops_window
                .clone()
                .ok_or_else(|| "主窗口不存在".to_string())
                .and_then(|window| crate::commands::cut_files(window, paths));
            match result {
                Ok(()) => {
                    if let Err(err) = app.emit("dowse://files-cut", count) {
                        eprintln!("右键菜单剪切文件后通知界面失败: {err}");
                    }
                }
                Err(err) => {
                    eprintln!("右键菜单剪切文件失败: {err}");
                    emit_file_operation_failure(&app, err);
                }
            }
        }
        ITEM_DELETE => {
            let deleted_paths = paths.clone();
            let result = file_ops_window
                .clone()
                .ok_or_else(|| "主窗口不存在".to_string())
                .and_then(|window| crate::commands::delete_files(window, paths));
            match result {
                Ok(()) => {
                    if let Err(err) = app.emit("dowse://files-deleted", deleted_paths) {
                        eprintln!("右键菜单删除文件后通知界面失败: {err}");
                    }
                }
                Err(err) => {
                    eprintln!("右键菜单删除文件失败: {err}");
                    emit_file_operation_failure(&app, err);
                }
            }
        }
        ITEM_COPY_PATHS => {
            if let Err(err) = app.clipboard().write_text(paths.join("\r\n")) {
                eprintln!("右键菜单复制路径失败: {err}");
            }
        }
        ITEM_COPY_NAMES => {
            let names = selection
                .paths
                .iter()
                .map(|path| {
                    path.file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .unwrap_or_else(|| path.to_string_lossy().into_owned())
                })
                .collect::<Vec<_>>()
                .join("\r\n");
            if let Err(err) = app.clipboard().write_text(names) {
                eprintln!("右键菜单复制文件名失败: {err}");
            }
        }
        _ => unreachable!("已在函数开头按白名单过滤过"),
    }
}
