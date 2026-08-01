// SPDX-License-Identifier: GPL-3.0

//! Grouping steps into the sections shown down the side of the window.
//!
//! This is the one part of [`crate::topgrade`] that topgrade cannot supply.
//! It has no notion of categories — its steps are a flat list of 174
//! identifiers — so the grouping is written here.
//!
//! That makes this the only place where a hard-coded list of steps appears, and
//! it is worth being precise about what that does and does not decide. This
//! table affects *presentation only*: which heading a step is filed under. It
//! does not decide whether a step exists, whether it is shown, or whether it can
//! be run — all three of those come from [`discover`](super::discover) and
//! [`probe`](super::probe), which ask the binary. A step this table has never
//! heard of, because it was added to topgrade after this was last touched, is
//! still discovered, still probed, still displayed and still runnable; it simply
//! appears under [`Category::Other`] instead of a more specific heading.
//!
//! So the failure mode of letting this go stale is a step filed under a vaguer
//! heading than it deserves, which is a cosmetic problem the user can see and
//! work around, rather than a step silently disappearing from the interface.
//! Categories with no discovered steps are not shown at all, so the sidebar
//! reflects the machine it is running on.

use std::collections::BTreeMap;

use super::discover::StepId;
use crate::fl;

/// A heading in the sidebar.
///
/// Ordered as declared, which is the order they appear in: the things most
/// people came for first, the more specialised further down, and the catch-all
/// last.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Category {
    System,
    Applications,
    Containers,
    Development,
    Editors,
    Repositories,
    Shell,
    AiTools,
    Cloud,
    Desktop,
    Custom,
    Other,
}

impl Category {
    /// Every category, in sidebar order.
    pub const ALL: [Self; 12] = [
        Self::System,
        Self::Applications,
        Self::Containers,
        Self::Development,
        Self::Editors,
        Self::Repositories,
        Self::Shell,
        Self::AiTools,
        Self::Cloud,
        Self::Desktop,
        Self::Custom,
        Self::Other,
    ];

    /// Localized heading.
    pub fn label(self) -> String {
        match self {
            Self::System => fl!("category-system"),
            Self::Applications => fl!("category-applications"),
            Self::Containers => fl!("category-containers"),
            Self::Development => fl!("category-development"),
            Self::Editors => fl!("category-editors"),
            Self::Repositories => fl!("category-repositories"),
            Self::Shell => fl!("category-shell"),
            Self::AiTools => fl!("category-ai-tools"),
            Self::Cloud => fl!("category-cloud"),
            Self::Desktop => fl!("category-desktop"),
            Self::Custom => fl!("category-custom"),
            Self::Other => fl!("category-other"),
        }
    }

    /// Symbolic icon shown beside the heading.
    ///
    /// All of these resolve through the COSMIC icon theme's inheritance chain
    /// (COSMIC → Pop → Adwaita → hicolor), so they are present on a plain
    /// install rather than depending on an icon set we would have to ship.
    pub fn icon_name(self) -> &'static str {
        match self {
            Self::System => "computer-symbolic",
            Self::Applications => "view-app-grid-symbolic",
            Self::Containers => "package-x-generic-symbolic",
            Self::Development => "applications-engineering-symbolic",
            Self::Editors => "accessories-text-editor-symbolic",
            Self::Repositories => "folder-remote-symbolic",
            Self::Shell => "utilities-terminal-symbolic",
            Self::AiTools => "applications-science-symbolic",
            Self::Cloud => "network-server-symbolic",
            Self::Desktop => "preferences-desktop-symbolic",
            Self::Custom => "applications-utilities-symbolic",
            Self::Other => "application-x-addon-symbolic",
        }
    }
}

