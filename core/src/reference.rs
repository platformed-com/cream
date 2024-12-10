use serde::{Deserialize, Serialize, Serializer};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Reference(ReferenceInner);

impl Reference {
    pub fn new_relative(relative: &str) -> Self {
        Self(ReferenceInner::Relative(RelativeReference(
            relative.to_string(),
        )))
    }
    pub fn new_absolute(absolute: &str) -> Self {
        Self(ReferenceInner::Absolute(absolute.to_string()))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
enum ReferenceInner {
    Absolute(String),
    #[serde(skip_deserializing)]
    Relative(RelativeReference),
}

#[cfg(feature = "tokio")]
tokio::task_local! {
    pub static BASE_URL: String;
}

#[derive(Debug, Clone)]
struct RelativeReference(String);

impl Serialize for RelativeReference {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[cfg(feature = "tokio")]
        let absolute_url = BASE_URL
            .try_with(|base_url| format!("{}{}", base_url, self.0))
            .unwrap_or(self.0.clone());
        #[cfg(not(feature = "tokio"))]
        let absolute_url = self.0.clone();
        serializer.serialize_str(&absolute_url)
    }
}
