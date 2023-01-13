use sea_query::Iden;

pub struct Exists;

impl Iden for Exists {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "EXISTS").unwrap();
    }
}

pub struct UpdateMapping;

impl Iden for UpdateMapping {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "update_mapping").unwrap();
    }
}

pub struct UpdateStatistics;

impl Iden for UpdateStatistics {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "update_statistics").unwrap();
    }
}
