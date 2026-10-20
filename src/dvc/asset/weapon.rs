use super::*;
use engage::List_1Ext;
use crate::utils::il2str;

pub const MAGIC: [&str; 31 ] = [
    "Fire", "Thunder", "Wind", "Surge", "WpFire", "LFire",
    "LThunder", "LWind", "LSurge", "Bolganone", "Thoron",
    "SThoron", "Excalibur", "Micaiah_Thani", "Micaiah_Shine",
    "Celica_Ragnarok", "Micaiah_Nosferatu", "Obscurite", 
    "Nova", "Meteo", "WpWind", "WpByl", "WpNature", "WpEir", 
    "WpRoy", "WpLight", "WpLer", "WpThunder", "FlameBall", "IceBreath", "IceBall"
];
pub const ROD: [&str; 16] = ["Heal", "HiHeal", "FarHeal", "Recover", "WholeHeal", "Rest", "Warp", "ReWarp", "Rescue", "Draw", "TorchRod", "Freeze", "Silence",  "Block", "Collapse", "Dance"];

pub fn edit_asset_weapon(result: AssetTable_Result, item: ItemData) {
    if result.get_body_anims().iter().any(|s| {
        let anim = s.to_rust_string();
        (anim.contains("Msn") && anim.contains("DF-Mg1")) || anim.contains("Sdp0AF-Mg1") || anim.contains("Sdk0AM-Mg1")
    })
    {
        result.set_right_hand("uWep_Mg00");
        return;
    }
    if UnitAssetMenuData::get().is_preview || item.is_null() { return; }
    let kind = item.get_kind().value;
    if kind == 8 {
        result.set_right_hand("uWep_Ft00");
        return;
    }
    if !DVCFlags::RandomWeaponAsset.get_value() { return; }
    if il2str(result.get_right_hand()).is_some_and(|v| v.contains("00")) { return; }
    let db = get_outfit_data();
    let rng = Random_2::get_system();
    let rn = rng.get_value_2(30);
    if kind != 7 && rn < 2 {
        result.set_right_hand("uBody_Msc0AT_c000");
        if kind == 6 { result.set_magic(format!("MG_{}", MAGIC[rng.get_value_2(31) as usize])); }
        return;
    }
    else if kind == 1 && rn < 5 {
        if rng.get_value_2(5) == 0 { result.set_left_hand("uBody_Msc0AT_c000"); }
        else if let Some(weapon) = db.weapons.get_random(kind) { result.set_left_hand(weapon.as_str()); }
    }
    if let Some(weapon) = db.weapons.get_random(kind){
        if kind == 4 {
            result.set_left_hand(weapon.as_str());
            if let Some(arrow) = db.weapons.get_random(8) { result.set_right_hand(arrow.as_str()) }
        }
        else if let Some(weapon) = db.weapons.get_random(kind) { result.set_right_hand(weapon.as_str()); }
        if kind == 6 { result.set_magic(format!("MG_{}", MAGIC[rng.get_value_2(31) as usize])); }
        else if kind == 7 { result.set_magic(format!("RD_{}", ROD[rng.get_value_2(16) as usize])); }
    }
}