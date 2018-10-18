//! Failure errors for stuff that can go wrong with rendering

use super::*;

use failure::{Backtrace, Context, Fail};


/// An error with loading and processing a config
#[derive(Debug)]
pub struct ConfigLoadError {
    inner: Context<ConfigLoadErrorKind>,
}

impl ConfigLoadError {
    pub fn kind(&self) -> &ConfigLoadErrorKind {
        &self.inner.get_context()
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Fail)]
pub enum ConfigLoadErrorKind {
    #[fail(display = "A problem with parsing the provided TOML")]
    TomlParseError,

    /// This variant wraps the `String` returned from the SDL2 library
    #[fail(display = "A problem with loading and processing images with SDL")]
    SDLError(String),

    #[fail(display = "One of the required blocks had no tiles supplied")]
    MissingBlock(Block),
}

impl ConfigLoadErrorKind {
    pub fn from_sdl_string_err(s: String) -> ConfigLoadErrorKind {
        ConfigLoadErrorKind::SDLError(s)
    }
}

impl Fail for ConfigLoadError {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for ConfigLoadError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl From<ConfigLoadErrorKind> for ConfigLoadError {
    fn from(kind: ConfigLoadErrorKind) -> ConfigLoadError {
        ConfigLoadError {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ConfigLoadErrorKind>> for ConfigLoadError {
    fn from(inner: Context<ConfigLoadErrorKind>) -> ConfigLoadError {
        ConfigLoadError { inner }
    }
}

/// An error with rendering an `IsoMap`
#[derive(Fail, Debug)]
#[fail(display = "SDL returned an error: {}", sdl_err)]
pub struct RendererError{sdl_err: String}

impl From<String> for RendererError {
   fn from(s: String) -> RendererError{
        RendererError{ sdl_err: s }
    }
}
