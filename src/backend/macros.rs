#[macro_export]
macro_rules! impl_backend_constants {
    ($name:ident) => {
        fn get_binary(&self) -> Text {
            BINARY
        }

        fn get_section(&self) -> Text {
            SECTION
        }

        fn get_switches_install(&self) -> Switches {
            SWITCHES_INSTALL
        }

        fn get_switches_remove(&self) -> Switches {
            SWITCHES_REMOVE
        }

        fn get_managed_packages(&self) -> &HashSet<Package> {
            &self.packages
        }
    };
}
