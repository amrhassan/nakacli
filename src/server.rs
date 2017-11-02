
/// Information about the Nakadi server
pub struct ServerInfo<'a> {
    pub url_base: &'a str,
    pub bearer_token: Option<&'a str>
}
