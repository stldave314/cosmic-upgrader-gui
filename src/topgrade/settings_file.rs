// SPDX-License-Identifier: GPL-3.0

//! Reading and writing topgrade's own configuration file.
//!
//! This file belongs to the user, not to this application. It is hand-written,
//! it is usually heavily commented — the reference topgrade generates for it
//! runs to 500 lines of explanation, and people keep those comments — and it may
//! well be under version control alongside the rest of someone's dotfiles.
//! Nothing here is allowed to disturb any of that.
//!
//! Two decisions follow from it.
//!
//! The first is that edits go through `toml_edit` rather than a
//! deserialize-modify-serialize round trip. A plain serializer would produce a
//! semantically identical file with every comment stripped, every hand-chosen
//! ordering lost and every blank line collapsed — the first time the user
//! flipped a single toggle. `toml_edit` keeps the document as written and
//! replaces only the value being changed.
//!
//! The second is that saving is atomic: the new contents go to a temporary file
//! alongside the original and are then renamed over it. A crash or a full disk
//! partway through a direct write would leave a truncated file, and a truncated
//! `topgrade.toml` is one topgrade refuses to start with — a bad way to find out
//! the disk was full.
//!
//! ## Which file
//!
//! topgrade reads `$XDG_CONFIG_HOME/topgrade.toml`, falling back to
//! `~/.config/topgrade.toml`, and additionally includes anything in
//! `topgrade.d/` alongside it. The included files are processed *before* the
//! main one, so the main file has the final say on any key it sets — which is
//! why editing it is enough for a change here to take effect, whatever else is
//! present.

use std::io;
use std::path::{Path, PathBuf};

use toml_edit::{Array, DocumentMut, Item, Table, Value as TomlValue};

use super::discover::StepId;
use crate::debug::SETTINGS;
use crate::debug_log;

/// What went wrong reading or writing the file.
#[derive(Clone, Debug)]
pub enum Error {
    /// The user has no home directory to look in.
    NoConfigDirectory,
    Io { path: PathBuf, message: String },
    /// The file exists but is not valid TOML.
    ///
    /// Deliberately not recovered from: the alternative is to start from an
    /// empty document, and saving that would overwrite a file the user has
    /// something in — probably a small typo they would rather be told about.
    Malformed { path: PathBuf, message: String },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoConfigDirectory => write!(f, "no configuration directory could be determined"),
            Self::Io { path, message } => write!(f, "{}: {message}", path.display()),
            Self::Malformed { path, message } => {
                write!(f, "{} is not valid TOML: {message}", path.display())
            }
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

/// A value as the interface deals in it, independent of TOML's representation.
#[derive(Clone, Debug, PartialEq)]
pub enum SettingValue {
    Bool(bool),
    Integer(i64),
    Text(String),
    List(Vec<String>),
}

impl SettingValue {
    fn to_toml(&self) -> TomlValue {
        match self {
            Self::Bool(value) => TomlValue::from(*value),
            Self::Integer(value) => TomlValue::from(*value),
            Self::Text(value) => TomlValue::from(value.as_str()),
            Self::List(values) => {
                let mut array = Array::new();
                for value in values {
                    array.push(value.as_str());
                }
                TomlValue::Array(array)
            }
        }
    }

    fn from_toml(item: &Item) -> Option<Self> {
        let value = item.as_value()?;
        Some(match value {
            TomlValue::Boolean(value) => Self::Bool(*value.value()),
            TomlValue::Integer(value) => Self::Integer(*value.value()),
            TomlValue::String(value) => Self::Text(value.value().clone()),
            TomlValue::Array(array) => Self::List(
                array
                    .iter()
                    .filter_map(|entry| entry.as_str().map(str::to_owned))
                    .collect(),
            ),
            // Floats and datetimes are not among the kinds topgrade's reference
            // describes, so there is no control for them. Rendering one as text
            // would invite an edit that changed its type on save.
            _ => return None,
        })
    }
}

/// topgrade's configuration file, held open for editing.
pub struct SettingsFile {
    path: PathBuf,
    document: DocumentMut,
    /// Whether anything has been changed since it was loaded or last saved.
    /// Lets the interface offer to save only when there is something to save.
    dirty: bool,
}

/// Shows what the file is and whether it needs saving, but not its contents:
/// the document can be tens of kilobytes of the user's own configuration, and
/// spilling it into a diagnostic log would be both unreadable and indiscreet.
impl std::fmt::Debug for SettingsFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SettingsFile")
            .field("path", &self.path)
            .field("dirty", &self.dirty)
            .finish_non_exhaustive()
    }
}

