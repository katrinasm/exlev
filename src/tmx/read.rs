use std::io;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use std::collections::BTreeMap;
use sxd_document::parser::parse;
use sxd_document::dom::Element;
use sxd_xpath::{Context, Value};
use sxd_xpath::nodeset::Node;
use why_sxd::{make_xpath, make_xpath_static, xpath_nodes, xpath_nodes_str, attr_u32, attr_hexbyte,
              only_match, only_match_str, node_element_attr, attr_bool};

use super::TmxError;
use snes_color::SnesPal;

use spr::*;
use entrance::{EntrancePlacement, EntranceId};
use level::{self, Level, PScrGrid, LevelHeader, Palette};

pub fn read_level<R: io::Read, P: AsRef<Path>>(source: &mut R, dir: P, levelnum: u16) -> Result<Level, TmxError> {
    let pkg = {
        let mut buf = String::new();
        source.read_to_string(&mut buf).unwrap();
        parse(&buf).map_err(|_| "bad TMX file")?
    };
    let doc = pkg.as_document();
    let root_node = doc.root().into();
    let ctx = Context::new();

    let first_gid_fg = not_nan(
        make_xpath_static("/map/tileset[@name='fg']/@firstgid")
            .evaluate(&ctx, root_node)?
            .number(),
    ).ok_or(
        "couldn't get first tile for fg; tileset 'fg' may be missing",
    )? as u16;

    let mut fg = read_block_grid(&ctx, root_node, "Level", first_gid_fg)?;
    let bg = read_block_grid(&ctx, root_node, "BG", first_gid_fg)?;

    let map = xpath_nodes_str(&ctx, root_node, "map")
        .unwrap()
        .document_order_first()
        .unwrap();
    let sprs = read_sprite_layer(&ctx, &map)?;
    level::place_sprites(&mut fg, &sprs);
    let exits = read_exit_layer(&ctx, &map, levelnum)?;
    level::place_exits(&mut fg, &exits);
    
    let sf = read_scroll_filter(&ctx, root_node)?;
    
    let dir_path = dir.as_ref();
    let hed = read_header(&ctx, root_node, dir_path)?;
    
    let ens = read_entrance_layer(&ctx, &map, levelnum)?;
    
    let lvl = Level::from_parts(fg, bg, sf, ens, hed);

    Ok(lvl)
}


fn not_nan(x: f64) -> Option<f64> {
    if !x.is_nan() { Some(x) } else { None }
}

fn read_block_grid(
    context: &Context,
    node: Node,
    name: &'static str,
    firstgid: u16,
) -> Result<PScrGrid, TmxError> {
    let tiles = read_block_layer(context, node, name, firstgid)?;
    Ok(level::pscreens_from_linear_tiles(&tiles, 32, 32))
}


fn read_block_layer(
    context: &Context,
    node: Node,
    name: &'static str,
    firstgid: u16,
) -> Result<Vec<u16>, TmxError> {
    let layer_path = make_xpath(&format!("/map/layer[@name='{}']", name))?;

    let layer = only_match(context, node, &layer_path).ok_or_else(|| {
        format!("need exactly 1 layer named {}", name)
    })?;

    Ok(read_tiles(context, &layer, firstgid).unwrap())
}

fn read_scroll_filter(context: &Context, node: Node) -> Result<Vec<bool>, TmxError> {
    let layer_path = make_xpath_static("/map/layer[@name='Scroll']");

    let layer = only_match(context, node, &layer_path).ok_or(
        "need exactly 1 layer named \"Scroll\"",
    )?;

    // We can use a fake first GID because we don’t care at all what kind of tile we find,
    // only whether tiles exist or not.
    let tiles = read_tiles(context, &layer, 1)?;

    let mut filt = vec![false; 1024];

    // iterate over screens
    for i in 0..filt.len() {
        let sx = i % 32;
        let sy = i / 32;

        // iterate over tiles in the screen
        for y in 0..16 {
            for x in 0..16 {
                let tile_idx = (sy * 16 + y) * 512 + (sx * 16 + x);
                if tiles[tile_idx] != 0x0025 {
                    filt[i] = true;
                    break;
                }
            }
        }
    }

    Ok(filt)
}

