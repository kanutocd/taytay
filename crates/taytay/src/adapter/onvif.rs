use crate::{TaytayError, model::SourceId};
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OnvifDevice {
    pub endpoint: String,
    pub name: Option<String>,
    pub profile_tokens: Vec<String>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OnvifEvent {
    pub id: String,
    pub topic: String,
    pub observed_at: String,
    pub source: SourceId,
}
pub fn parse_event(xml: &str, source: SourceId) -> Result<OnvifEvent, TaytayError> {
    let id = between(xml, "<MessageId>", "</MessageId>")
        .ok_or_else(|| TaytayError::Protocol("ONVIF event has no message ID".into()))?;
    let topic = between(xml, "<Topic>", "</Topic>")
        .ok_or_else(|| TaytayError::Protocol("ONVIF event has no topic".into()))?;
    let observed_at = between(xml, "<UtcTime>", "</UtcTime>")
        .ok_or_else(|| TaytayError::Protocol("ONVIF event has no timestamp".into()))?;
    Ok(OnvifEvent {
        id: id.into(),
        topic: topic.into(),
        observed_at: observed_at.into(),
        source,
    })
}
impl OnvifDevice {
    pub fn select_profile(&self, preferred: &[&str]) -> Option<&str> {
        preferred
            .iter()
            .find_map(|wanted| {
                self.profile_tokens
                    .iter()
                    .find(|token| token == wanted)
                    .map(String::as_str)
            })
            .or_else(|| self.profile_tokens.first().map(String::as_str))
    }
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
        assert_eq!(d.select_profile(&["missing", "t1"]), Some("t1"));
        let event = parse_event(
            "<MessageId>m1</MessageId><Topic>Motion</Topic><UtcTime>2027</UtcTime>",
            SourceId::new("front"),
        )
        .unwrap();
        assert_eq!(event.topic, "Motion");
    }
}
