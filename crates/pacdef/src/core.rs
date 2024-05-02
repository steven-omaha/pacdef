use anyhow::Result;
use const_format::formatcp;

use crate::prelude::*;
use crate::review::review;
use crate::ui::get_user_confirmation;

impl MainArguments {
    /// Run the action that was provided by the user as first argument.
    ///
    /// For convenience sake, all called functions take a `&self` argument, even if
    /// these are not strictly required.
    ///
    /// # Errors
    ///
    /// This function propagates errors from the underlying functions.
    pub fn run(self, groups: &Groups, config: &Config) -> Result<()> {
        match self.subcommand {
            MainSubcommand::Clean(clean) => clean.run(groups, config),
            MainSubcommand::Review(review) => review.run(groups, config),
            MainSubcommand::Sync(sync) => sync.run(groups, config),
            MainSubcommand::Unmanaged(unmanaged) => unmanaged.run(groups, config),
            MainSubcommand::Version(version) => version.run(config),
        }
    }
}

impl VersionArguments {
    /// If the crate was compiled from git, return `pacdef, <version> (<hash>)`.
    /// Otherwise return `pacdef, <version>`.
    fn run(self, _: &Config) -> Result<()> {
        /// If the crate was compiled from git, return `<version> (<hash>)`. Otherwise
        /// return `<version>`.
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        const HASH: &str = env!("GIT_HASH");

        let version = if HASH.is_empty() {
            VERSION
        } else {
            formatcp!("{VERSION} ({HASH})")
        };

        println!("pacdef, version: {}\n", version);

        Ok(())
    }
}

impl CleanPackageAction {
    fn run(self, groups: &Groups, config: &Config) -> Result<()> {
        let unmanaged = PackagesIds::unmanaged(groups, config)?;

        if unmanaged.is_empty() {
            println!("nothing to do");
            return Ok(());
        }

        println!("Would remove the following packages:\n\n{unmanaged}\n");

        if self.no_confirm {
            println!("proceeding without confirmation");
        } else if !get_user_confirmation()? {
            return Ok(());
        }

        let packages_to_remove = PackagesRemove::from_packages_ids_defaults(&unmanaged);

        packages_to_remove.remove(self.no_confirm, config)
    }
}

impl ReviewPackageAction {
    fn run(self, _: &Groups, _: &Config) -> Result<()> {
        review()
    }
}

impl SyncPackageAction {
    fn run(self, groups: &Groups, config: &Config) -> Result<()> {
        let missing = PackagesIds::missing(groups, config)?;

        if missing.is_empty() {
            println!("nothing to do");
            return Ok(());
        }

        println!("Would install the following packages:\n\n{missing}\n");

        if self.no_confirm {
            println!("proceeding without confirmation");
        } else if !get_user_confirmation()? {
            return Ok(());
        }

        let packages_to_install = PackagesInstall::from_packages_ids_defaults(&missing);

        packages_to_install.install(self.no_confirm, config)
    }
}

impl UnmanagedPackageAction {
    fn run(self, groups: &Groups, config: &Config) -> Result<()> {
        let unmanaged = PackagesIds::unmanaged(groups, config)?;

        if unmanaged.is_empty() {
            println!("no unmanaged packages");
        } else {
            println!("unmanaged packages:\n\n{unmanaged}");
        }

        Ok(())
    }
}