fn read_tiles(context: &Context, layer: &Node, firstgid: u16) -> Result<Vec<u16>, TmxError> {

    if attr_u32(layer, "width") != Some(512) || attr_u32(layer, "height") != Some(512) {
        return Err("all layers need to be 512x512".into());
    };

    let data = if let Some(v) = only_match_str(context, *layer, "data").and_then(|n| n.element()) {
        v
    } else {
        return Err("a <layer> is badly formatted".into());
    };

    let encoding = data.attribute("encoding").map(|a| a.value()).unwrap_or(
        "xml",
    );

    let mut tiles = Vec::with_capacity(512 * 512);

    match encoding {
        //"xml" => read_tiles_xml(context, layer, &mut tiles, firstgid)?,
        "csv" => read_tiles_csv(layer, &mut tiles, firstgid)?,
        s => {
            return Err(
                format!("a <layer> has an unsupported encoding (encoding=\"{}\")", s).into(),
            )
        }
    };

    if tiles.len() < 512 * 512 {
        Err("a <layer> has too few tiles for its dimensions".into())
    } else if tiles.len() > 512 * 512 {
        Err("a <layer> has too many tiles for its dimensions".into())
    } else {
        Ok(tiles)
    }
}

fn read_header(context: &Context, doc: Node, dir: &Path) -> Result<LevelHeader, TmxError> {
    let mut hed = LevelHeader::default();
    header_bitfield(context, doc, &mut hed.mode, "level-mode", 5)?;
    header_byte(context, doc, &mut hed.audio_track, "audio-track")?;
    header_bitfield(context, doc, &mut hed.tileset_fg, "fg-tileset", 4)?;
    header_bitfield(context, doc, &mut hed.tileset_sp, "sp-tileset", 4)?;
    header_bitfield(context, doc, &mut hed.scroll, "scroll-allowance-numeric", 2)?;
    header_bitfield(context, doc, &mut hed.l3_img, "layer-3-image", 2)?;
    header_bool(context, doc, &mut hed.l3_prio, "layer-3-priority")?;
    header_palette(context, doc, &mut hed.palette, dir)?;
    Ok(hed)
}

fn el_type_is(el: &Element, name: &str) -> bool {
    el.attribute("type").map_or(false, |v| v.value() == name)
}

fn header_byte(context: &Context, doc: Node, targ: &mut u8, name: &str) -> Result<(), TmxError> {
    header_bitfield(context, doc, targ, name, 8)
}

fn header_bitfield(context: &Context, doc: Node, targ: &mut u8, name: &str, width: u8) -> Result<(), TmxError> {
    assert!(width <= 8, "tried to extract too wide bitfield");
    assert!(width != 0, "tried to extract zero-width bitfield");
    let filter = 0xff >> (8 - width);
    
    if let Some(el) = header_element(context, doc, name)? {
        if !el_type_is(&el, "int")  {
            return Err(format!("bad header component ({}): should have type \"int\"", name).into());
        };
        if let Some(ev) = el.attribute("value") {
            let v = ev.value().parse::<u8>()
                .map_err(|_| "that's not a number binch")?;
            if v > filter {
                return Err(
                    format!("bad header component ({}): out-of-range (max is {})", name, filter).into()
                );
            }
            *targ = v;
        };
    };
    
    Ok(())
}

fn header_bool(context: &Context, doc: Node, targ: &mut bool, name: &str) -> Result<(), TmxError> {
    if let Some(el) = header_element(context, doc, name)? {
        if !el_type_is(&el, "bool") {
            return Err(format!("bad header component ({}): should have type \"bool\"", name).into());
        };
        if let Some(ev) = el.attribute("value") {
            let v = ev.value().parse::<bool>()
                .map_err(|_| "that's not a bool binch")?;
            *targ = v;
        };
    };
    
    Ok(())
}

fn header_palette(context: &Context, doc: Node, targ: &mut Palette, dir: &Path) -> Result<(), TmxError> {
    if let Some(el) = header_element(context, doc, "palette")? {
        if !el_type_is(&el, "file") {
            return Err(format!("bad header component (palette): should have type \"file\"").into());
        };
        
        if let Some(ev) = el.attribute("value") {
            let mut p = dir.to_path_buf();
            p.push(ev.value());
            let mut f = File::open(p.clone()).map_err(|e| format!("bad header component (palette): {}", e))?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf).map_err(|e| format!("bad header component (palette): {}", e))?;
            *targ = Palette::Custom(
                SnesPal::from_lm_pal(&buf).ok_or_else(
                    || format!("bad header componenent (palette): file \"{:}\" is not a valid .pal file", p.to_string_lossy())
                )?
            );
        } else {
            return Err(format!("bad header component (palette): has no value").into())
        };
    };
    
    Ok(())
}

