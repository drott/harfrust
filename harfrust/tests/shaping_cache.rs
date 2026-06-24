use std::fs;
use std::path::PathBuf;
use harfrust::{
    font::{BuiltinFontFuncs, FontFuncs},
    FontRef, ShapeOptions, ShaperData, UnicodeBuffer,
};
use read_fonts::types::GlyphId;

fn get_ahem_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fonts/text-rendering-tests/Ahem.ttf")
}

struct FontAFuncs;
impl FontFuncs for FontAFuncs {
    fn nominal_glyph(&mut self, builtin: &BuiltinFontFuncs, c: u32) -> Option<GlyphId> {
        if (0x40..=0x4F).contains(&c) { builtin.nominal_glyph(c) } else { None }
    }
}

struct FontBFuncs;
impl FontFuncs for FontBFuncs {
    fn nominal_glyph(&mut self, builtin: &BuiltinFontFuncs, c: u32) -> Option<GlyphId> {
        if (0x40..=0x42).contains(&c) { builtin.nominal_glyph(c) } else { None }
    }
}

#[test]
fn reproduce_shape_plan_caching_issue() {
    let font_data = fs::read(get_ahem_path()).unwrap();
    let font = FontRef::new(&font_data).unwrap();
    let data = ShaperData::new(&font);
    let shaper = data.shaper(&font).build();
    let text = "ABCD"; // 0x41, 0x42, 0x43, 0x44

    // 1. Shape with Font A (maps all of ABCD)
    let glyphs_a = shaper.shape(
        { let mut b = UnicodeBuffer::new(); b.push_str(text); b.guess_segment_properties(); b },
        ShapeOptions::new().font_funcs(Some(&mut FontAFuncs)),
    );
    let ids_a: Vec<_> = glyphs_a.glyph_infos().iter().map(|g| g.glyph_id).collect();

    // 2. Shape with Font B (should only map AB, but will see cached CD from Font A)
    let glyphs_b = shaper.shape(
        { let mut b = UnicodeBuffer::new(); b.push_str(text); b.guess_segment_properties(); b },
        ShapeOptions::new().font_funcs(Some(&mut FontBFuncs)),
    );
    let ids_b: Vec<_> = glyphs_b.glyph_infos().iter().map(|g| g.glyph_id).collect();

    println!("Font A: {:?}", ids_a);
    println!("Font B: {:?}", ids_b);

    if ids_a == ids_b {
        panic!("BUG REPRODUCED: Font B used cached results from Font A. Both: {:?}", ids_a);
    }
}
