pub struct PlatformProfile {
    pub source: String,
    pub location: std::path::PathBuf,
    pub profile: String,
    pub choices: Vec<String>,
} impl PlatformProfile {
    pub fn set_profile(&mut self, wanted: String) {
        if self.do_wanted_profile_exists_in_the_first_place(&wanted) {
            if let Ok(()) = crate::sysfs::boop(self.location.clone().join("profile"), wanted.clone()) {
                log::info!("Platform-profile ({}) set to \"{}\"", self.source, &wanted);
                self.profile = wanted;
            } else { log::error!("Unable to set platform-profile: {}, {}", self.source, wanted) }
        } else {
            log::error!("The wanted profile ({}) doesn't exists for this platform ({}). Here's the available choices: {:?}", wanted, self.source, self.choices);
        }
    }

    fn do_wanted_profile_exists_in_the_first_place(&self, wanted: &String) -> bool { self.choices.contains(wanted) }
}