fn header_element<'d>(context: &Context<'d>, doc: Node<'d>, name: &str) -> Result<Option<Element<'d>>, TmxError> {
    let path = make_xpath(&format!("/map/properties/property[@name = \"{}\"]", name))?;
    
    let nodes = if let Value::Nodeset(ns) = path.evaluate(context, doc)? {
        ns
    } else {
        return Ok(None);
    };
    
    if nodes.size() == 0 {
        Ok(None)
    } else if nodes.size() > 1 {
        Err(format!("too many entries for header field {}", name).into())
    } else {
        if let Some(Node::Element(e)) = nodes.document_order_first() {
            Ok(Some(e))
        } else {
            Err(format!("header componenent {} is deeply messed up", name).into())
        }
    }
}

fn tile_val(n: u16, first: u16) -> u16 {
    if n != 0 { n - first } else { 0x0025 }
}

fn read_tiles_csv(layer: &Node, buf: &mut Vec<u16>, firstgid: u16) -> Result<(), TmxError> {
    let s = layer.string_value();
    for chunk in s.split(',') {
        let core = chunk.trim();
        let value = core.parse::<u16>().map_err(|_| "invalid tile ID")?;
        buf.push(tile_val(value, firstgid));
    }
    Ok(())
}

fn read_sprite_layer(context: &Context, map: &Node) -> Result<SprSet, TmxError> {
    let mut sprlist = SprSet::new();

    let firstgid = not_nan(
        make_xpath_static("tileset[@name='sprites']/@firstgid")
            .evaluate(&context, *map)?
            .number(),
    ).ok_or(
        "couldn't find first tile for sprites; tileset 'sprites' may be missing",
    )? as u16;

    for node in xpath_nodes_str(&context, *map, "objectgroup[@name='Sprites']/object")? {
        sprlist.insert(sprite_from_node(&context, &node, firstgid)?);
    }

    Ok(sprlist)
}

fn sprite_from_node(
    context: &Context,
    node: &Node,
    firstgid: u16,
) -> Result<SpritePlacement, TmxError> {
    let path = make_xpath_static("properties/property");

    let mut xbit = false;
    let mut xbytes = [0; 4];

    for prop_node in xpath_nodes(context, *node, &path)? {
        let name = node_element_attr(&prop_node, "name")
            .ok_or("Invalid sprite property (has no name)")?;
        match name {
            "ebit" => xbit = attr_hexbyte(&prop_node, "value").unwrap_or(0) != 0,
            "xb1" => xbytes[0] = attr_hexbyte(&prop_node, "value").unwrap_or(0),
            "xb2" => xbytes[1] = attr_hexbyte(&prop_node, "value").unwrap_or(0),
            "xb3" => xbytes[2] = attr_hexbyte(&prop_node, "value").unwrap_or(0),
            "xb4" => xbytes[3] = attr_hexbyte(&prop_node, "value").unwrap_or(0),
            x => return Err(format!("Invalid sprite property: {}", x).into()),
        };
    }

    let id = attr_u32(node, "gid").ok_or("A sprite has an invalid ID")? as u16 - firstgid;
    // x + 16, y - 16 is the center of a 32x32 square
    let pos_x = (attr_u32(node, "x").ok_or("A sprite has an invalid X pos")? + 16) as u16 / 16;
    let pos_y = (attr_u32(node, "y").ok_or("A sprite has an invalid Y pos")? - 16) as u16 / 16;

    Ok(SpritePlacement::new(id, pos_x, pos_y, xbit, xbytes))
}