impl SettingsFile {
    /// Where topgrade looks for its configuration.
    pub fn default_path() -> Result<PathBuf> {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
            .ok_or(Error::NoConfigDirectory)?;
        Ok(base.join("topgrade.toml"))
    }

    /// Read the file, or start an empty document if there is not one yet.
    ///
    /// A missing file is the ordinary state for someone who has never
    /// configured topgrade, and is not an error: the document starts empty and
    /// the file is created when something is first saved.
    pub fn load() -> Result<Self> {
        let path = Self::default_path()?;
        Self::load_from(path)
    }

    fn load_from(path: PathBuf) -> Result<Self> {
        let document = match std::fs::read_to_string(&path) {
            Ok(text) => text.parse::<DocumentMut>().map_err(|error| {
                debug_log!(SETTINGS, "{} is malformed: {error}", path.display());
                Error::Malformed {
                    path: path.clone(),
                    message: error.to_string(),
                }
            })?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                debug_log!(SETTINGS, "{} does not exist yet", path.display());
                DocumentMut::new()
            }
            Err(error) => {
                return Err(Error::Io {
                    path,
                    message: error.to_string(),
                })
            }
        };

        Ok(Self {
            path,
            document,
            dirty: false,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Read a setting, or `None` when the file does not set it — in which case
    /// topgrade's own default applies and the interface shows that instead.
    pub fn get(&self, section: &str, key: &str) -> Option<SettingValue> {
        SettingValue::from_toml(self.document.get(section)?.as_table()?.get(key)?)
    }

    /// Set a value, creating the section if it is not there.
    pub fn set(&mut self, section: &str, key: &str, value: &SettingValue) {
        let table = self
            .document
            .entry(section)
            .or_insert_with(|| {
                // A table written now is one the user never typed, so it is
                // given the conventional standalone form rather than being
                // folded into a dotted key.
                let mut table = Table::new();
                table.set_implicit(false);
                Item::Table(table)
            })
            .as_table_mut();

        let Some(table) = table else {
            // The name is taken by something that is not a table — an array of
            // tables, or a scalar. Overwriting it would discard whatever the
            // user put there, so the edit is declined instead.
            debug_log!(SETTINGS, "[{section}] is not a table, not setting {key}");
            return;
        };

        match table.get_mut(key).and_then(Item::as_value_mut) {
            // Replacing the value in place keeps the key's own formatting and
            // any comment trailing it on the same line.
            Some(existing) => {
                let decor = existing.decor().clone();
                let mut replacement = value.to_toml();
                *replacement.decor_mut() = decor;
                *existing = replacement;
            }
            None => table[key] = Item::Value(value.to_toml()),
        }

        self.dirty = true;
        debug_log!(SETTINGS, "set {section}.{key} = {value:?}");
    }

    /// Remove a setting, so topgrade's default applies again.
    ///
    /// This is what "reset to default" does, and it is meaningfully different
    /// from writing the default value in: topgrade is then free to change that
    /// default in a later release, and the user follows it.
    pub fn unset(&mut self, section: &str, key: &str) {
        let Some(table) = self
            .document
            .get_mut(section)
            .and_then(Item::as_table_mut)
        else {
            return;
        };

        if table.remove(key).is_some() {
            self.dirty = true;
            debug_log!(SETTINGS, "unset {section}.{key}");
        }
    }

    /// The steps listed in `misc.disable`.
    ///
    /// This is how a step is turned off, rather than any notion of its own:
    /// `disable` is what topgrade itself reads, and what `--disable` sets, so a
    /// change made here means the same thing as one made on the command line or
    /// by hand in the file.
    pub fn disabled_steps(&self) -> Vec<StepId> {
        match self.get("misc", "disable") {
            Some(SettingValue::List(values)) => values.into_iter().map(StepId::new).collect(),
            _ => Vec::new(),
        }
    }

    /// Turn a step on or off.
    ///
    /// Removing the key entirely once nothing is disabled — rather than leaving
    /// `disable = []` behind — keeps the file looking like one a person would
    /// have written.
    pub fn set_step_enabled(&mut self, step: &StepId, enabled: bool) {
        let mut disabled = self.disabled_steps();
        let present = disabled.iter().any(|id| id == step);

        if enabled && present {
            disabled.retain(|id| id != step);
        } else if !enabled && !present {
            disabled.push(step.clone());
            disabled.sort();
        } else {
            return;
        }

        if disabled.is_empty() {
            self.unset("misc", "disable");
        } else {
            let values = disabled.iter().map(|id| id.as_str().to_owned()).collect();
            self.set("misc", "disable", &SettingValue::List(values));
        }
    }

    pub fn is_step_enabled(&self, step: &StepId) -> bool {
        !self.disabled_steps().iter().any(|id| id == step)
    }

    /// The user-named entries in a free-form section.
    ///
    /// `[commands]`, `[pre_commands]` and `[post_commands]` are maps whose keys
    /// the user invents — `"Emacs Snapshot" = "rm -rf ..."` — so there is no
    /// schema to drive a form from and they are read out as they stand.
    /// Returned sorted so the list does not reshuffle as entries are added.
    pub fn free_form_entries(&self, section: &str) -> Vec<(String, String)> {
        let Some(table) = self.document.get(section).and_then(Item::as_table) else {
            return Vec::new();
        };

        let mut entries: Vec<(String, String)> = table
            .iter()
            .filter_map(|(key, item)| {
                // Only string values are commands; anything else in the table
                // was put there by hand and is left alone rather than shown in
                // a control that would rewrite it as a string on save.
                item.as_str()
                    .map(|value| (key.to_owned(), value.to_owned()))
            })
            .collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        entries
    }

    /// Add or replace one entry in a free-form section.
    ///
    /// An empty name is refused rather than written: TOML would accept `"" =
    /// "..."`, and topgrade would then run a command with no name in its step
    /// list, which is worse than the edit not happening.
    pub fn set_free_form(&mut self, section: &str, name: &str, command: &str) -> bool {
        let name = name.trim();
        if name.is_empty() {
            return false;
        }

        let table = self
            .document
            .entry(section)
            .or_insert_with(|| {
                let mut table = Table::new();
                table.set_implicit(false);
                Item::Table(table)
            })
            .as_table_mut();

        let Some(table) = table else {
            debug_log!(SETTINGS, "[{section}] is not a table, not adding {name}");
            return false;
        };

        table[name] = Item::Value(TomlValue::from(command));
        self.dirty = true;
        debug_log!(SETTINGS, "set {section}.{name:?}");
        true
    }

    /// Remove one entry from a free-form section.
    pub fn remove_free_form(&mut self, section: &str, name: &str) {
        let Some(table) = self.document.get_mut(section).and_then(Item::as_table_mut) else {
            return;
        };
        if table.remove(name).is_some() {
            self.dirty = true;
            debug_log!(SETTINGS, "removed {section}.{name:?}");
        }
    }

    /// Write the file out, replacing it atomically.
    pub fn save(&mut self) -> Result<()> {
        let text = self.document.to_string();

        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| Error::Io {
                path: parent.to_path_buf(),
                message: error.to_string(),
            })?;
        }

        // The temporary file has to share a directory with the target, or the
        // rename would cross a filesystem boundary and stop being atomic.
        let temporary = self.path.with_extension("toml.new");
        std::fs::write(&temporary, &text).map_err(|error| Error::Io {
            path: temporary.clone(),
            message: error.to_string(),
        })?;

        std::fs::rename(&temporary, &self.path).map_err(|error| {
            // Leaving the temporary file behind after a failed rename would be
            // litter next to a file the user cares about.
            let _ = std::fs::remove_file(&temporary);
            Error::Io {
                path: self.path.clone(),
                message: error.to_string(),
            }
        })?;

        self.dirty = false;
        debug_log!(SETTINGS, "saved {} ({} bytes)", self.path.display(), text.len());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn from_text(text: &str) -> SettingsFile {
        SettingsFile {
            path: PathBuf::from("/nonexistent/topgrade.toml"),
            document: text.parse().expect("test fixture should be valid TOML"),
            dirty: false,
        }
    }

    const COMMENTED: &str = r#"# My topgrade configuration.

[misc]
# Do not run as root -- I have been bitten by this before.
allow_root = false

# Steps I never want to run here.
disable = ["emacs", "vim"]

[git]
# Only these, and only over ssh.
repos = ["~/dev"]
"#;

    #[test]
    fn reads_values_of_each_kind() {
        let file = from_text(COMMENTED);
        assert_eq!(file.get("misc", "allow_root"), Some(SettingValue::Bool(false)));
        assert_eq!(
            file.get("git", "repos"),
            Some(SettingValue::List(vec!["~/dev".to_owned()]))
        );
    }

    #[test]
    fn an_unset_key_reads_as_absent() {
        let file = from_text(COMMENTED);
        assert_eq!(file.get("misc", "sudo_command"), None);
        assert_eq!(file.get("nonexistent", "whatever"), None);
    }

    #[test]
    fn editing_a_value_keeps_every_comment() {
        let mut file = from_text(COMMENTED);
        file.set("misc", "allow_root", &SettingValue::Bool(true));
        let out = file.document.to_string();

        assert!(out.contains("allow_root = true"), "{out}");
        for comment in [
            "# My topgrade configuration.",
            "# Do not run as root -- I have been bitten by this before.",
            "# Steps I never want to run here.",
            "# Only these, and only over ssh.",
        ] {
            assert!(out.contains(comment), "lost {comment:?} from:\n{out}");
        }
    }

    #[test]
    fn editing_one_value_leaves_the_others_alone() {
        let mut file = from_text(COMMENTED);
        file.set("misc", "allow_root", &SettingValue::Bool(true));
        let out = file.document.to_string();
        assert!(out.contains(r#"repos = ["~/dev"]"#), "{out}");
        assert!(out.contains(r#"disable = ["emacs", "vim"]"#), "{out}");
    }

    #[test]
    fn a_new_section_is_created_when_needed() {
        let mut file = from_text("[misc]\nallow_root = false\n");
        file.set("containers", "runtime", &SettingValue::Text("podman".to_owned()));
        let out = file.document.to_string();
        assert!(out.contains("[containers]"), "{out}");
        assert!(out.contains(r#"runtime = "podman""#), "{out}");
    }

    #[test]
    fn unsetting_removes_the_key_so_the_default_applies() {
        let mut file = from_text(COMMENTED);
        file.unset("misc", "allow_root");
        assert_eq!(file.get("misc", "allow_root"), None);
        assert!(!file.document.to_string().contains("allow_root ="));
    }

    #[test]
    fn reads_disabled_steps() {
        let file = from_text(COMMENTED);
        let disabled: Vec<_> = file
            .disabled_steps()
            .iter()
            .map(StepId::to_string)
            .collect();
        assert_eq!(disabled, ["emacs", "vim"]);
    }

    #[test]
    fn disabling_a_step_adds_it_to_the_list() {
        let mut file = from_text(COMMENTED);
        file.set_step_enabled(&StepId::new("cargo"), false);
        assert!(!file.is_step_enabled(&StepId::new("cargo")));
        assert!(file.document.to_string().contains("cargo"));
    }

    #[test]
    fn enabling_a_step_removes_it_from_the_list() {
        let mut file = from_text(COMMENTED);
        file.set_step_enabled(&StepId::new("vim"), true);
        assert!(file.is_step_enabled(&StepId::new("vim")));
        let disabled: Vec<_> = file.disabled_steps().iter().map(StepId::to_string).collect();
        assert_eq!(disabled, ["emacs"]);
    }

    #[test]
    fn emptying_the_list_removes_the_key_rather_than_leaving_an_empty_array() {
        let mut file = from_text("[misc]\ndisable = [\"vim\"]\n");
        file.set_step_enabled(&StepId::new("vim"), true);
        let out = file.document.to_string();
        assert!(!out.contains("disable"), "left an empty list behind:\n{out}");
    }

    #[test]
    fn a_step_not_mentioned_is_enabled() {
        let file = from_text(COMMENTED);
        assert!(file.is_step_enabled(&StepId::new("cargo")));
    }

    #[test]
    fn a_redundant_toggle_does_not_dirty_the_file() {
        let mut file = from_text(COMMENTED);
        file.set_step_enabled(&StepId::new("cargo"), true);
        assert!(!file.is_dirty(), "no change was needed, so none should be recorded");
    }

    #[test]
    fn an_edit_marks_the_file_dirty() {
        let mut file = from_text(COMMENTED);
        assert!(!file.is_dirty());
        file.set("misc", "allow_root", &SettingValue::Bool(true));
        assert!(file.is_dirty());
    }

    #[test]
    fn reads_user_named_commands() {
        let file = from_text(
            "[commands]\n\"Emacs Snapshot\" = \"cp -rl a b\"\n\"Sync\" = \"rsync -a x y\"\n",
        );
        let entries = file.free_form_entries("commands");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].0, "Emacs Snapshot");
        assert_eq!(entries[0].1, "cp -rl a b");
    }

    #[test]
    fn adding_a_command_creates_the_section_if_needed() {
        let mut file = from_text("[misc]\nallow_root = false\n");
        assert!(file.set_free_form("commands", "Backup", "restic backup ~"));
        let out = file.document.to_string();
        assert!(out.contains("[commands]"), "{out}");
        assert!(out.contains("restic backup ~"), "{out}");
        assert_eq!(file.free_form_entries("commands").len(), 1);
    }

    #[test]
    fn a_command_name_with_spaces_is_quoted_on_write() {
        let mut file = from_text("");
        file.set_free_form("commands", "Nightly Backup", "restic backup ~");
        let out = file.document.to_string();
        assert!(out.contains("\"Nightly Backup\""), "{out}");
        // And reads back under the same name.
        assert_eq!(file.free_form_entries("commands")[0].0, "Nightly Backup");
    }

    #[test]
    fn an_unnamed_command_is_refused() {
        let mut file = from_text("");
        assert!(!file.set_free_form("commands", "   ", "echo hi"));
        assert!(file.free_form_entries("commands").is_empty());
        assert!(!file.is_dirty());
    }

    #[test]
    fn a_command_can_be_replaced_and_removed() {
        let mut file = from_text("[commands]\n\"Sync\" = \"old\"\n");
        file.set_free_form("commands", "Sync", "new");
        assert_eq!(file.free_form_entries("commands")[0].1, "new");

        file.remove_free_form("commands", "Sync");
        assert!(file.free_form_entries("commands").is_empty());
    }

    #[test]
    fn adding_a_command_keeps_the_rest_of_the_file() {
        let mut file = from_text(COMMENTED);
        file.set_free_form("commands", "Backup", "restic backup ~");
        let out = file.document.to_string();
        assert!(out.contains("# My topgrade configuration."), "{out}");
        assert!(out.contains(r#"disable = ["emacs", "vim"]"#), "{out}");
    }

    #[test]
    fn a_non_string_entry_is_left_alone_rather_than_shown() {
        // Something hand-written that is not a command should not be offered in
        // a text box that would rewrite it as a string.
        let file = from_text("[commands]\n\"Sync\" = \"rsync\"\nweird = 42\n");
        let entries = file.free_form_entries("commands");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "Sync");
    }

    #[test]
    fn a_missing_file_starts_an_empty_document_rather_than_failing() {
        let file = SettingsFile::load_from(PathBuf::from("/nonexistent/dir/topgrade.toml"))
            .expect("a missing file is not an error");
        assert_eq!(file.get("misc", "allow_root"), None);
    }

    #[test]
    fn malformed_toml_is_reported_rather_than_silently_replaced() {
        let directory = std::env::temp_dir().join("cosmic-upgrader-gui-test-malformed");
        std::fs::create_dir_all(&directory).expect("temp dir");
        let path = directory.join("topgrade.toml");
        std::fs::write(&path, "[misc\nallow_root = ").expect("write fixture");

        let result = SettingsFile::load_from(path.clone());
        assert!(matches!(result, Err(Error::Malformed { .. })), "{result:?}");

        let _ = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn saving_round_trips_through_the_filesystem() {
        let directory = std::env::temp_dir().join("cosmic-upgrader-gui-test-save");
        std::fs::create_dir_all(&directory).expect("temp dir");
        let path = directory.join("topgrade.toml");
        std::fs::write(&path, COMMENTED).expect("write fixture");

        let mut file = SettingsFile::load_from(path.clone()).expect("load");
        file.set("misc", "sudo_command", &SettingValue::Text("pkexec".to_owned()));
        file.save().expect("save");
        assert!(!file.is_dirty(), "saving should clear the dirty flag");

        let reloaded = SettingsFile::load_from(path.clone()).expect("reload");
        assert_eq!(
            reloaded.get("misc", "sudo_command"),
            Some(SettingValue::Text("pkexec".to_owned()))
        );
        // The comments have to survive a real write, not just an in-memory edit.
        let text = std::fs::read_to_string(&path).expect("read back");
        assert!(text.contains("# My topgrade configuration."), "{text}");

        // Nothing should be left alongside the file after an atomic replace.
        assert!(!directory.join("topgrade.toml.new").exists());

        let _ = std::fs::remove_dir_all(&directory);
    }
}
