pub mod number_vector;
pub use number_vector::DefNumberIter;
pub use number_vector::NewNumberIter;
pub use number_vector::SetNumberIter;

pub mod text_vector;
pub use text_vector::DefTextIter;
pub use text_vector::NewTextIter;
pub use text_vector::SetTextIter;

pub mod switch_vector;
pub use switch_vector::DefSwitchIter;
pub use switch_vector::NewSwitchIter;
pub use switch_vector::SetSwitchIter;

pub mod light_vector;
pub use light_vector::DefLightIter;
pub use light_vector::SetLightIter;

pub mod blob_vector;
pub use blob_vector::DefBlobIter;
pub use blob_vector::SetBlobIter;

pub mod message;
pub use message::MessageIter;

pub mod del_property;
pub use del_property::DelPropertyIter;

pub mod get_properties;
use super::*;
pub use get_properties::GetPropertiesIter;

use quick_xml::Result as XmlResult;
use quick_xml::{Reader, Writer};

pub trait XmlSerialization {
    fn send<'a, T: std::io::Write>(
        &self,
        xml_writer: &'a mut Writer<T>,
    ) -> XmlResult<&'a mut Writer<T>>;
}

#[derive(Debug)]
pub enum DeError {
    XmlError(quick_xml::Error),
    DecodeUtf8(str::Utf8Error),
    DecodeLatin(Cow<'static, str>),
    ParseIntError(num::ParseIntError),
    ParseFloatError(num::ParseFloatError),
    ParseSexagesimalError(String),
    ParseDateTimeError(ParseError),
    MissingAttr(&'static str),
    BadAttr(AttrError),
    UnexpectedAttr(String),
    UnexpectedEvent(String),
    UnexpectedTag(String),
}

impl From<quick_xml::Error> for DeError {
    fn from(err: quick_xml::Error) -> Self {
        DeError::XmlError(err)
    }
}
impl From<str::Utf8Error> for DeError {
    fn from(err: str::Utf8Error) -> Self {
        DeError::DecodeUtf8(err)
    }
}
impl From<Cow<'static, str>> for DeError {
    fn from(err: Cow<'static, str>) -> Self {
        DeError::DecodeLatin(err)
    }
}
impl From<num::ParseIntError> for DeError {
    fn from(err: num::ParseIntError) -> Self {
        DeError::ParseIntError(err)
    }
}
impl From<num::ParseFloatError> for DeError {
    fn from(err: num::ParseFloatError) -> Self {
        DeError::ParseFloatError(err)
    }
}
impl From<ParseError> for DeError {
    fn from(err: ParseError) -> Self {
        DeError::ParseDateTimeError(err)
    }
}
impl From<AttrError> for DeError {
    fn from(err: AttrError) -> Self {
        DeError::BadAttr(err)
    }
}

impl<'a> TryFrom<Attribute<'a>> for SwitchRule {
    type Error = DeError;

    fn try_from(value: Attribute<'a>) -> Result<Self, Self::Error> {
        match value.unescaped_value()? {
            Cow::Borrowed(b"OneOfMany") => Ok(SwitchRule::OneOfMany),
            Cow::Borrowed(b"AtMostOne") => Ok(SwitchRule::AtMostOne),
            Cow::Borrowed(b"AnyOfMany") => Ok(SwitchRule::AnyOfMany),
            e => return Err(DeError::UnexpectedEvent(format!("{:?}", e))),
        }
    }
}

impl<'a> TryFrom<Attribute<'a>> for PropertyState {
    type Error = DeError;

    fn try_from(value: Attribute<'a>) -> Result<Self, Self::Error> {
        match value.unescaped_value()? {
            Cow::Borrowed(b"Idle") => Ok(PropertyState::Idle),
            Cow::Borrowed(b"Ok") => Ok(PropertyState::Ok),
            Cow::Borrowed(b"Busy") => Ok(PropertyState::Busy),
            Cow::Borrowed(b"Alert") => Ok(PropertyState::Alert),
            e => return Err(DeError::UnexpectedEvent(format!("{:?}", e))),
        }
    }
}

impl<'a> TryFrom<BytesText<'a>> for PropertyState {
    type Error = DeError;

