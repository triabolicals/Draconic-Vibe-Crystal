use std::{io::{Read, Seek}, sync::OnceLock};
use super::*;
use engage::{
    app::{UnitPool, Ut, GodPool},
    combat::ICharacterAssetForm,
    unity_engine::{material::*, Color, renderer::*, IObject_2Methods},
    List_1Ext,
};
use outfit_core::UnitAssetMenuData;
use crate::{DVCFlags, DVCVariables, gamedata::{EmblemPool, PersonPool}};
pub static PRESET_MATERIAL_DATA: OnceLock<Vec<PresetMaterialData>> = OnceLock::new();
pub struct PresetMaterialData {
    pub kind: u8,
    pub colors: [Color; 7],
    pub parameters: [f32; 8],
}
#[repr(u8)]
#[derive(PartialOrd, PartialEq)]
pub enum PresetMaterialKind {
    EmblemHair,
    EmblemDress,
    EmblemEye,
    EmblemSkin,
    DarkEmblemHair,
    DarkEmblemDress,
    DarkEmblemEye,
    DarkEmblemSkin,
    DivineHair,
    DivineEye,
    FellHair,
    FellEye,
    EmblemBrow,
    DarkEmblemBrow,
}
impl PresetMaterialData {
    pub fn new(kind: u8) -> Self {
        let colors = [Color{r: 0.0, b: 0.0, g: 0.0, a: 0.0}; 7];
        let parameters = [0.0; 8];
        Self { kind, colors, parameters }
    }
    pub fn apply_material_preset(kind: PresetMaterialKind, material: Material) {
        let index = kind as usize;
        if let Some(data) = PRESET_MATERIAL_DATA.get_or_init(|| Self::init()).get(index){
            data.apply_to_material(material);
        }
    }
    pub fn apply_to_material(&self, material: Material) {
        match self.kind {
            0|1|3 => {
                ["_BaseColor", "_GradationColor", "_OutlineColor", "_EmissionColor",
                    if self.kind == 0 { "_AngelRingColor" } else { "_EngageEmissionColor" },
                    "_RimLightColorLight", "RimLightColorShadow"
                ].into_iter().enumerate()
                    .for_each(|(i, x)|{
                        if self.colors[i].a > 0.1 { material.set_color_2(x, self.colors[i]); }
                    });

                ["_Preset", "_S_Key_RimLight",
                    if self.kind == 0 { "_AngelRingOffsetU" } else { "_S_Key_BumpAttenuation" },
                    "_LightColorToWhite", "_LightShadowToWhite",
                    "_RimLightBlend", "_RimLightScale",
                    "_OutlineScale"
                ].into_iter().enumerate()
                    .for_each(|(i, x)|{ material.set_float(x, self.parameters[i]); });
            }
            2 => {
                UnitAssetData::EYE_COLOR.iter()
                    .enumerate()
                    .for_each(|(i, x)|{
                        if self.colors[i].a > 0.1 { material.set_color_2(*x, self.colors[i]); }
                    });
            }
            4 => {
                ["_BaseColor", "_ShadowColor", "_ShadowAddColor"].into_iter().enumerate()
                    .for_each(|(i, x)|{ if self.colors[i].a > 0.1 { material.set_color_2(x, self.colors[i]); } });
            }
            _ => {}
        }
    }
    pub fn init() -> Vec<Self> {
        let data = include_bytes!("../../data/colors.bin");
        let count = data[0];
        let mut cursor = std::io::Cursor::new(data);
        let mut buffer_1 = [0u8; 38];
        let mut buffer_2 = [0u8; 36];
        let mut buffer_3 = [0u8; 80];
        let _ = cursor.seek(std::io::SeekFrom::Start(1)).unwrap();
        let mut c = 0;
        let mut out = vec![];
        while c < count {
            cursor.read(&mut buffer_1).unwrap();
            let material_type = buffer_1[0];
            let mut preset = Self::new(material_type);
            for x in 0..3 {
                preset.colors[x].r = Self::get_f32(&buffer_1, 2, 3*x);
                preset.colors[x].g = Self::get_f32(&buffer_1, 2, 3*x+1);
                preset.colors[x].b = Self::get_f32(&buffer_1, 2, 3*x+2);
                if preset.colors[x].r + preset.colors[x].g + preset.colors[x].b > 0.001 {
                    preset.colors[x].a = 1.0;
                }
                else { preset.colors[x].a = 0.0; }
            }
            match material_type {
                0|1|3 => {  // Hair / Dress / Skin
                    cursor.read(&mut buffer_3).unwrap();
                    for x in 0..4 {
                        preset.colors[x+3].r = Self::get_f32(&buffer_3, 0, 3*x);
                        preset.colors[x+3].g = Self::get_f32(&buffer_3, 0, 3*x+1);
                        preset.colors[x+3].b = Self::get_f32(&buffer_3, 0, 3*x+2);
                        if preset.colors[x+3].r + preset.colors[x+3].g + preset.colors[x+3].b > 0.001 { preset.colors[x+3].a = 1.0; }
                        else { preset.colors[x+3].a = 0.0; }
                    }
                    for x in 0..8 { preset.parameters[x] = Self::get_f32(&buffer_3, 0, 12 + x); }
                }
                2 => {  // Eye
                    cursor.read(&mut buffer_2).unwrap();
                    for x in 0..3 {
                        preset.colors[x+3].r = Self::get_f32(&buffer_2, 0, 3*x);
                        preset.colors[x+3].g = Self::get_f32(&buffer_2, 0, 3*x+1);
                        preset.colors[x+3].b = Self::get_f32(&buffer_2, 0, 3*x+2);
                        if (preset.colors[x+3].r + preset.colors[x+3].g + preset.colors[x+3].b) > 0.001 { preset.colors[x+3].a = 1.0; } else { preset.colors[x+3].a = 0.0; }
                    }
                }
                _ => {}
            }
            out.push(preset);
            c += 1;
        }
        out
    }
    pub fn get_f32(buffer: &[u8], offset: usize, index: usize) -> f32 {
        let i = 4*index + offset;
        f32::from_le_bytes([buffer[i], buffer[i+1], buffer[i+2], buffer[i+3]])
    }
}
fn emblem_material_edit_all(go: List_1<Material>, dark: bool, hair: bool, hero: bool) {
    /*
    go.get_components_in_children_3::<SkinnedMeshRenderer>(true).iter()
        .flat_map(|r| r.get_materials().iter().chain(r.get_shared_materials().iter()))
        .for_each(|m| {
            emblem_material_edit(m, dark && !hero, hair);
            if hero {
                let name = m.get_name().to_rust_string();
                if name.starts_with("MtHair") && !name.contains("Brow") {
                    PresetMaterialData::apply_material_preset(PresetMaterialKind::EmblemHair, m);
                }
            }
        });
     */
    go.iter().for_each(|m| {
       emblem_material_edit(m, dark && !hero, hair);
        if hero {
            let name = m.get_name().to_rust_string();
            if name.starts_with("MtHair") && !name.contains("Brow") {
                PresetMaterialData::apply_material_preset(PresetMaterialKind::EmblemHair, m);
            }
        }
    });
}
fn set_color(c1: &mut Color, r: f32, g: f32, b: f32) { c1.r = r;c1.g = g;c1.b = b; }

