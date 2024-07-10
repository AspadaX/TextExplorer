use std::fs;


pub enum ConfigurationErrors {
    LoadConfigurationFileError, 
    JSONParsingError
}


pub fn load_configurations(
    filepath: &str
) -> Result<serde_json::Value, ConfigurationErrors> {

    // read configuration file as a string
    let data = match fs::read_to_string(filepath) {
        Ok(result) => result, 
        Err(_) => return Err(ConfigurationErrors::JSONParsingError)
    };

    // load the string into a json value with serde_json
    let _value: serde_json::Value = match serde_json::from_str(&data) {
        Ok(result) => return Ok(result), 
        Err(_) => return Err(ConfigurationErrors::JSONParsingError)
    };

}