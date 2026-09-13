//! Which Unicode TeX engine (and which OpenType renderer inside it) a
//! computation emulates. The two engines differ in observable ways that this
//! crate models explicitly; every difference below was measured with TeX
//! Live 2026 (XeTeX 0.999998, LuaHBTeX 1.24.0, luaotfload fontloader
//! 2023-12-28) — see `fixtures/expected`.

/// The engine whose behaviour is emulated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Engine {
    /// pdfTeX: 8-bit fonts, TFM ligature/kerning programs, `inputenc` LICR.
    PdfTeX,
    /// XeTeX: every OpenType font is shaped by HarfBuzz; TeX ligatures come
    /// from the `tex-text` TECkit character mapping applied before shaping.
    XeTeX,
    /// LuaTeX with luaotfload.
    LuaTeX,
}

/// The OpenType layout implementation a font is shaped with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Renderer {
    /// HarfBuzz: XeTeX always; LuaTeX with fontspec `Renderer=HarfBuzz`.
    /// Script from the text (`latn` → `DFLT`), legacy `kern` table used when
    /// GPOS has no `kern` feature.
    HarfBuzz,
    /// luaotfload `mode=node` (LuaTeX default): script `dflt` unless
    /// `Script=` is given, legacy `kern` table ignored (Times New Roman `AV`
    /// is unkerned under LuaLaTeX but kerned under XeLaTeX).
    LuaNode,
    /// luaotfload `mode=base`: features folded into TFM-like ligature/kern
    /// tables. Treated like `LuaNode` for the features modelled here.
    LuaBase,
}

impl Engine {
    /// The renderer used when a document does not say otherwise.
    pub fn default_renderer(self) -> Renderer {
        match self {
            Engine::XeTeX | Engine::PdfTeX => Renderer::HarfBuzz,
            Engine::LuaTeX => Renderer::LuaNode,
        }
    }

    pub fn is_unicode(self) -> bool {
        !matches!(self, Engine::PdfTeX)
    }

    pub fn name(self) -> &'static str {
        match self {
            Engine::PdfTeX => "pdftex",
            Engine::XeTeX => "xetex",
            Engine::LuaTeX => "luatex",
        }
    }
}

/// Engine plus renderer: everything shaping and glue depend on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EngineProfile {
    pub engine: Engine,
    pub renderer: Renderer,
}

impl EngineProfile {
    pub fn new(engine: Engine) -> EngineProfile {
        EngineProfile {
            engine,
            renderer: engine.default_renderer(),
        }
    }

    pub fn with_renderer(mut self, renderer: Renderer) -> EngineProfile {
        self.renderer = renderer;
        self
    }

    pub const XETEX: EngineProfile = EngineProfile {
        engine: Engine::XeTeX,
        renderer: Renderer::HarfBuzz,
    };
    pub const LUATEX: EngineProfile = EngineProfile {
        engine: Engine::LuaTeX,
        renderer: Renderer::LuaNode,
    };
}
