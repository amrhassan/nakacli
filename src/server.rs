
/// Information about the Nakadi server
pub struct ServerInfo<'a> {
    pub url_base: &'a str,
    pub authorization: Authorization<'a>
}

pub enum Authorization<'a> {
    None,
    BearerToken(&'a str),
    Zign
}
