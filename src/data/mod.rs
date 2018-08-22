#[derive(Deserialize, Default, Debug, Clone)]
pub struct PostFlags {
    /// Whether it is safe for work or not.
    #[serde(default)]
    pub nsfw: bool,
}