    fn try_from(value: BytesText<'a>) -> Result<Self, Self::Error> {
        match value.unescaped()? {
            Cow::Borrowed(b"Idle") => Ok(PropertyState::Idle),
            Cow::Borrowed(b"Ok") => Ok(PropertyState::Ok),
            Cow::Borrowed(b"Busy") => Ok(PropertyState::Busy),
            Cow::Borrowed(b"Alert") => Ok(PropertyState::Alert),
            e => return Err(DeError::UnexpectedEvent(format!("{:?}", e))),
        }
    }
}

impl<'a> TryFrom<BytesText<'a>> for SwitchState {
    type Error = DeError;

    fn try_from(value: BytesText<'a>) -> Result<Self, Self::Error> {
        match value.unescaped()? {
            Cow::Borrowed(b"On") => Ok(SwitchState::On),
            Cow::Borrowed(b"Off") => Ok(SwitchState::Off),
            e => return Err(DeError::UnexpectedEvent(format!("{:?}", e))),
        }
    }
}
impl<'a> TryFrom<Attribute<'a>> for PropertyPerm {
    type Error = DeError;

    fn try_from(value: Attribute<'a>) -> Result<Self, Self::Error> {
        match value.unescaped_value()? {
            Cow::Borrowed(b"ro") => Ok(PropertyPerm::RO),
            Cow::Borrowed(b"wo") => Ok(PropertyPerm::WO),
            Cow::Borrowed(b"rw") => Ok(PropertyPerm::RW),
            e => return Err(DeError::UnexpectedEvent(format!("{:?}", e))),
        }
    }
}

pub struct CommandIter<T: std::io::BufRead> {
    xml_reader: Reader<T>,
    buf: Vec<u8>,
}

impl<T: std::io::BufRead> Iterator for CommandIter<T> {
    type Item = Result<Command, DeError>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.next_command() {
            Ok(Some(command)) => {
                return Some(Ok(command));
            }
            Ok(None) => return None,
            Err(e) => {
                return Some(Err(e));
            }
        }
    }
}

impl<T: std::io::BufRead> CommandIter<T> {
    pub fn new(xml_reader: Reader<T>) -> CommandIter<T> {
        let buf = Vec::new();
        CommandIter { xml_reader, buf }
    }

    pub fn buffer_position(&self) -> usize {
        self.xml_reader.buffer_position()
    }

