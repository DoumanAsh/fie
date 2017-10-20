
#[inline]
pub fn message(message: &str, tags: &Option<Vec<String>>) -> String {
    match tags {
        &Some(ref tags) => format!("{}\n{}", message, tags.join(" ")),
        &None => message.to_string()
    }
}
