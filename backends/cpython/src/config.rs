use heck::{ToLowerCamelCase, ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};

/// Style of documentation in generated Python code
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PythonNamingStyle {
    /// Names are the same as in the original rust code
    AsIs,
    /// Names all in lowercase without spacing e.g. 'thetypename'
    Lowercase,
    /// Names all in uppercase without spacing e.g. 'THETYPENAME'
    Uppercase,
    /// Names in mixed case starting with lowercase without spacing e.g. 'theTypeName'
    LowerCamelCase,
    /// Names in mixed case starting with uppercase without spacing e.g. 'TheTypeName'
    UpperCamelCase,
    /// Names in lower case with '_' as spacing e.g. 'the_type_name'
    SnakeCase,
    /// Names in upper case with '_' as spacing e.g. 'THE_TYPE_NAME'
    ShoutySnakeCase,
}

pub(crate) trait ToNamingStyle {
    fn to_naming_style(&self, style: &PythonNamingStyle) -> String;
}

impl ToNamingStyle for String {
    fn to_naming_style(&self, style: &PythonNamingStyle) -> String {
        self.as_str().to_naming_style(style)
    }
}

impl ToNamingStyle for &str {
    fn to_naming_style(&self, style: &PythonNamingStyle) -> String {
        match style {
            PythonNamingStyle::AsIs => self.to_string(),
            PythonNamingStyle::Lowercase => self.to_lowercase(),
            PythonNamingStyle::Uppercase => self.to_uppercase(),
            PythonNamingStyle::LowerCamelCase => self.to_lower_camel_case(),
            PythonNamingStyle::UpperCamelCase => self.to_upper_camel_case(),
            PythonNamingStyle::SnakeCase => self.to_snake_case(),
            PythonNamingStyle::ShoutySnakeCase => self.to_shouty_snake_case(),
        }
    }
}

/// Configures Python code generation.
#[derive(Clone, Debug)]
pub struct Config {
    /// How to name the function responsible for loading the DLL, e.g., `init_api`.
    pub init_api_function_name: String,
    /// Attribute by which the `cffi` object is exposed, e.g., `ffi`.
    pub ffi_attribute: String,
    /// Namespace to put functions into, e.g., `api`.
    pub raw_fn_namespace: String,
    /// Namespace for callback helpers, e.g., `callbacks`.
    pub callback_namespace: String,
    // How to convert enum variant names
    pub enum_variant_naming: PythonNamingStyle,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            init_api_function_name: "init_api".to_string(),
            ffi_attribute: "ffi".to_string(),
            raw_fn_namespace: "api".to_string(),
            callback_namespace: "callbacks".to_string(),
            enum_variant_naming: PythonNamingStyle::AsIs,
        }
    }
}

/// Configures Python documentation generation.
#[derive(Clone, Debug, Default)]
pub struct DocConfig {
    /// Header to append to the generated documentation.
    pub header: String,
}
