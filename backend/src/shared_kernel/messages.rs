/// The default messages one part of the application ships: a JSON object of
/// ICU `MessageFormat` texts by key, in each language. Each module declares
/// its own and the composition root collects them, so a module adds a key
/// without editing anyone else's file, as it does with its permissions.
#[derive(Debug, Clone, Copy)]
pub struct Messages {
    /// Who ships them, named when a key is missing or shipped twice.
    pub owner: &'static str,
    pub pl: &'static str,
    pub en: &'static str,
}
