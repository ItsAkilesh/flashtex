//! XeLaTeX/LuaLaTeX compatibility for FlashTeX (task KC-104).
//!
//! * [`detect`]: which engine a document expects, the compile mode to use,
//!   diagnostics for Lua code.
//! * [`fontspec`]: font commands, options, face requests, feature plans.
//! * [`locate`]: [`locate::FontLocator`] trait and a directory implementation.
//! * [`texlig`]: `Ligatures=TeX` (XeTeX tex-text mapping, luaotfload `tlig`).
//! * [`otl`] + [`shaper`]: script-aware GSUB/GPOS for fontspec features.
//! * [`glue`]: interword glue and space factor per engine.
//! * [`measure`]: word positions inside a line at natural glue.
//! * [`unimath`] + [`mathfont`]: unicode-math alphabets/styles and MATH constants.
//!
//! See `CONTRACT.md` for how compiler, render-pipeline, math-layout and
//! font-engine can adopt this crate. TeX engines are used only to produce the
//! committed oracle JSON; nothing here runs TeX.

pub mod detect;
pub mod engine;
pub mod fontspec;
pub mod glue;
pub mod keyval;
pub mod locate;
pub mod mathfont;
pub mod measure;
pub mod otl;
pub mod shaper;
pub mod texlig;
pub mod unimath;

pub use detect::{CompileMode, Detection, detect};
pub use engine::{Engine, EngineProfile, Renderer};
