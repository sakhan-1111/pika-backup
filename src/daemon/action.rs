use super::prelude::*;

use super::dbus;

pub struct StartBackup;

impl StartBackup {
    const NAME: &'static str = "startbackup";

    pub fn action() -> gio::SimpleAction {
        let action = gio::SimpleAction::new(Self::NAME, Some(&glib::VariantTy::STRING));
        action.connect_activate(|_, config_id| {
            if let Some(config_id) = config_id.and_then(|v| v.str().map(|x| x.to_string())) {
                glib::MainContext::default().spawn(async move {
                    dbus::PikaBackup::start_backup(&ConfigId::new(config_id))
                        .await
                        .handle(gettext("Failed to start backup from daemon"));
                });
            } else {
                error!("Invalid parameter for {}: {:?}", Self::NAME, config_id);
            }
        });
        action
    }

    pub fn name() -> String {
        format!("app.{}", Self::NAME)
    }
}