fn read_entrance_layer(context: &Context, map: &Node, levelnum: u16) -> Result<Vec<EntrancePlacement>, TmxError> {
    let mut entlist = Vec::new();
    
    let firstgid = not_nan(
        make_xpath_static("tileset[@name='entrances']/@firstgid")
            .evaluate(&context, *map)?
            .number(),
    ).ok_or(
        "couldn't find first tile for entrances; tileset 'entrances' may be missing",
    )? as u16;
    
    for node in xpath_nodes_str(&context, *map, "objectgroup[@name='Entrances']/object")? {
        entlist.push(entrance_from_node(&context, &node, levelnum, firstgid)?);
    }

    Ok(entlist)
}

fn entrance_from_node(
    context: &Context,
    node: &Node,
    levelnum: u16,
    firstgid: u16,
) -> Result<EntrancePlacement, TmxError> {
    let path = make_xpath_static("properties/property");
    
    let mut id = None;
    let mut water = false;
    let mut slippery = false;
    
    for prop_node in xpath_nodes(context, *node, &path)? {
        let name = node_element_attr(&prop_node, "name")
            .ok_or("Invalid entrance property (has no name)")?;
        match name {
            // "fragment" => id = Some(EntranceId::from_num_and_fragment(levelnum, "m00").unwrap()),
            "fragment" => {
                id = Some(
                    EntranceId::from_num_and_fragment(
                        levelnum,
                        node_element_attr(&prop_node, "value")
                            .ok_or("Invalid entrance property (fragment): has no value")?,
                    ).ok_or("Invalid entrance property (fragment)")?
                );
            },
            "water" =>
                water = attr_bool(&prop_node, "value").ok_or("An entrance has an invalid boolean")?,
            "slippery" =>
                slippery = attr_bool(&prop_node, "value").ok_or("An entrance has an invalid boolean")?,
            x => return Err(format!("Invalid entrance property: {}", x).into()),
        };
    }
    
    let idv = id.ok_or("Entrance has no fragment")?;
    
    let anim = (attr_u32(node, "gid").ok_or("An entrance has an invalid animation")? - (firstgid as u32)) as u8;
    
    if anim >= 8 {
        return Err(format!("An entrance has an invalid animation ({}), should be in 0 ..= 7", anim).into())
    }
    
    // x, y - 32 is the top left of a 32x32 square
    let pos_x = (attr_u32(node, "x").ok_or("An entrance has an invalid X pos")?) as u16 / 16;
    let pos_y = (attr_u32(node, "y").ok_or("An entrance has an invalid Y pos")? - 32) as u16 / 16;
    
    Ok(EntrancePlacement::new(idv, pos_x, pos_y, anim, water, slippery))
}


fn read_exit_layer(context: &Context, map: &Node, levelnum: u16) -> Result<BTreeMap<(u8, u8), EntranceId>, TmxError> {
    let mut exitmap = BTreeMap::new();
    for node in xpath_nodes_str(&context, *map, "objectgroup[@name='Exits']/object")? {
        let (x, y, exit) = exit_from_node(&context, &node, levelnum)?;
        if let Some(_) = exitmap.insert((x, y), exit) {
            return Err(format!("screen {}, {} has two exit objects", x, y).into());
        }
    }
    
    Ok(exitmap)
}

fn exit_from_node(context: &Context, node: &Node, levelnum: u16) -> Result<(u8, u8, EntranceId), TmxError> {
    let path = make_xpath_static("properties/property");
    
    let mut id = None;
    
    for prop_node in xpath_nodes(context, *node, &path)? {
        let name = node_element_attr(&prop_node, "name")
            .ok_or("Invalid entrance property (has no name)")?;
        match name {
            "target" => {
                let v = node_element_attr(&prop_node, "value")
                            .ok_or("Invalid exit property (target): has no value")?;
                id = Some(if v.starts_with('#') {
                    let (_, rest) = v.split_at(1);
                    EntranceId::from_num_and_fragment(levelnum, rest).ok_or("Invalid exit property (target)")?
                } else {
                    EntranceId::from_name(v).ok_or("Invalid exit property (target)")?
                });
            },
            x => return Err(format!("Invalid exit property: {}", x).into()),
        };
    }
    
    let idv = id.ok_or("Exit has no path")?;
    
    let scr_x = (attr_u32(node, "x").ok_or("An entrance has an invalid X pos")? / 256) as u8;
    let scr_y = (attr_u32(node, "y").ok_or("An entrance has an invalid Y pos")? / 256) as u8;
    
    Ok((scr_x, scr_y, idv))
}

