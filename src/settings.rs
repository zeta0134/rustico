// Standard Library
use std::ffi::OsString;
use std::fs;

// Third-party
use toml::Value;
use toml::map::Map;

// First-party
use events::Event;


const DEFAULT_CONFIG: &str = r#"
[video]
ntsc_filter = false
simulate_overscan = false
display_fps = false
scale_factor = 2

[piano_roll]
canvas_width = 1920
canvas_height = 1080
key_length = 64
key_thickness = 16
octave_count = 9
scale_factor = 1
speed_multiplier = 6
starting_octave = 0
waveform_height = 128

"#;

pub struct RusticNesSettings {
    pub root: Value
}

impl RusticNesSettings {
    pub fn load(filename: &OsString) -> RusticNesSettings {
        match fs::read_to_string(filename) {
            Ok(config_str) => {
                let config_from_file = config_str.parse::<Value>().unwrap();
                return RusticNesSettings {
                    root: config_from_file
                }
            },
            Err(_) => {
                let default_config = DEFAULT_CONFIG.parse::<Value>().unwrap();
                return RusticNesSettings {
                    root: default_config
                }
            }
        }
    }

    pub fn save(&self, filename: &OsString) {
        let config_str = toml::to_string(&self.root).unwrap();
        fs::write(filename, config_str).expect("Unable to write settings!");
        println!("Wrote settings to {:?}", filename);
    }

    fn _emit_events(value: Value, prefix: String) -> Vec<Event> {
        let mut events: Vec<Event> = Vec::new();
        match value {
            Value::Table(table) => {
                for key in table.keys() {
                    let new_prefix = if prefix == "" {key.to_string()} else {format!("{}.{}", prefix, key)};
                    events.extend(RusticNesSettings::_emit_events(table[key].clone(), new_prefix));
                }
            },
            Value::Boolean(boolean_value) => {events.push(Event::ApplyBooleanSetting(prefix, boolean_value));},
            Value::Float(float_value) => {events.push(Event::ApplyFloatSetting(prefix, float_value));},
            Value::Integer(integer_value) => {events.push(Event::ApplyIntegerSetting(prefix, integer_value));},
            Value::String(string_value) => {events.push(Event::ApplyStringSetting(prefix, string_value));},
            _ => {
                /* Unimplemented! */
            }
        }
        return events;
    }

    pub fn apply_settings(&self) -> Vec<Event> {
        return RusticNesSettings::_emit_events(self.root.clone(), "".to_string());
    }

    fn _ensure_path_exists(path: String, current_table: &mut Map<String, Value>, default_value: Value) {
        let components = path.split(".").collect::<Vec<&str>>();
        if components.len() == 1 {
            // This is the last path element. Either confirm the existence of this key
            // or create it with the default value.
            if current_table.contains_key(components[0]) {
                // we're done!
                return;
            } else {
                current_table.insert(components[0].to_string(), default_value);
            }
        } else {
            if !current_table.contains_key(components[0]) {
                current_table.insert(components[0].to_string(), Value::try_from(Map::new()).unwrap());
            }
            let child_table = current_table[components[0]].as_table_mut().unwrap();
            let remaining_path = components[1..].join(".");
            RusticNesSettings::_ensure_path_exists(remaining_path, child_table, default_value);
        }
    }

    pub fn ensure_path_exists(&mut self, path: String, default_value: Value) {
        let root_table = self.root.as_table_mut().unwrap();
        RusticNesSettings::_ensure_path_exists(path, root_table, default_value);
    }

    pub fn _get(path: String, current_table: &Map<String, Value>) -> Option<&Value> {
        let components = path.split(".").collect::<Vec<&str>>();
        if components.len() == 1 {
            if current_table.contains_key(components[0]) {
                return Some(&current_table[&components[0].to_string()]);
            }
        } else {
            if current_table.contains_key(components[0]) {
                let child = &current_table[&components[0].to_string()];
                if child.is_table() {
                    let child_table = current_table[components[0]].as_table().unwrap();
                    let remaining_path = components[1..].join(".");
                    return RusticNesSettings::_get(remaining_path, child_table);
                }
            }
        }
        return None;
    }

    pub fn get(&self, path: String) -> Option<&Value> {
        let root_table = self.root.as_table().unwrap();
        return RusticNesSettings::_get(path, root_table);
    }

    pub fn _set(path: String, current_table: &mut Map<String, Value>, new_value: Value) {
        let components = path.split(".").collect::<Vec<&str>>();
        if components.len() == 1 {
            if current_table.contains_key(components[0]) {
                current_table[&components[0].to_string()] = new_value;
            }
        } else {
            if current_table.contains_key(components[0]) {
                let child = &current_table[&components[0].to_string()];
                if child.is_table() {
                    let child_table = current_table[components[0]].as_table_mut().unwrap();
                    let remaining_path = components[1..].join(".");
                    RusticNesSettings::_set(remaining_path, child_table, new_value);
                }
            }
        }
    }

    pub fn set(&mut self, path: String, new_value: Value) {
        let root_table = self.root.as_table_mut().unwrap();
        return RusticNesSettings::_set(path, root_table, new_value);
    }

    pub fn handle_event(&mut self, event: Event) -> Vec<Event> {
        let mut events: Vec<Event> = Vec::new();
        match event {
            Event::StoreBooleanSetting(path, value) => {
                self.ensure_path_exists(path.clone(), Value::from(false));
                self.set(path.clone(), Value::from(value));
                events.push(Event::ApplyBooleanSetting(path, value));
            },
            Event::StoreFloatSetting(path, value) => {
                self.ensure_path_exists(path.clone(), Value::from(false));
                self.set(path.clone(), Value::from(value));
                events.push(Event::ApplyFloatSetting(path, value));
            },
            Event::StoreIntegerSetting(path, value) => {
                self.ensure_path_exists(path.clone(), Value::from(false));
                self.set(path.clone(), Value::from(value));
                events.push(Event::ApplyIntegerSetting(path, value));
            },
            Event::StoreStringSetting(path, value) => {
                self.ensure_path_exists(path.clone(), Value::from(false));
                self.set(path.clone(), Value::from(value.clone()));
                events.push(Event::ApplyStringSetting(path, value.clone()));
            },
            Event::ToggleBooleanSetting(path) => {
                self.ensure_path_exists(path.clone(), Value::from(false));
                let current_value = self.get(path.clone()).unwrap().as_bool().unwrap();
                self.set(path.clone(), Value::from(!current_value));
                events.push(Event::ApplyBooleanSetting(path, !current_value));
            },
            _ => {}
        }
        return events;
    }
}