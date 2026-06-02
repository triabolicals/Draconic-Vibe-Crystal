use engage::{
    combat::{Character, CharacterAppearance},
    unityengine::{GameObject, Material2, Renderer, SkinnedMeshRenderer, UnityComponent, UnityObject, UnityRenderer, UnityTransform},
    ut::Ut
};
use outfit_core::UnitAssetMenuData;
use unity::{engine::Color, prelude::OptionalMethod};
use crate::assets::dress::{BLACK, RED_EYES};
use crate::DVCVariables;
const MATERIAL_EYE_COLORS: [&str; 6] = ["_BaseColor", "_BlackColor", "_DecalColor1", "_DecalColor2", "_DecalColor3", "_DecalColor4"];
pub const EYE_MATERIAL_COLORS: [u8; 54] = [
    231, 0,	0, 255, 255, 255, 7, 2, 2, 248, 123, 123, 43, 3, 3, 231, 40, 40,    // Red
    233, 47, 47, 255, 255, 255, 120, 1, 1, 243, 159, 159, 72, 14, 14, 255, 179, 179,    // Dark
    194, 194, 255, 153, 150, 219, 58, 45, 106, 241, 199, 218, 55, 53, 132, 204, 204, 236,   //God
];

const DRESS_MATERIAL_COLORS: [u8; 30] = [
    36, 4, 	4, 80, 	80, 120, 255, 25, 25, 183, 23, 23, 183, 24, 24, //  Dark
    44, 43, 79, 80, 80, 120, 166, 182, 236, 54, 224, 255, 26, 147, 168, //God
];
const BROW_COLOR: [u8; 21] = [
    80, 80, 120,    // Emission
    55, 0, 	0, 103, 5, 	5, 190, 190, 190,   // Dark
    16, 16, 55, 130, 42, 42, 118, 95, 95,   // God
];

const EMBLEM_SKIN_COLOR: [u8; 30] = [
    36, 34, 34, 80, 80, 20, 255, 25, 25, 238, 154, 154, 202, 117, 0,    // Dark
    135, 74, 113, 80, 80, 120, 110, 111, 209, 54, 224, 255, 40, 183, 209,   // God
];

#[unity::hook("Combat", "CharacterAppearance", "ModifyColors")]
pub fn modify_colors(this: &mut CharacterAppearance, go: &GameObject, _: OptionalMethod) {
    let mut emblem_type = 0;
    if this.hair_color.r >= 2.0 && this.hair_color.r < 3.0 {
        emblem_type = 1;
        this.hair_color.r -= 2.0;
    }
    else if this.hair_color.r >= 3.0 && this.hair_color.r < 4.0 {
        emblem_type = 2;
        this.hair_color.r -= 3.0;
    }
    outfit_core::get_head_hair_colors(go);
    call_original!(this, go, None);
    if emblem_type > 0 {
        let dark = emblem_type == 2;
        go.get_components_in_children::<Renderer>(true).iter().for_each(|r| {
            Ut::get_instance_materials(r).iter().for_each(|m|{ emblem_material_edit(m, dark); });
        });
        this.instanced_materials.iter().for_each(|m|{ emblem_material_edit(m, dark); });
    }

    if let Some(pos) = BLACK.iter().position(|h| *h == this.person_hash) {
        if pos == 4 {
            let replacement = DVCVariables::get_dvc_recruitment_index(0);
            if replacement != 0 && replacement != 32 {
                this.instanced_materials.iter().for_each(|m| { evil_color(m, true, true); });
            }
        }
        else {
            let replacement = DVCVariables::get_dvc_recruitment_index(32);
            if pos != 1 && replacement == 0 {
                this.instanced_materials.iter().for_each(|m| { evil_color(m, true, false); });
            }
        }
    }
    if RED_EYES.contains(&this.person_hash) {
        if let Some(m) = get_material_from_object(go, "MtEye") {
            evil_color(m, false, true);
        }
    }
    if this.person_hash == -1402039598 {
        go.get_component_in_children::<Renderer>(true).iter().flat_map(|r| r.get_game_object())
            .for_each(|r| {
                if r.get_name().contains("Eye") {
                    r.set_active2(false);
                    r.set_active(false);
                }
            });
    }
    else { outfit_core::apply_preview_head_hair_color(this, go); }
}
#[skyline::hook(offset=0x2b011f0)]
pub fn combat_character_play_facial(this: &Character, state_hash: i32, transition: f32, optional_method: OptionalMethod) {
    let hash = this.get_builder().appearance.person_hash;
    if !UnitAssetMenuData::get().is_preview {
        if let Some(person) = UnitAssetMenuData::get_current_profile(this.get_builder().appearance.person_hash) {
            if let Some(pos) = outfit_core::FACIAL_STATES.iter().enumerate().position(|(i, s)| s.1 == state_hash && i < 4){
                let exp = person.expression[pos] as usize;
                if exp > 0 && exp != (pos + 1) {
                    return call_original!(this, outfit_core::FACIAL_STATES[exp -1 ].1, transition, optional_method);
                }
            }
        }
    }
    if state_hash == 640249298 || state_hash == 2091671594{
        if let Some(pos) = BLACK.iter().position(|h| *h == hash) {
            let mut changed = false;
            let lr = DVCVariables::get_dvc_recruitment_index(0);
            changed = (pos == 4 && lr != 0 && lr != 32) || (pos != 4 && DVCVariables::get_dvc_recruitment_index(32) != 32);
            return call_original!(this, 16779677, transition, optional_method);
        }
    }
    call_original!(this, state_hash, transition, optional_method);
}

