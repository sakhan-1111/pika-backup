use std::{convert::Into, io::Read, rc::Rc};

use gio::prelude::*;
use gtk::prelude::*;
use zeroize::Zeroizing;

use crate::borg;
use crate::borg::prelude::*;
use crate::shared;
use crate::shared::*;
use crate::ui;
use crate::ui::builder;
use crate::ui::globals::*;
use crate::ui::prelude::*;
use ui::page_pending;

pub fn new_backup() {
    let ui_new = Rc::new(ui::builder::DialogAddConfig::new());
    load_available_mounts_and_repos(ui_new.clone());
    ui_new
        .password_quality()
        .add_offset_value(&gtk::LEVEL_BAR_OFFSET_LOW, 7.0);
    ui_new
        .password_quality()
        .add_offset_value(&gtk::LEVEL_BAR_OFFSET_HIGH, 5.0);
    ui_new
        .password_quality()
        .add_offset_value(&gtk::LEVEL_BAR_OFFSET_FULL, 3.0);

    ui_new
        .new_backup()
        .set_transient_for(Some(&main_ui().window()));

    let dialog = ui_new.new_backup();
    ui_new.cancel_button().connect_clicked(move |_| {
        dialog.close();
        dialog.hide();
    });

    let ui = ui_new.clone();
    ui_new
        .add_repo_list()
        .connect_row_activated(move |_, row| on_add_repo_list_activated(row, ui.clone()));

    let ui = ui_new.clone();
    ui_new
        .init_repo_list()
        .connect_row_activated(move |_, row| on_init_repo_list_activated(row, &ui));

    let ui = ui_new.clone();
    ui_new
        .password()
        .connect_changed(move |_| on_init_repo_password_changed(&ui));

    let ui = ui_new.clone();
    ui_new
        .add_button()
        .connect_clicked(move |_| on_add_button_clicked(ui.clone()));

    let ui = ui_new.clone();
    ui_new
        .init_button()
        .connect_clicked(move |_| on_init_button_clicked(ui.clone()));

    // refresh ui on mount events
    let monitor = gio::VolumeMonitor::get();

    monitor.connect_mount_added(enclose!((ui_new) move |_, mount| {
        debug!("Mount added");
        load_mount(ui_new.clone(), mount.clone());
    }));

    monitor.connect_mount_removed(enclose!((ui_new) move |_, mount| {
        debug!("Mount removed");
        remove_mount(&ui_new.add_repo_list(), mount.get_root().unwrap().get_uri());
        remove_mount(
            &ui_new.init_repo_list(),
            mount.get_root().unwrap().get_uri(),
        );
    }));

    ui_new.new_backup().show_all();
}

fn load_available_mounts_and_repos(ui: Rc<builder::DialogAddConfig>) {
    debug!("Refreshing list of existing repos");
    let monitor = gio::VolumeMonitor::get();

    ui::utils::clear(&ui.add_repo_list());
    ui::utils::clear(&ui.init_repo_list());

    for mount in monitor.get_mounts() {
        load_mount(ui.clone(), mount);
    }

    debug!("List of existing repos refreshed");
}

fn load_mount(ui: Rc<builder::DialogAddConfig>, mount: gio::Mount) {
    if let Some(mount_point) = mount.get_root().unwrap().get_path() {
        add_mount(&ui.init_repo_list(), &mount, None);
        ui::utils::async_react(
            "check_mount_for_repos",
            move || {
                let mut paths = Vec::new();
                if let Ok(dirs) = mount_point.read_dir() {
                    for dir in dirs {
                        if let Ok(path) = dir {
                            if is_backup_repo(&path.path()) {
                                paths.push(path.path());
                            }
                        }
                    }
                }
                paths
            },
            enclose!((ui) move |paths: Vec<std::path::PathBuf>| {
                for path in paths {
                    trace!("Adding repo to ui '{:?}'", path);
                    add_mount(&ui.add_repo_list(), &mount, Some(&path));
                }
            }),
        );
    }
}

fn on_add_repo_list_activated(row: &gtk::ListBoxRow, ui: Rc<builder::DialogAddConfig>) {
    let name = row.get_widget_name();
    if name == "-add-local" {
        add_local(ui);
    } else if name == "-add-remote" {
        ui.stack().set_visible_child(&ui.add_remote_page());
        ui.add_button().show();
        ui.add_button().grab_default();
    } else {
        let path = match glib::filename_from_uri(&name) {
            Ok((path, _)) => path,
            Err(err) => {
                ui::utils::show_error("URI conversion failed", err);
                return;
            }
        };
        add_repo_config_local(std::path::Path::new(&path), ui);
    }
}

