use crate::{TaytayError, events::MotionEvent, model::SourceId};
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
impl OnvifEvent {
    pub fn as_motion_event(&self) -> MotionEvent {
        MotionEvent {
            id: self.id.clone(),
            source: self.source.clone(),
            observed_at: self.observed_at.clone(),
            kind: self.topic.clone(),
            confidence: None,
        }
    }
}
pub fn parse_event(xml: &str, source: SourceId) -> Result<OnvifEvent, TaytayError> {
    let id = local_element(xml, "MessageId")
        .ok_or_else(|| TaytayError::Protocol("ONVIF event has no message ID".into()))?;
    let topic = local_element(xml, "Topic")
        .ok_or_else(|| TaytayError::Protocol("ONVIF event has no topic".into()))?;
    let observed_at = local_element(xml, "UtcTime")
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
    let endpoint = local_element(xml, "XAddrs")
        .ok_or_else(|| TaytayError::Protocol("ONVIF probe has no XAddrs".into()))?;
    let name = local_element(xml, "Name");
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
fn local_element<'a>(xml: &'a str, local_name: &str) -> Option<&'a str> {
    let start = xml.match_indices('<').find_map(|(index, _)| {
        let remainder = &xml[index + 1..];
        let name = remainder
            .split(|character: char| character == '>' || character.is_whitespace())
            .next()?;
        (name.rsplit(':').next()? == local_name).then_some(index)
    })?;
    let opening_end = xml[start..].find('>')? + start;
    let closing_start = xml[opening_end + 1..]
        .match_indices("</")
        .find_map(|(offset, _)| {
            let remainder = &xml[opening_end + 1 + offset + 2..];
            let name = remainder.split('>').next()?.trim();
            (name.rsplit(':').next()? == local_name).then_some(opening_end + 1 + offset)
        })?;
    Some(xml[opening_end + 1..closing_start].trim())
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
            "<tt:MessageId>m1</tt:MessageId><tt:Topic>Motion</tt:Topic><tt:UtcTime>2027</tt:UtcTime>",
            SourceId::new("front"),
        )
        .unwrap();
        assert_eq!(event.topic, "Motion");
        assert_eq!(event.as_motion_event().kind, "Motion");
    }
}