/// The configuration sections that belong with a category.
///
/// topgrade's configuration is grouped by tool, and its own grouping does not
/// line up with the categories here — `[cargo]`, `[go]` and `[npm]` are all
/// Development, while `[misc]` sits behind System. This maps one onto the other
/// so a category's settings can be reached from the category itself, rather
/// than by hunting down a long configuration page for the section that happens
/// to relate to what is on screen.
///
/// Names that the installed topgrade does not have are filtered out by the
/// caller against the discovered schema, so a section that only exists in a
/// newer or older release simply does not appear.
pub fn config_sections(category: Category) -> &'static [&'static str] {
    match category {
        Category::System => &["misc", "linux", "firmware", "mandb", "pkgfile", "lensfun"],
        Category::Applications => &["flatpak", "brew"],
        Category::Containers => &["containers", "distrobox"],
        Category::Development => &[
            "python", "conda", "composer", "cargo", "rustup", "go", "npm", "yarn", "deno",
            "viteplus", "julia", "zigup", "pixi", "mise", "flutter",
        ],
        Category::Editors => &["vim", "vscode", "doom"],
        Category::Repositories => &["git", "chezmoi"],
        Category::Cloud => &["vagrant"],
        Category::Custom => &["commands", "pre_commands", "post_commands"],
        // Nothing in topgrade's configuration corresponds to these.
        Category::Shell | Category::AiTools | Category::Desktop | Category::Other => &[],
    }
}

/// Which heading a step belongs under.
///
/// Written as a match rather than a lookup table so the grouping reads as a
/// list of what belongs together, and so the compiler turns it into a jump
/// table rather than a scan.
pub fn category_of(id: &StepId) -> Category {
    match id.as_str() {
        // The operating system and the things that keep it healthy.
        "system" | "config_update" | "restarts" | "firmware" | "audit" | "clam_av_db" | "mandb"
        | "pkgfile" | "lensfun" | "maza" | "certbot" | "auto_cpufreq" | "sera" | "falconf"
        | "self_update" | "wsl" | "wsl_update" | "microsoft_office" | "powershell" => {
            Category::System
        }

        // Ways of installing end-user applications.
        "flatpak" | "snap" | "am" | "app_man" | "gearlever" | "soar" | "deb_get" | "pacstall"
        | "pacdef" | "dkp_pacman" | "brew_cask" | "brew_formula" | "zerobrew" | "macports"
        | "mas" | "chocolatey" | "scoop" | "winget" | "microsoft_store" | "pkg" | "pkgin"
        | "guix" | "nix" | "nix_helper" | "home_manager" | "lure" | "protonplus" | "protonup"
        | "waydroid" | "bin" | "stew" | "install_release" | "getnf" | "xcodes" | "sparkle" => {
            Category::Applications
        }

        "containers" | "distrobox" | "toolbx" | "colima" => Category::Containers,

        // Language toolchains, package managers and build tooling.
        "cargo" | "rustup" | "go" | "node" | "npm" | "yarn" | "pnpm" | "bun" | "bun_packages"
        | "volta_packages" | "deno" | "vite_plus" | "pip3" | "pip_review" | "pip_review_local"
        | "pipupgrade" | "pipx" | "pipxu" | "poetry" | "uv" | "conda" | "mamba" | "pixi"
        | "rye" | "pyenv" | "gem" | "ruby_gems" | "composer" | "dotnet" | "julia" | "juliaup"
        | "elan" | "ghcup" | "stack" | "haxelib" | "opam" | "raco" | "typst" | "zigup" | "zvm"
        | "flutter" | "sdkman" | "asdf" | "mise" | "vcpkg" | "platformio_core" | "miktex"
        | "tlmgr" | "choosenim" | "bob" | "pkgit" | "rtcl" | "pi" | "jetpack" | "aqua" => {
            Category::Development
        }

        // Editors, IDEs and the plugins they carry.
        "vim" | "emacs" | "helix" | "helix_db" | "kakoune" | "micro" | "atom" | "yazi" | "doom"
        | "vscode" | "vscode_insiders" | "vscodium" | "vscodium_insiders" | "cursor"
        | "windsurf" | "antigravity" | "android_studio" | "jetbrains_toolbox"
        | "jetbrains_aqua" | "jetbrains_clion" | "jetbrains_datagrip" | "jetbrains_dataspell"
        | "jetbrains_gateway" | "jetbrains_goland" | "jetbrains_idea" | "jetbrains_mps"
        | "jetbrains_phpstorm" | "jetbrains_pycharm" | "jetbrains_rider"
        | "jetbrains_rubymine" | "jetbrains_rustrover" | "jetbrains_webstorm" => Category::Editors,

        "git_repos" | "myrepos" | "fossil" | "remotes" | "yadm" | "chezmoi" | "rcm" => {
            Category::Repositories
        }

        "shell" | "sheldon" | "tmux" | "tpack" | "atuin" | "pearl" | "tldr" => Category::Shell,

        "claude_code" | "claude_code_plugins" | "codex" | "opencode" | "cursor_agent"
        | "ollama" | "skills" => Category::AiTools,

        "gcloud" | "helm" | "krew" | "vagrant" => Category::Cloud,

        "gnome_shell_extensions" | "cinnamon_spices" | "hyprpm" | "spicetify"
        | "github_cli_extensions" => Category::Desktop,

        "custom_commands" => Category::Custom,

        // Anything topgrade has gained since this table was last reviewed.
        _ => Category::Other,
    }
}

