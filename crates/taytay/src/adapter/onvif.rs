use crate::{TaytayError, model::SourceId};
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OnvifDevice {
    pub endpoint: String,
    pub name: Option<String>,
    pub profile_tokens: Vec<String>,
}
pub fn parse_probe(xml: &str) -> Result<OnvifDevice, TaytayError> {
    let endpoint = between(xml, "<XAddrs>", "</XAddrs>")
        .ok_or_else(|| TaytayError::Protocol("ONVIF probe has no XAddrs".into()))?;
    let name = between(xml, "<Name>", "</Name>");
    let profile_tokens = xml
        .split("token=\"")
        .skip(1)
        .filter_map(|x| x.split('"').next())
        .map(str::to_owned)
        .collect();
    Ok(OnvifDevice {
        endpoint: endpoint.to_owned(),
        name: name.map(str::to_owned),
        profile_tokens,
    })
}
fn between<'a>(value: &'a str, start: &str, end: &str) -> Option<&'a str> {
    value
        .split_once(start)?
        .1
        .split_once(end)
        .map(|x| x.0.trim())
}
pub struct OnvifAdapter {
    source: SourceId,
    device: OnvifDevice,
}
impl OnvifAdapter {
    pub fn new(source: SourceId, device: OnvifDevice) -> Self {
        Self { source, device }
    }
    pub fn source_id(&self) -> &SourceId {
        &self.source
    }
    pub fn device(&self) -> &OnvifDevice {
        &self.device
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_fixture() {
        let d = parse_probe("<ProbeMatch><XAddrs>http://camera/onvif</XAddrs><Name>front</Name><Profile token=\"t1\"/></ProbeMatch>").unwrap();
        assert_eq!(d.endpoint, "http://camera/onvif");
        assert_eq!(d.profile_tokens, vec!["t1"]);
    }
}
