use std::io::{BufRead, BufReader, Read};

pub struct Parser {
    data_buffer: String,
    event_type_buffer: String,
    last_event_id_buffer: String,
    reconnection_time: Option<u64>,
    buffer: String,
    last_delimiter: Option<String>,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            data_buffer: String::new(),
            event_type_buffer: String::new(),
            last_event_id_buffer: String::new(),
            reconnection_time: None,
            buffer: String::new(),
            last_delimiter: None,
        }
    }

    pub fn feed<F>(&mut self, chunk: &str, mut callback: F)
    where
        F: FnMut(String, String, String, Option<u64>),
    {
        self.buffer.push_str(chunk);

        if self.last_delimiter == Some("\r".to_string()) {
            self.buffer.drain(..1);
        }

        while let Some(pos) = self.buffer.find(|c| "\r\n".contains(c)) {
            let line = self.buffer.drain(..pos).collect::<String>();
            let delim_len = if self.buffer.starts_with("\r\n") {
                2
            } else {
                1
            };
            let delim = self.buffer.drain(..delim_len).collect::<String>();
            self.last_delimiter = Some(delim);
            self.process_line(&line, &mut callback);
        }
    }

    pub fn stream<F, R>(&mut self, mut reader: R, mut callback: F)
    where
        F: FnMut(String, String, String, Option<u64>),
        R: Read,
    {
        let mut reader = BufReader::new(reader);
        let mut buffer = String::new();

        while reader.read_line(&mut buffer).unwrap() > 0 {
            self.feed(&buffer, &mut callback);
            buffer.clear();
        }
    }

    fn process_line<F>(&mut self, line: &str, callback: &mut F)
    where
        F: FnMut(String, String, String, Option<u64>),
    {
        match line {
            "" => self.dispatch_event(callback),
            line if line.starts_with(':') => self.ignore(),
            line if line.contains(':') => {
                let (field, value) = line.split_once(':').unwrap();
                let value = value.trim_start();
                self.process_field(field, value);
            }
            line => self.process_field(line, ""),
        }
    }

    fn process_field(&mut self, field: &str, value: &str) {
        match field {
            "event" => self.event_type_buffer = value.to_string(),
            "data" => {
                self.data_buffer.push_str(value);
                self.data_buffer.push('\n');
            }
            "id" => {
                if !value.contains('\0') {
                    self.last_event_id_buffer = value.to_string();
                }
            }
            "retry" => {
                if let Ok(retry) = value.parse::<u64>() {
                    self.reconnection_time = Some(retry);
                }
            }
            _ => self.ignore(),
        }
    }

    fn dispatch_event<F>(&mut self, callback: &mut F)
    where
        F: FnMut(String, String, String, Option<u64>),
    {
        let id = self.last_event_id_buffer.clone();

        if self.data_buffer.is_empty() {
            self.data_buffer.clear();
            self.event_type_buffer.clear();
            return;
        }

        if self.data_buffer.ends_with('\n') {
            self.data_buffer.pop();
        }

        let event_type = self.event_type_buffer.clone();
        let data = self.data_buffer.clone();

        self.data_buffer.clear();
        self.event_type_buffer.clear();

        callback(event_type, data, id, self.reconnection_time);
    }

    fn ignore(&mut self) {}
}