fn emblem_material_edit(m: &Material2, dark: bool) {
    let name = m.get_name().to_string();
    if name.starts_with("MtHair") {
        if name.contains("Brow") { change_brow_color(m, dark); }
        else { change_hair_material(m, dark); }
    }
    if name.starts_with("MtSkin") { emblem_change_skin_material(m, dark); }
    if name.starts_with("MtEye") { emblem_change_eye_material(m, dark); }
    if name.starts_with("MtDress") { emblem_change_dress_material(m, dark); }
}

fn get_material_from_object(go: &GameObject, name: &str) -> Option<&'static &'static Material2> {
    go.get_components_in_children::<Renderer>(true).iter()
        .flat_map(|smr| Ut::get_instance_materials(smr).iter())
        .find(|v| v.get_name().to_string().contains(name))
        .or_else(||
            go.get_components_in_children::<SkinnedMeshRenderer>(true).iter()
                .flat_map(|smr| Ut::get_instance_materials2(smr).iter())
                .find(|v| v.get_name().to_string().contains(name))
        )
}


fn evil_hair(go: &GameObject) {
    for x in ["MtHair", "MtHair1", "MtHair2"]{
        if let Some(m) = get_material_from_object(go, x) { evil_color(m, true, false); }
    }
}
fn evil_color(m: &Material2, hair: bool, eye: bool) {
    let name = m.get_name().to_string();
    if hair && name.contains("MtHair") {
        m.set_color( "_BaseColor",Color::new(0.6698,0.123,0.1492, 1.0));
        m.set_color( "_GradationColor",Color::new(0.4245,0.05807,0.05807, 1.0));
    }
    if eye && name.contains("MtEye"){
        let mut color = [1.0f32; 4];
        for x in 0..6 {
            for i in 0..3 { color[i] = EYE_MATERIAL_COLORS[x*3+ i] as f32 / 255.0; }
            m.set_color(MATERIAL_EYE_COLORS[x], Color::new(color[0], color[1], color[2], color[3]));
        }
    }
}
fn change_hair_material(m: &Material2, is_dark: bool) {
    if is_dark {
        m.set_color( "_BaseColor",Color::new(0.84906,0.49261,0.49261, 1.0));
        m.set_color( "_OutlineColor",Color::new(1.0, 0.09803919,0.09803919, 1.0));
        m.set_color( "_EmissionColor",Color::new(0.5294118,	0.011764706,	0.011764706, 1.0));
        m.set_color( "_RimLightColorLight",Color::new(1.00000,1.00000,1.00000, 1.00000));
        m.set_color( "_RimLightColorShadow",Color::new(0.85098,0.49412,0.49412, 1.00000));
    }
    else {
        m.set_color( "_EmissionColor",Color::new(0.24968153,0.246974,0.4716981, 1.0));
        m.set_color( "_OutlineColor",Color::new(0.6509804,0.71288776,0.9254902, 1.0));
        m.set_color( "_RimLightColorLight",Color::new(0.21176471,0.8784314,1.0, 1.0));
        m.set_color( "_RimLightColorShadow",Color::new(0.21176471,0.8784314,1.0, 1.0));
    }
    m.set_float( "_LightColorToWhite",0.8);
    m.set_float( "_LightShadowToWhite",0.8);
    m.set_float( "_Preset",2.0);
    m.set_float( "_S_Key_RimLight",0.0);
    m.set_float( "_RimLightBlend",if is_dark { 0.3 } else { 0.5});
    m.set_float( "_RimLightScale",0.5);
    m.set_float( "_OutlineScale",3.50);
}
fn emblem_change_eye_material(m: &Material2, is_dark: bool) {
    let mut color = [1.0f32; 4];
    let start = if is_dark { 18 } else { 36 };
    for x in 0..6 {
        for i in 0..3 { color[i] = EYE_MATERIAL_COLORS[(start + x) * 3 + i] as f32 / 255.0; }
        m.set_color(MATERIAL_EYE_COLORS[x], Color::new(color[0], color[1], color[2], color[3]));
    }
    m.set_float( "_LightColorToWhite",0.8);
    m.set_float( "_LightShadowToWhite",0.8);
}
fn emblem_change_skin_material(m: &Material2, is_dark: bool) {
    let mut color = [1.0f32; 4];
    let colors = ["_EmissionColor", "_EngageEmissionColor", "_OutlineColor", "_RimLightColorLight", "_RimLightColorShadow"];
    let start = if is_dark { 0 } else { 5 };
    for x in 0..5 {
        for i in 0..3 { color[x] = EMBLEM_SKIN_COLOR[(start + x)*3 + i] as f32 / 255.0; }
        m.set_color(colors[x], Color::new(color[0], color[1], color[2], color[3]));
    }
    m.set_float( "_Preset",5.00);
    m.set_float( "_OutlineScale",3.50);
    m.set_float( "_LightColorToWhite",if is_dark { 0.60 } else { 0.80 });
    m.set_float( "_LightShadowToWhite",0.80);
}
fn emblem_change_dress_material(m: &Material2, is_dark: bool) {
    let dress_color = ["_EmissionColor", "_EngageEmissionColor", "_OutlineColor", "_RimLightColorLight", "_RimLightColorShadow"];
    let start = if is_dark { 0 } else { 15 };
    let mut color = [1.0f32; 4];
    for x in 0..5 {
        for i in 0..3 { color[i] = DRESS_MATERIAL_COLORS[(start + x) * 3 + i] as f32 / 255.0; }
        m.set_color(dress_color[x], Color::new(color[0], color[1], color[2], color[3]));
    }
    m.set_float( "_S_Key_RimLight",1.00);
    m.set_float( "_S_Key_BumpAttenuation",1.00);
    m.set_float( "_LightColorToWhite",0.80);
    m.set_float( "_LightShadowToWhite",0.80);
    m.set_float( "_Preset",5.00);
    m.set_float( "_OutlineScale",4.00);
    m.set_float( "_RimLightBlend",if is_dark { 0.65 } else { 0.45});
    m.set_float( "_RimLightScale",1.00);
}
fn change_brow_color(m: &Material2, is_dark: bool) {
    let mut color = [1.0f32; 4];
    let brows = ["_EngageEmissionColor", "_EmissionColor", "_ShadowAddColor", "_ShadowColor"];
    for x1 in 0..4 {
        let mut x = x1;
        if x > 0 && !is_dark { x = x1 + 3; }
        for i in 0..3 { color[i] = DRESS_MATERIAL_COLORS[x * 3 + i] as f32 / 255.0; }
        m.set_color(brows[x], Color::new(color[0], color[1], color[2], color[3]));
    }
}