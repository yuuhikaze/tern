use crate::controller::Profile;

pub struct Converter<'a> {
    profile: &'a Profile,
}

impl<'a> Converter<'a> {
    pub fn new(profile: &'a Profile) -> Self {
        Self { profile }
    }
}
