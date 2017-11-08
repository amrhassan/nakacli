use global::GlobalParams;

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

impl <'a> ServerInfo<'a> {
    pub fn from_params(global_params: &'a GlobalParams) -> ServerInfo<'a> {
        let authorization =
            if global_params.zign {
                Authorization::Zign
            } else if let Some(bearer_token) = global_params.bearer_token {
                Authorization::BearerToken(bearer_token)
            } else {
                Authorization::None
            };

        ServerInfo {
            url_base: global_params.nakadi_url.unwrap_or("http://localhost"),
            authorization,
        }
    }
}