fn add_local(ui: Rc<builder::DialogAddConfig>) {
    ui.new_backup().hide();

    if let Some(path) = ui::utils::folder_chooser_dialog(&gettext("Select existing repository")) {
        ui::page_pending::show(&gettext("Loading backup repository"));
        if is_backup_repo(&path) {
            add_repo_config_local(&path, ui);
        } else {
            ui::utils::dialog_error(gettext(
                "The selected directory is not a valid backup repository.",
            ));
            ui::page_pending::back();
            ui.new_backup().show();
        }
    } else {
        ui.new_backup().show();
    }
}

fn on_add_button_clicked(ui: Rc<builder::DialogAddConfig>) {
    page_pending::show(&gettext("Loading backup repository"));
    ui.new_backup().hide();

    let uri = ui.add_remote_uri().get_text();
    add_repo_config_remote(uri.to_string(), ui);
}

fn on_init_repo_list_activated(row: &gtk::ListBoxRow, ui: &builder::DialogAddConfig) {
    ui.init_dir().set_text(&format!(
        "backup-{}-{}",
        glib::get_host_name()
            .map(|x| x.to_string())
            .unwrap_or_default(),
        glib::get_user_name()
            .and_then(|x| x.into_string().ok())
            .unwrap_or_default()
    ));
    let name = row.get_widget_name();
    if name == "-init-remote" {
        ui.init_location().set_visible_child(&ui.init_remote());
    } else {
        ui.init_location().set_visible_child(&ui.init_local());
        if name != "-init-local" {
            trace!("Setting {} as init_path", &name);
            ui.init_path().set_current_folder_uri(&name);
        } else {
            ui.init_path().grab_focus();
        }
    }
    ui.password_quality().set_value(0.0);
    ui.stack().set_visible_child(&ui.init_page());
    ui.init_button().show();
    ui.init_button().grab_default();
}

fn on_init_button_clicked(ui: Rc<builder::DialogAddConfig>) {
    let encrypted =
        ui.encryption().get_visible_child() != Some(ui.unencrypted().upcast::<gtk::Widget>());

    if encrypted && ui.password().get_text() != ui.password_confirm().get_text() {
        ui::utils::dialog_error(gettext("Entered passwords do not match. Please try again."));
        return;
    }

    let repo = if ui.init_location().get_visible_child()
        == Some(ui.init_local().upcast::<gtk::Widget>())
    {
        let mut path = std::path::PathBuf::new();

        if let Some(init_path) = ui.init_path().get_filename() {
            path.push(init_path);
        } else {
            ui::utils::dialog_error(gettext("You have to select a repository location."));
            return;
        }

        path.push(ui.init_dir().get_text().as_str());
        trace!("Init repo at {:?}", &path);

        BackupRepo::new_from_path(&path)
    } else {
        let url = ui.init_url().get_text().to_string();
        if url.is_empty() {
            ui::utils::dialog_error(gettext("You have to enter a repository location."));
            return;
        }
        BackupRepo::new_from_uri(url)
    };

    page_pending::show(&gettext("Creating backup repository"));
    ui.new_backup().hide();

    let mut borg = borg::BorgOnlyRepo::new(repo.clone());
    let password = Zeroizing::new(ui.password().get_text().as_bytes().to_vec());
    if encrypted {
        borg.set_password(password.clone());
    }

    ui::utils::async_react(
        "borg::init",
        move || borg.init(),
        enclose!((repo, ui, password) move |result: Result<borg::List, _>| match result {
            Err(err) => {
                ui::utils::show_error(&gettext("Failed to initialize repository"), &err);
                page_pending::back();
                ui.new_backup().show();
            }
            Ok(info) => {
                let config = shared::BackupConfig::new(repo.clone(), info, encrypted);

                insert_backup_config(config.clone());
                if encrypted && ui.password_store().get_active() {
                    ui::utils::dialog_catch_err(
                        ui::utils::secret_service_set_password(&config, &password),
                        gettext("Failed to store password."),
                    );
                }
                ui::page_detail::view_backup_conf(&config.id);

                ui.new_backup().close();
            }
        }),
    );
}

fn on_init_repo_password_changed(ui: &builder::DialogAddConfig) {
    let password = ui.password().get_text();
    let score = if let Ok(pw_check) = zxcvbn::zxcvbn(&password, &[]) {
        if pw_check.score() > 3 {
            let n = pw_check.guesses_log10();
            if (12.0..13.0).contains(&n) {
                5
            } else if (13.0..14.0).contains(&n) {
                6
            } else if n > 14.0 {
                7
            } else {
                4
            }
        } else {
            pw_check.score()
        }
    } else {
        0
    };

    ui.password_quality().set_value(score.into());
}