/// Group discovered steps under their headings.
///
/// Only categories that actually received a step appear in the result, which is
/// what keeps the sidebar showing the machine in front of the user rather than
/// every heading this application knows how to draw.
pub fn group(steps: &[StepId]) -> BTreeMap<Category, Vec<StepId>> {
    let mut grouped: BTreeMap<Category, Vec<StepId>> = BTreeMap::new();
    for id in steps {
        grouped.entry(category_of(id)).or_default().push(id.clone());
    }
    for steps in grouped.values_mut() {
        steps.sort();
    }
    grouped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_category_with_settings_names_them() {
        assert!(config_sections(Category::Containers).contains(&"containers"));
        assert!(config_sections(Category::Development).contains(&"cargo"));
    }

    #[test]
    fn a_category_with_no_settings_names_none() {
        assert!(config_sections(Category::Other).is_empty());
        assert!(config_sections(Category::AiTools).is_empty());
    }

    #[test]
    fn every_category_is_answered_for() {
        // A `match` with no wildcard would catch this at compile time, but the
        // catch-all arm groups several categories, so it is checked here.
        for category in Category::ALL {
            let _ = config_sections(category);
        }
    }

    #[test]
    fn files_a_known_step_under_its_heading() {
        assert_eq!(category_of(&StepId::new("cargo")), Category::Development);
        assert_eq!(category_of(&StepId::new("flatpak")), Category::Applications);
        assert_eq!(category_of(&StepId::new("containers")), Category::Containers);
    }

    #[test]
    fn an_unknown_step_is_still_categorised_rather_than_lost() {
        // The point of the fallback: a step from a topgrade newer than this
        // table still reaches the interface.
        assert_eq!(
            category_of(&StepId::new("some_future_package_manager")),
            Category::Other
        );
    }

    #[test]
    fn grouping_omits_categories_with_no_steps() {
        let steps = [StepId::new("cargo"), StepId::new("rustup")];
        let grouped = group(&steps);
        assert_eq!(grouped.len(), 1);
        assert!(grouped.contains_key(&Category::Development));
    }

    #[test]
    fn grouping_keeps_every_step_it_was_given() {
        let steps = [
            StepId::new("cargo"),
            StepId::new("flatpak"),
            StepId::new("a_brand_new_step"),
        ];
        let grouped = group(&steps);
        let total: usize = grouped.values().map(Vec::len).sum();
        assert_eq!(total, steps.len(), "a step went missing: {grouped:?}");
    }

    #[test]
    fn steps_within_a_category_are_ordered() {
        let steps = [
            StepId::new("rustup"),
            StepId::new("cargo"),
            StepId::new("deno"),
        ];
        let grouped = group(&steps);
        let development = &grouped[&Category::Development];
        assert_eq!(
            development.iter().map(StepId::as_str).collect::<Vec<_>>(),
            ["cargo", "deno", "rustup"]
        );
    }
}
