use super::*;
use crate::{
    gamedata::{EmblemPool, PersonPool},
    utils::il2str
};

pub const ENGAGE_M: [&'static str; 6] = ["Ike0AM-Sw1_c534_Engage1", "Lei0AM-Sw1_c532_Engage1 ", "Mar0AM-Sw1_c530_Engage1", "Roy0AM-Sw1_c533_Engage1", "Byl0AM-Sw1_c535_Engage1", "Sig0BM-Sw1_c531_Engage1"];
pub const ENGAGE_F: [&'static str; 6] = ["Luc0AF-Sw1_c584_Engage1", "Lyn0AF-Bw1_c581_Engage1", "Mic0AF-Mg1_c583_Engage1", "Cel0AF-Mg1_c580_Engage1", "Cor0AF-Sw1_c585_Engage1", "Eir0AF-Sw1_c582_Engage1"];

#[skyline::hook(offset=0x01bb2d80)]
pub fn asset_table_result_god_setup(
    this: AssetTable_Result,
    mode: i32,
    god_data: GodData,
    is_darkness: bool,
    conditions: Array<Il2CppString>,
    method_info: OptionalMethod
) -> AssetTable_Result {
    if god_data.is_null() { return call_original!(this, mode, god_data, is_darkness, conditions, method_info); }
    let gid = god_data.get_gid().to_rust_string();
    if gid.contains("_X_"){
        let god = GodData::get(gid.replace("GID_X_", "GID_").into());
        let darkness = god.get_flag().m_value() & 8 != 0;
        return asset_table_result_god_setup(this, mode, god, darkness, conditions, method_info);
    }
    this.set_pid(god_data.get_gid());
    let names = DVCFlags::GodNames.get_value();
    let result =
        if names {
            let result = call_original!(this, mode, god_data, is_darkness, conditions, method_info);
            let mut body_anims = vec![];
            for i in 0..4 { body_anims.push(get_result_anim(result, i).map(|v| v.to_rust_string())); }
            if !AppearanceManager::set_rand_emblem_appearance(god_data, result, mode, is_darkness) {
                for i in 0..4 {
                    if let Some(body_anim) = body_anims[i].as_ref() { set_result_anim(result, i, body_anim.as_str()); }
                }
            }
            result
        }
        else if gid.contains("リュール") {
            if let Some(person) = PersonPool::get_replacement(0, false, true){
                this.setup_4(AssetTable_Modes{value: mode}, person, conditions);
                AppearanceManager::apply_hero_mask_to_hair(this, true, mode);
                this.commit_8(new_accessory("c_trans", format!("{}Acc_Eff_EmblemAura-01-00", if mode == 1 { "o" } else { "u" })));
                this.replace(AssetTable_Modes{value: mode});
                this
            }
            else { call_original!(this, mode, GodData::get("GID_リュール".into()), is_darkness, conditions, method_info) }
        }
        else if gid.contains("ルフレ") {   // Robin
            let pid = if is_darkness { "PID_闇ルフレ" } else { "PID_ルフレ" };
            this.setup_4(AssetTable_Modes{value: mode}, PersonData::get(pid.into()), conditions)
        }
        else {
            let m002 = gid.contains("M002");
            let enemy_emblem = (god_data.get_force_type().value == 1 || m002) && !gid.contains("相手");
            if enemy_emblem {
                if let Some(god) = EmblemPool::get().enemy_emblem.iter().find(|g| g.emblem_data.hash == god_data.hash()).and_then(|v| v.get_replacement_source()){
                    let gid2 = god.get_gid().to_rust_string();
                    let result =
                        if m002 { call_original!(this, mode, god, is_darkness, conditions, method_info) }
                        else if EmblemPool::GIDS.iter().position(|x| gid2.contains(x)).is_some_and(|x| x >= 12 && x != 20) && !m002 {
                            let gid2 = gid2.replace("GID_", "GID_E006_敵");
                            let god = GodData::get(gid2.into());
                            call_original!(this, mode, god, true, conditions, method_info)
                        }
                        else { call_original!(this, mode, god, true, conditions, method_info) };
                    if !m002 {
                        result.commit_8(new_accessory("c_hip_jnt", "null"));
                        let aura_suffix = if AssetTable::s_condition_flags().m_bits().get(get_condition_index("暴走").unwrap_or(0)) { "9-00" } else { "2-00" };
                        let aura = format!("{}Acc_Eff_EmblemAura-0{}", if mode == 1 { "o" } else { "u" }, aura_suffix);
                        result.commit_8(new_accessory("c_trans", aura));
                        result.replace(AssetTable_Modes{value: mode});
                    }
                    result
                }
                else { call_original!(this, mode, god_data, true, conditions, method_info) }
            }
            else if is_darkness {
                let result =
                    if EmblemPool::GIDS.iter().position(|x| gid.contains(x)).is_some_and(|x| x >= 12 && x != 20) {
                        let gid = god_data.get_gid().to_rust_string().replace("GID_", "GID_E006_敵");
                        let god = GodData::get(gid.into());
                        call_original!(this, mode, god, true, conditions, method_info)
                    }
                    else { call_original!(this, mode, god_data, true, conditions, method_info) };

                result.commit_8(new_accessory("c_hip_jnt", "null"));
                result.commit_8(new_accessory("c_trans", format!("{}Acc_Eff_EmblemAura-02-00", if mode == 1 { "o" } else { "u" })));
                result.replace(AssetTable_Modes{value: mode});
                result
            }
            else if god_data.hash() == -316599994 {
                if let Some(replacement) = EmblemPool::get_replacement_god_3(0, false) {
                    call_original!(this, mode, replacement, is_darkness, conditions, method_info)
                }
                else { call_original!(this, mode, god_data, is_darkness, conditions, method_info) }
            }
            else {
                call_original!(this, mode, god_data, is_darkness, conditions, method_info)
            }
        };
    result.set_pid(god_data.get_gid());
    emblem_asset_rand(result, mode, god_data, is_darkness);
    result
}
fn emblem_asset_rand(result: AssetTable_Result, mode: i32, god: GodData, is_dark: bool){
    let emblem_appearance = DVCVariable::EmblemAppearance.get_value();
    let rng = DVCVariables::create_hash_rng(god.hash(), 1);
    let female = if god.is_hero() { DVCVariable::LueurGender.get_value() == 2 } else { god.get_female() == 1 };
    let menu_data = UnitAssetMenuData::get();
    let hash = rng.get_value();
    if emblem_appearance & 1 != 0 { get_outfit_data().random_body(result, mode, rng, female); }
    if emblem_appearance & 2 != 0 { for x in [0, 1, 4, 5, 6, 7] { set_color_by_i32(result, x, rng.get_value()); } }
    random_body_scale(result, Some(hash), false);
    if il2str(result.m_demo_anim()).is_none_or(|v| !v.contains("God")) {
        if female { result.set_m_demo_anim("AOC_Demo_God0F".into()) }
        else { result.set_m_demo_anim("AOC_Demo_God0M".into()); }
    }
    if il2str(result.m_hub_anim()).is_none_or(|v| !v.contains("God")) {
        if female { result.set_m_hub_anim("AOC_Hub_God0F".into()) }
        else { result.set_m_hub_anim("AOC_Hub_God0M".into()); }
    }
    if menu_data.is_preview && menu_data.god_mode { menu_data.preview.preview_data.set_result(result, mode, false, false); }
    else { UnitAssetMenuData::set_god_assets(result, mode, god, is_dark); }
}