#[unity::hook("Combat", "CharacterAppearance", "ModifyColors")]
pub fn modify_colors(this: CharacterAppearance, go: GameObject, _method_info: OptionalMethod) {
    //get_head_hair_colors(go);
    let hash = unity::field_get_value_at_offset::<i32>(this, 0xd4);
    if hash == 0 {
        call_original!(this, go, None);
        return;
    }
    if hash == 1 && PersonPool::get_replacement_index(0, false).is_some_and(|v| v > 40){
        let mut hair = this.hair_color();
        let mut grad = this.grad_color();
        set_color(&mut hair, 0.152, 0.27, 0.462);
        set_color(&mut grad, 0.09, 0.176, 0.34);
        this.set_hair_color(hair);
        this.set_grad_color(grad);
    }
    // get_eyes_colors(this, go);
    call_original!(this, go, None);
    UnitAssetMenuData::get().preview.update = 0;
    let instanced_materials = this.m_instanced_materials();
    // if DVCConfig::get().debug {
        // println!("Instance Materials");
        // instanced_materials.iter().for_each(|m| { println!(" Material: {}", m.get_name()); });
   //  }
    if let Some(god) = to_opt(GodData::try_get_from_hash(hash)){
        let m022 = DVCVariables::get_chapter_index() == 22;
        let dark = EmblemPool::is_enemy_dark(god) || to_opt(GodPool::try_get_2(god, true)).is_some_and(|v| v.get_is_darkness() || (v.m_is_escaping() && m022));
        let hair = god.is_hero() || dark || (DVCFlags::GodNames.get_value() && !EmblemPool::is_custom(god));
        if god.is_hero() { emblem_material_edit_all(instanced_materials, false, true, true); }
        else if DVCFlags::GodNames.get_value() || dark { emblem_material_edit_all(instanced_materials, dark, hair, false); }
        else if !dark && EmblemPool::is_custom(god) { emblem_material_edit_all(instanced_materials, dark, hair, false); }
    }
    else if hash == 1 {
        emblem_material_edit_all(instanced_materials, false, true, true);
        let hero = UnitPool::get_hero(false).get_person();
        let hero_hash = hero.hash();
        unity::field_set_value_at_offset::<i32>(this, 0xd4, hero_hash);
    }
    if let Some(pos) = AppearanceManager::BLACK.iter().position(|h| *h == hash) {
        if pos == 4 {
            if let Some(replacement) = PersonPool::get_replacement_index(0, false){
                if replacement != 0 && replacement != 32 { instanced_materials.iter().for_each(|m| { evil_color(m, true, true); }); }
            }
        }
        else {
            if let Some(replacement) = PersonPool::get_replacement_index(32, false){
                if pos != 1 && replacement == 0{ instanced_materials.iter().for_each(|m| { evil_color(m, true, false); }); }
            }
        }
    }
    if PersonPool::DIVINE_LUEUR.contains(&hash) {
        /*
        ["MtEye2", "MtEye", "MtHair2", "MtHair"].iter()
            .flat_map(|material_name|get_material_from_object(go, material_name))
            .for_each(|material|{ divine_color(material); });

         */
        instanced_materials.iter().for_each(|m| { divine_color(m); })
    }
    if AppearanceManager::RED_EYES.contains(&hash) {
        if let Some(m) = get_material_from_object(go, "MtEye") { evil_color(m, false, true); }
    }
    apply_colors_2(this, go);
    // apply_preview_head_hair_color(this, go);
}

