use adw::prelude::*;
use ui::prelude::*;

use crate::config;
use crate::ui;
use ui::builder::DialogArchivePrefix;

pub fn run(config: &config::Backup) {
    let ui = DialogArchivePrefix::new();

    ui.archive_prefix()
        .set_text(&config.archive_prefix.to_string());
    ui.archive_prefix().grab_focus();

    ui.dialog().set_transient_for(Some(&main_ui().window()));

    let config_id = config.id.clone();
    ui.ok()
        .connect_clicked(clone!(@weak ui, @strong config_id =>
            move |_| Handler::new().error_transient_for(ui.dialog()).handle_sync(on_ok(ui, config_id.clone()))));

    ui.dialog().show();

    // ensure lifetime until window closes
    let mutex = std::sync::Mutex::new(Some(ui.clone()));
    ui.dialog().connect_close_request(move |_| {
        *mutex.lock().unwrap() = None;
        gtk::Inhibit(false)
    });
}

fn on_ok(ui: DialogArchivePrefix, config_id: ConfigId) -> Result<()> {
    let new_prefix = ui.archive_prefix().text();

    BACKUP_CONFIG.update_result(enclose!(
        (config_id, new_prefix) | config | {
            config
                .get_result_mut(&config_id)?
                .set_archive_prefix(
                    config::ArchivePrefix::new(&new_prefix),
                    BACKUP_CONFIG.load().iter(),
                )
                .err_to_msg(gettext("Invalid Archive Prefix"))?;
            Ok(())
        }
    ))?;

    ui::write_config()?;
    ui::page_archives::update_info(BACKUP_CONFIG.load().active()?);
    ui.dialog().close();

    Ok(())
}