fn remove_mount(list: &gtk::ListBox, root: glib::GString) {
    for list_row in list.get_children() {
        if list_row.get_widget_name() == root {
            list.remove(&list_row);
        }
    }
}

fn add_mount(list: &gtk::ListBox, mount: &gio::Mount, repo: Option<&std::path::Path>) {
    let drive = mount.get_drive();

    let name = repo.map(std::path::Path::to_string_lossy);
    let (row, horizontal_box) =
        ui::utils::add_list_box_row(list, name.as_ref().map(std::borrow::Borrow::borrow), 0);

    row.set_widget_name(&mount.get_root().unwrap().get_uri());

    if let Some(icon) = drive.as_ref().and_then(gio::Drive::get_icon) {
        let img = gtk::Image::from_gicon(&icon, gtk::IconSize::Dialog);
        horizontal_box.add(&img);
    }

    let mut label1: String = mount.get_name().map(Into::into).unwrap_or_default();

    let mut label2: String = drive
        .as_ref()
        .and_then(gio::Drive::get_name)
        .map(Into::into)
        .unwrap_or_default();

    if let Some(root) = mount.get_root() {
        if let Some((fs_size, fs_free)) = ui::utils::fs_usage(&root) {
            label2.push_str(&gettext!(
                ", {free} of {size} available",
                free = ui::utils::hsized(fs_free, 0),
                size = ui::utils::hsized(fs_size, 0)
            ));
        }

        if let Some(mount_path) = root.get_path() {
            if let Some(repo_path) = repo {
                row.set_widget_name(&gio::File::new_for_path(repo_path).get_uri());
                if let Ok(suffix) = repo_path.strip_prefix(mount_path) {
                    if !suffix.to_string_lossy().is_empty() {
                        label1.push_str(&format!(" / {}", suffix.to_string_lossy()));
                    }
                }
            }
        }
    }

    let (vertical_box, _, _) =
        ui::utils::list_vertical_box(Some(label1.as_str()), Some(label2.as_str()));
    horizontal_box.add(&vertical_box);

    list.show_all();
}

fn add_repo_config_local(repo: &std::path::Path, ui: Rc<builder::DialogAddConfig>) {
    let repo = BackupRepo::new_from_path(repo);
    insert_backup_config_encryption_unknown(repo, ui);
}

fn add_repo_config_remote(uri: String, ui: Rc<builder::DialogAddConfig>) {
    let repo = BackupRepo::new_from_uri(uri);
    insert_backup_config_encryption_unknown(repo, ui);
}

fn insert_backup_config_encryption_unknown(
    repo: shared::BackupRepo,
    ui: Rc<builder::DialogAddConfig>,
) {
    ui.new_backup().hide();

    ui::utils::Async::borg_only_repo_suggest_store(
        "borg::peek",
        borg::BorgOnlyRepo::new(repo.clone()),
        |borg| borg.peek(),
        move |result| match result {
            Ok((info, pw_data)) => {
                let encrypted = pw_data
                    .clone()
                    .map(|(password, _)| !password.is_empty())
                    .unwrap_or_default();
                let config = shared::BackupConfig::new(repo.clone(), info, encrypted);
                insert_backup_config(config.clone());
                ui::utils::store_password(&config, &pw_data);
                ui::page_detail::view_backup_conf(&config.id);
                ui.new_backup().close();
            }
            Err(borg_err) => {
                debug!("This repo config is not working");
                ui::utils::show_error(
                    gettext("There was an error with the specified repository"),
                    borg_err,
                );
                ui.new_backup().show();
                page_pending::back();
            }
        },
    )
}

fn insert_backup_config(config: shared::BackupConfig) {
    SETTINGS.update(move |s| {
        s.backups.insert(config.id.clone(), config.clone());
    });

    ui::write_config();
}

/// Checks if a directory is most likely a borg repository. Performed checks are
///
/// - `data/` exists and is a directory
/// - `config` exists and contains the string "[repository]"
pub fn is_backup_repo(path: &std::path::Path) -> bool {
    trace!("Checking path if it is a repo '{}'", &path.display());
    if let Ok(data) = std::fs::File::open(path.join("data")).and_then(|x| x.metadata()) {
        if data.is_dir() {
            if let Ok(mut cfg) = std::fs::File::open(path.join("config")) {
                if let Ok(metadata) = cfg.metadata() {
                    if metadata.len() < 1024 * 1024 {
                        let mut content = String::new();
                        #[allow(unused_must_use)]
                        {
                            cfg.read_to_string(&mut content);
                        }
                        return content.contains("[repository]");
                    }
                }
            }
        }
    };

    false
}