#[skyline::hook(offset=0x2b011f0)]
pub fn combat_character_play_facial(this: Character, state_hash: i32, transition: f32, optional_method: OptionalMethod) {
    let hash = unity::field_get_value_at_offset::<i32>(this.get_builder().appearance(), 0xd4);
    if !UnitAssetMenuData::get().is_preview {
        let hash: i32 = unity::field_get_value_at_offset(this.get_builder().appearance(), 0xd4);
        if let Some(person) = UnitAssetMenuData::get_current_profile(hash) {
            if let Some(pos) = FACIAL_STATES.iter().enumerate().position(|(i, s)| s.1 == state_hash && i < 4){
                let exp = person.expression[pos] as usize;
                if exp > 0 && exp != (pos + 1) {
                    return call_original!(this, FACIAL_STATES[exp -1 ].1, transition, optional_method);
                }
            }
        }
    }
    if state_hash == 640249298 || state_hash == 2091671594{
        if AppearanceManager::BLACK.contains(&hash){
            return call_original!(this, 16779677, transition, optional_method);
        }
        /*
        if let Some(pos) = BLACK.iter().position(|h| *h == hash) {
            let mut changed = false;
            let lr = DVCVariables::get_dvc_recruitment_index(0);
            // changed = (pos == 4 && lr != 0 && lr != 32) || (pos != 4 && DVCVariables::get_dvc_recruitment_index(32) != 32);
        }

         */
    }
    call_original!(this, state_hash, transition, optional_method);
}

fn emblem_material_edit(m: Material, dark: bool, hair: bool) {
    let name = m.get_name().to_string();
    if name.starts_with("MtHair") && hair {
        if name.contains("Brow") { if dark { change_brow_color(m, dark); } }
        else {
            if dark {
                let color = m.get_color_2("_BaseColor");
                if (color.b == color.g) && (color.r > color.b){ return; }
            }
            change_hair_material(m, dark);
        }
    }
    if name.starts_with("MtSkin") { emblem_change_skin_material(m, dark); }
    if name.starts_with("MtEye") { emblem_change_eye_material(m, dark); }
    if name.starts_with("MtDress") { emblem_change_dress_material(m, dark); }
}

