use std::str::FromStr;

use crate::ui::prelude::*;
use gio::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, glib::Enum, Default)]
#[enum_type(name = "PkSetupKind")]
pub enum SetupAction {
    #[default]
    Init,
    AddExisting,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, glib::Enum, Default)]
#[enum_type(name = "PkSetupRepoKind")]
pub enum SetupLocationKind {
    #[default]
    Local,
    Remote,
}

#[derive(Debug, Clone, glib::Boxed)]
#[boxed_type(name = "PkSetupRepoLocation", nullable)]
pub enum SetupRepoLocation {
    Local(gio::File),
    Remote(String),
}

impl SetupRepoLocation {
    /// Takes a [`gio::File`]. Any checks will be performed when creating the repo config
    pub fn from_file(file: gio::File) -> Self {
        Self::Local(file)
    }

    /// Parse borg URIs and canonicalize them to the `ssh://` syntax.
    /// All other URIs are being taken verbatim
    pub fn parse_url(input: String) -> std::result::Result<Self, String> {
        let url = if !input.contains("://") {
            if let Some((target, path)) = input.split_once(':') {
                let path_begin = path.chars().next();

                let url_path = if path_begin == Some('~') {
                    format!("/{path}")
                } else if path_begin != Some('/') {
                    format!("/./{path}")
                } else {
                    path.to_string()
                };

                format!("ssh://{target}{url_path}")
            } else {
                return Err(gettext("Incomplete URL or borg syntax"));
            }
        } else {
            input
        };

        match glib::Uri::parse(&url, glib::UriFlags::NONE) {
            Ok(uri) => {
                if uri.path().is_empty() {
                    return Err(gettext("The remote location must have a specified path."));
                }

                if uri.scheme() == "ssh" {
                    Ok(Self::Remote(url))
                } else {
                    Ok(Self::Local(gio::File::for_uri(&url)))
                }
            }
            Err(err) => Err(gettextf("Invalid remote location: “{}”", &[err.message()])),
        }
    }
}

impl std::fmt::Display for SetupRepoLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SetupRepoLocation::Local(file) => write!(f, "{}", file.uri()),
            SetupRepoLocation::Remote(uri) => write!(f, "{}", uri),
        }
    }
}

#[derive(Debug, Default, Clone, glib::Boxed)]
#[boxed_type(name = "PkSetupCommandLineArgs", nullable)]
pub struct SetupCommandLineArgs(Vec<String>);

impl FromStr for SetupCommandLineArgs {
    type Err = crate::ui::error::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self(crate::ui::utils::borg::parse_borg_command_line_args(
            s,
        )?))
    }
}

impl SetupCommandLineArgs {
    pub const NONE: Self = Self(Vec::new());

    pub fn into_inner(self) -> Vec<String> {
        self.0
    }
}