    fn next_command(&mut self) -> Result<Option<Command>, DeError> {
        let event = self.xml_reader.read_event(&mut self.buf)?;
        match event {
            Event::Start(e) => {
                let result = match e.name() {
                    b"defTextVector" => {
                        let mut text_vector = DefTextIter::text_vector(&self.xml_reader, &e)?;

                        for text in DefTextIter::new(self) {
                            let text = text?;
                            text_vector.texts.push(text);
                        }

                        Ok(Some(Command::DefTextVector(text_vector)))
                    }
                    b"setTextVector" => {
                        let mut text_vector = SetTextIter::text_vector(&self.xml_reader, &e)?;

                        for text in SetTextIter::new(self) {
                            let text = text?;
                            text_vector.texts.push(text);
                        }

                        Ok(Some(Command::SetTextVector(text_vector)))
                    }
                    b"newTextVector" => {
                        let mut text_vector = NewTextIter::text_vector(&self.xml_reader, &e)?;

                        for text in NewTextIter::new(self) {
                            let text = text?;
                            text_vector.texts.push(text);
                        }

                        Ok(Some(Command::NewTextVector(text_vector)))
                    }
                    b"defNumberVector" => {
                        let mut number_vector = DefNumberIter::number_vector(&self.xml_reader, &e)?;

                        for number in DefNumberIter::new(self) {
                            let number = number?;
                            number_vector.numbers.push(number);
                        }

                        Ok(Some(Command::DefNumberVector(number_vector)))
                    }
                    b"setNumberVector" => {
                        let mut number_vector = SetNumberIter::number_vector(&self.xml_reader, &e)?;

                        for number in SetNumberIter::new(self) {
                            let number = number?;
                            number_vector.numbers.push(number);
                        }

                        Ok(Some(Command::SetNumberVector(number_vector)))
                    }
                    b"newNumberVector" => {
                        let mut number_vector = NewNumberIter::number_vector(&self.xml_reader, &e)?;

                        for number in NewNumberIter::new(self) {
                            let number = number?;
                            number_vector.numbers.push(number);
                        }

                        Ok(Some(Command::NewNumberVector(number_vector)))
                    }
                    b"defSwitchVector" => {
                        let mut switch_vector = DefSwitchIter::switch_vector(&self.xml_reader, &e)?;

                        for switch in DefSwitchIter::new(self) {
                            let switch = switch?;
                            switch_vector.switches.push(switch);
                        }

                        Ok(Some(Command::DefSwitchVector(switch_vector)))
                    }
                    b"setSwitchVector" => {
                        let mut switch_vector = SetSwitchIter::switch_vector(&self.xml_reader, &e)?;

                        for switch in SetSwitchIter::new(self) {
                            let switch = switch?;
                            switch_vector.switches.push(switch);
                        }

                        Ok(Some(Command::SetSwitchVector(switch_vector)))
                    }
                    b"newSwitchVector" => {
                        let mut switch_vector = NewSwitchIter::switch_vector(&self.xml_reader, &e)?;

                        for switch in NewSwitchIter::new(self) {
                            let switch = switch?;
                            switch_vector.switches.push(switch);
                        }

                        Ok(Some(Command::NewSwitchVector(switch_vector)))
                    }
                    b"defLightVector" => {
                        let mut light_vector = DefLightIter::light_vector(&self.xml_reader, &e)?;

                        for light in DefLightIter::new(self) {
                            let light = light?;
                            light_vector.lights.push(light);
                        }

                        Ok(Some(Command::DefLightVector(light_vector)))
                    }
                    b"setLightVector" => {
                        let mut light_vector = SetLightIter::light_vector(&self.xml_reader, &e)?;

                        for light in SetLightIter::new(self) {
                            let light = light?;
                            light_vector.lights.push(light);
                        }

                        Ok(Some(Command::SetLightVector(light_vector)))
                    }
                    b"defBLOBVector" => {
                        let mut blob_vector = DefBlobIter::blob_vector(&self.xml_reader, &e)?;

                        for blob in DefBlobIter::new(self) {
                            let blob = blob?;
                            blob_vector.blobs.push(blob);
                        }

                        Ok(Some(Command::DefBlobVector(blob_vector)))
                    }
                    b"setBLOBVector" => {
                        let mut blob_vector = SetBlobIter::blob_vector(&self.xml_reader, &e)?;

                        for blob in SetBlobIter::new(self) {
                            let blob = blob?;
                            blob_vector.blobs.push(blob);
                        }

                        Ok(Some(Command::SetBlobVector(blob_vector)))
                    }
                    b"message" => {
                        let message = MessageIter::message(&self.xml_reader, &e)?;
                        for _ in MessageIter::new(self) {}

                        Ok(Some(Command::Message(message)))
                    }
                    b"delProperty" => {
                        let message = DelPropertyIter::del_property(&self.xml_reader, &e)?;
                        for _ in DelPropertyIter::new(self) {}

                        Ok(Some(Command::DelProperty(message)))
                    }

                    b"getProperties" => {
                        let get_properties =
                            GetPropertiesIter::get_properties(&self.xml_reader, &e)?;
                        for _ in GetPropertiesIter::new(self) {}

                        Ok(Some(Command::GetProperties(get_properties)))
                    }
                    tag => Err(DeError::UnexpectedTag(str::from_utf8(tag)?.to_string())),
                };
                result
            }
            Event::End(tag) => {
                println!("Unexpected end: {}", tag.escape_ascii().to_string());
                Err(DeError::UnexpectedEvent(format!("{:?}", tag)))
            }
            Event::Eof => Ok(None),
            e => return Err(DeError::UnexpectedEvent(format!("{:?}", e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_def_number_vector() {
        let xml = r#"
    <defNumberVector device="CCD Simulator" name="SIMULATOR_SETTINGS" label="Settings" group="Simulator Config" state="Idle" perm="rw" timeout="60" timestamp="2022-08-12T05:52:27">
        <defNumber name="SIM_XRES" label="CCD X resolution" format="%4.0f" min="512" max="8192" step="512">
    1280
        </defNumber>
        <defNumber name="SIM_YRES" label="CCD Y resolution" format="%4.0f" min="512" max="8192" step="512">
    1024
        </defNumber>
        <defNumber name="SIM_XSIZE" label="CCD X Pixel Size" format="%4.2f" min="1" max="30" step="5">
    5.2000000000000001776
        </defNumber>
    </defNumberVector>
                    "#;
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut command_iter = CommandIter::new(reader);

        match command_iter.next().unwrap().unwrap() {
            Command::DefNumberVector(param) => {
                assert_eq!(param.device, "CCD Simulator");
                assert_eq!(param.name, "SIMULATOR_SETTINGS");
                assert_eq!(param.numbers.len(), 3)
            }
            e => {
                panic!("Unexpected: {:?}", e)
            }
        }
    }

    #[test]
    fn test_set_number_vector() {
        let xml = r#"
<setNumberVector device="CCD Simulator" name="SIM_FOCUSING" state="Ok" timeout="60" timestamp="2022-10-01T21:21:10">
    <oneNumber name="SIM_FOCUS_POSITION">
7340
    </oneNumber>
    <oneNumber name="SIM_FOCUS_MAX">
100000
    </oneNumber>
    <oneNumber name="SIM_SEEING">
3.5
    </oneNumber>
</setNumberVector>
"#;
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut command_iter = CommandIter::new(reader);

        match command_iter.next().unwrap().unwrap() {
            Command::SetNumberVector(param) => {
                assert_eq!(param.device, "CCD Simulator");
                assert_eq!(param.name, "SIM_FOCUSING");
                assert_eq!(param.numbers.len(), 3)
            }
            e => {
                panic!("Unexpected: {:?}", e)
            }
        }
    }

    #[test]
    fn test_new_number_vector() {
        let xml = r#"
<newNumberVector device="CCD Simulator" name="SIM_FOCUSING" timestamp="2022-10-01T21:21:10">
    <oneNumber name="SIM_FOCUS_POSITION">
7340
    </oneNumber>
    <oneNumber name="SIM_FOCUS_MAX">
100000
    </oneNumber>
    <oneNumber name="SIM_SEEING">
3.5
    </oneNumber>
</newNumberVector>
"#;
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut command_iter = CommandIter::new(reader);

        match command_iter.next().unwrap().unwrap() {
            Command::NewNumberVector(param) => {
                assert_eq!(param.device, "CCD Simulator");
                assert_eq!(param.name, "SIM_FOCUSING");
                assert_eq!(param.numbers.len(), 3)
            }
            e => {
                panic!("Unexpected: {:?}", e)
            }
        }
    }

    #[test]
    fn test_def_text_vector() {
        let xml = r#"
<defTextVector device="CCD Simulator" name="ACTIVE_DEVICES" label="Snoop devices" group="Options" state="Idle" perm="rw" timeout="60" timestamp="2022-09-05T21:07:22">
    <defText name="ACTIVE_TELESCOPE" label="Telescope">
Telescope Simulator
    </defText>
    <defText name="ACTIVE_ROTATOR" label="Rotator">
Rotator Simulator
    </defText>
    <defText name="ACTIVE_FOCUSER" label="Focuser">
Focuser Simulator
    </defText>
    <defText name="ACTIVE_FILTER" label="Filter">
CCD Simulator
    </defText>
    <defText name="ACTIVE_SKYQUALITY" label="Sky Quality">
SQM
    </defText>
</defTextVector>
                    "#;
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut command_iter = CommandIter::new(reader);

        match command_iter.next().unwrap().unwrap() {
            Command::DefTextVector(param) => {
                assert_eq!(param.device, "CCD Simulator");
                assert_eq!(param.name, "ACTIVE_DEVICES");
                assert_eq!(param.texts.len(), 5)
            }
            e => {
                panic!("Unexpected: {:?}", e)
            }
        }
    }

    #[test]
    fn test_set_text_vector() {
        let xml = r#"
<setTextVector device="CCD Simulator" name="ACTIVE_DEVICES" state="Ok" timeout="60" timestamp="2022-10-01T17:06:14">
    <oneText name="ACTIVE_TELESCOPE">
Simulator changed
    </oneText>
    <oneText name="ACTIVE_ROTATOR">
Rotator Simulator
    </oneText>
    <oneText name="ACTIVE_FOCUSER">
Focuser Simulator
    </oneText>
    <oneText name="ACTIVE_FILTER">
CCD Simulator
    </oneText>
    <oneText name="ACTIVE_SKYQUALITY">
SQM
    </oneText>
</setTextVector>
                    "#;
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut command_iter = CommandIter::new(reader);

        match command_iter.next().unwrap().unwrap() {
            Command::SetTextVector(param) => {
                assert_eq!(param.device, "CCD Simulator");
                assert_eq!(param.name, "ACTIVE_DEVICES");
                assert_eq!(param.texts.len(), 5)
            }
            e => {
                panic!("Unexpected: {:?}", e)
            }
        }
    }

    #[test]
    fn test_new_text_vector() {
        let xml = r#"
<newTextVector device="CCD Simulator" name="ACTIVE_DEVICES" timestamp="2022-10-01T17:06:14">
    <oneText name="ACTIVE_TELESCOPE">
Simulator changed
    </oneText>
    <oneText name="ACTIVE_ROTATOR">
Rotator Simulator
    </oneText>
    <oneText name="ACTIVE_FOCUSER">
Focuser Simulator
    </oneText>
    <oneText name="ACTIVE_FILTER">
CCD Simulator
    </oneText>
    <oneText name="ACTIVE_SKYQUALITY">
SQM
    </oneText>
</newTextVector>
                    "#;
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut command_iter = CommandIter::new(reader);

        match command_iter.next().unwrap().unwrap() {
            Command::NewTextVector(param) => {
                assert_eq!(param.device, "CCD Simulator");
                assert_eq!(param.name, "ACTIVE_DEVICES");
                assert_eq!(param.texts.len(), 5)
            }
            e => {
                panic!("Unexpected: {:?}", e)
            }
        }
    }

    #[test]
    fn test_def_switch_vector() {
        let xml = r#"
<defSwitchVector device="CCD Simulator" name="SIMULATE_BAYER" label="Bayer" group="Simulator Config" state="Idle" perm="rw" rule="OneOfMany" timeout="60" timestamp="2022-09-06T01:41:22">
    <defSwitch name="INDI_ENABLED" label="Enabled">
Off
    </defSwitch>
    <defSwitch name="INDI_DISABLED" label="Disabled">
On
    </defSwitch>
</defSwitchVector>
                    "#;
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut command_iter = CommandIter::new(reader);

        match command_iter.next().unwrap().unwrap() {
            Command::DefSwitchVector(param) => {
                assert_eq!(param.device, "CCD Simulator");
                assert_eq!(param.name, "SIMULATE_BAYER");
                assert_eq!(param.switches.len(), 2)
            }
            e => {
                panic!("Unexpected: {:?}", e)
            }
        }
    }

    #[test]
    fn test_set_switch_vector() {
        let xml = r#"
<setSwitchVector device="CCD Simulator" name="DEBUG" state="Ok" timeout="0" timestamp="2022-10-01T22:07:22">
    <oneSwitch name="ENABLE">
On
    </oneSwitch>
    <oneSwitch name="DISABLE">
Off
    </oneSwitch>
</setSwitchVector>
                    "#;
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut command_iter = CommandIter::new(reader);

        match command_iter.next().unwrap().unwrap() {
            Command::SetSwitchVector(param) => {
                assert_eq!(param.device, "CCD Simulator");
                assert_eq!(param.name, "DEBUG");
                assert_eq!(param.switches.len(), 2)
            }
            e => {
                panic!("Unexpected: {:?}", e)
            }
        }
    }

    #[test]
    fn test_new_switch_vector() {
        let xml = r#"
<newSwitchVector device="CCD Simulator" name="DEBUG" timestamp="2022-10-01T22:07:22">
    <oneSwitch name="ENABLE">
On
    </oneSwitch>
    <oneSwitch name="DISABLE">
Off
    </oneSwitch>
</newSwitchVector>
                    "#;
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut command_iter = CommandIter::new(reader);

        match command_iter.next().unwrap().unwrap() {
            Command::NewSwitchVector(param) => {
                assert_eq!(param.device, "CCD Simulator");
                assert_eq!(param.name, "DEBUG");
                assert_eq!(param.switches.len(), 2)
            }
            e => {
                panic!("Unexpected: {:?}", e)
            }
        }
    }

    #[test]
    fn test_def_light_vector() {
        let xml = r#"
<defLightVector device="CCD Simulator" name="SIMULATE_BAYER" label="Bayer" group="Simulator Config" state="Idle" timestamp="2022-09-06T01:41:22">
    <defLight name="INDI_ENABLED" label="Enabled">
Busy
    </defLight>
    <defLight name="INDI_DISABLED" label="Disabled">
Ok
    </defLight>
</defLightVector>
                    "#;
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut command_iter = CommandIter::new(reader);

        match command_iter.next().unwrap().unwrap() {
            Command::DefLightVector(param) => {
                assert_eq!(param.device, "CCD Simulator");
                assert_eq!(param.name, "SIMULATE_BAYER");
                assert_eq!(param.lights.len(), 2)
            }
            e => {
                panic!("Unexpected: {:?}", e)
            }
        }
    }

    #[test]
    fn test_set_light_vector() {
        let xml = r#"
<setLightVector device="CCD Simulator" name="SIMULATE_BAYER" state="Idle" timestamp="2022-09-06T01:41:22">
    <oneLight name="INDI_ENABLED">
Busy
    </oneLight>
    <oneLight name="INDI_DISABLED">
Ok
    </oneLight>
</setLightVector>
                    "#;
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut command_iter = CommandIter::new(reader);

        match command_iter.next().unwrap().unwrap() {
            Command::SetLightVector(param) => {
                assert_eq!(param.device, "CCD Simulator");
                assert_eq!(param.name, "SIMULATE_BAYER");
                assert_eq!(param.lights.len(), 2)
            }
            e => {
                panic!("Unexpected: {:?}", e)
            }
        }
    }

    #[test]
    fn test_blob_vector() {
        let xml = r#"
<defBLOBVector device="CCD Simulator" name="SIMULATE_BAYER" label="Bayer" group="Simulator Config" perm="rw"  state="Idle" timestamp="2022-09-06T01:41:22">
    <defBLOB name="INDI_ENABLED" label="Enabled"/>
    <defBLOB name="INDI_DISABLED" label="Disabled"/>
</defBLOBVector>
                    "#;
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        reader.expand_empty_elements(true);
        let mut command_iter = CommandIter::new(reader);

        match command_iter.next().unwrap().unwrap() {
            Command::DefBlobVector(param) => {
                assert_eq!(param.device, "CCD Simulator");
                assert_eq!(param.name, "SIMULATE_BAYER");
                assert_eq!(param.blobs.len(), 2)
            }
            e => {
                panic!("Unexpected: {:?}", e)
            }
        }
    }

    #[test]
    fn test_set_blob_vector() {
        let xml = include_str!("../../tests/image_capture_blob_vector.log");

        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        reader.expand_empty_elements(true);
        let mut command_iter = CommandIter::new(reader);

        match command_iter.next().unwrap().unwrap() {
            Command::SetBlobVector(param) => {
                assert_eq!(param.device, "CCD Simulator");
                assert_eq!(param.name, "CCD1");
                assert_eq!(param.state, PropertyState::Ok);
                assert_eq!(param.blobs.len(), 1)
            }
            e => {
                panic!("Unexpected: {:?}", e)
            }
        }
    }

    #[test]
    fn test_message() {
        let xml = r#"
<message device="Telescope Simulator" timestamp="2022-10-02T00:37:07" message="[INFO] update mount and pier side: Pier Side On, mount type 2"/>
                    "#;
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        reader.expand_empty_elements(true);
        let mut command_iter = CommandIter::new(reader);

        match command_iter.next().unwrap().unwrap() {
            Command::Message(param) => {
                assert_eq!(param.device, Some(String::from("Telescope Simulator")));
                assert_eq!(
                    param.message,
                    Some(String::from(
                        "[INFO] update mount and pier side: Pier Side On, mount type 2"
                    ))
                );
            }
            e => {
                panic!("Unexpected: {:?}", e)
            }
        }
    }

    #[test]
    fn test_get_properties() {
        let xml = r#"
<getProperties version="1.7" device="Telescope Simulator" name="foothing"/>
                    "#;
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        reader.expand_empty_elements(true);
        let mut command_iter = CommandIter::new(reader);

        match command_iter.next().unwrap().unwrap() {
            Command::GetProperties(param) => {
                assert_eq!(param.device, Some(String::from("Telescope Simulator")));
                assert_eq!(param.name, Some(String::from("foothing")));
                assert_eq!(param.version, String::from("1.7"));
            }
            e => {
                panic!("Unexpected: {:?}", e)
            }
        }
    }

    #[test]
    fn test_set_simulator_log() {
        let xml = include_str!("../../tests/image_capture.log");

        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        reader.expand_empty_elements(true);
        let mut command_iter = CommandIter::new(reader);

        for command in command_iter.by_ref() {
            match command {
                Ok(_) => (),
                Err(e) => {
                    println!("position: {}", command_iter.buffer_position());
                    panic!("{:?}", e);
                }
            }
        }
    }
}