fn get_material_from_object(go: GameObject, name: &str) -> Option<Material> {
    go.get_components_in_children_3::<Renderer>(true).iter()
        .flat_map(|r| Ut::get_instance_materials(r).iter())
        .find(|v| v.get_name().to_string().contains(name))
}
fn evil_color(m: Material, hair: bool, eye: bool) {
    let name = m.get_name().to_string();
    if hair && name.contains("MtHair") { PresetMaterialData::apply_material_preset(PresetMaterialKind::FellHair, m); }
    if eye && name.contains("MtEye"){ PresetMaterialData::apply_material_preset(PresetMaterialKind::FellEye, m); }
}
fn change_hair_material(m: Material, is_dark: bool) {
    if is_dark {
        PresetMaterialData::apply_material_preset(PresetMaterialKind::DarkEmblemHair, m);
        let mut base_color = m.get_color_2("_BaseColor");
        let max =
            if base_color.r > base_color.g && base_color.r > base_color.g { base_color.r }
            else if base_color.g > base_color.r && base_color.g > base_color.b { base_color.g }
            else { base_color.b };
        let min =
            if base_color.r < base_color.g && base_color.r < base_color.b { base_color.r }
            else if base_color.g < base_color.r && base_color.g < base_color.b { base_color.g }
            else { base_color.b };
        let sat = (max - min) / max;
        if sat < 0.2 {
            base_color.r = max + sat;
            base_color.g = min * (1.0 - sat);
            base_color.b = min * (1.0 - sat);
        }
        else {
            base_color.r = 0.62264144;
            base_color.g = 0.11454254;
            base_color.b = 0.11454254;
        }
        m.set_color_2("_BaseColor", base_color);
    }
    else {
        PresetMaterialData::apply_material_preset(PresetMaterialKind::EmblemHair, m);
    }
}
fn emblem_change_eye_material(m: Material, is_dark: bool) {
    let c = if is_dark { PresetMaterialKind::DarkEmblemEye } else { PresetMaterialKind::EmblemEye };
    PresetMaterialData::apply_material_preset(c, m);
}
fn emblem_change_skin_material(m: Material, is_dark: bool) {
    let c = if is_dark { PresetMaterialKind::DarkEmblemSkin } else { PresetMaterialKind::EmblemSkin };
    PresetMaterialData::apply_material_preset(c, m);
    if is_dark {
        m.set_color_2("_RimLightColorLight", Color{r: 0.718, g: 0.09, b: 0.09, a: 1.0});
    }
}
fn emblem_change_dress_material(m: Material, is_dark: bool) {
    let c = if is_dark { PresetMaterialKind::DarkEmblemDress } else { PresetMaterialKind::EmblemDress };
    PresetMaterialData::apply_material_preset(c, m);
    if is_dark {
        m.set_float("_S_Key_ColorChangeMask", 1.0);
        m.set_color_2("_ColorChangeMask025", Color{r: 0.921, g: 0.921, b: 0.921, a: 1.0});
        m.set_color_2("_ColorChangeMask050", Color{r: 0.764, g: 0.764, b: 0.764, a: 1.0});
        m.set_color_2("_ColorChangeMask075", Color{r: 0.901, g: 0.901, b: 0.901, a: 1.0});
        m.set_color_2("_ColorChangeMask100", Color{r: 0.941, g: 0.941, b: 0.941, a: 1.0});
        m.set_color_2("_RimLightColorLight", Color{r: 0.93, g: 0.60, b: 0.60, a: 1.0});
    }
}
fn change_brow_color(m: Material, is_dark: bool) {
    let c = if is_dark { PresetMaterialKind::DarkEmblemBrow } else { PresetMaterialKind::EmblemBrow };
    PresetMaterialData::apply_material_preset(c, m);
}
fn divine_color(m: Material) {
    let name = m.get_name().to_rust_string();
    if name.contains("Hair") { PresetMaterialData::apply_material_preset(PresetMaterialKind::DivineHair, m); }
    if name.contains("Eye") { PresetMaterialData::apply_material_preset(PresetMaterialKind::DivineEye, m); }
}