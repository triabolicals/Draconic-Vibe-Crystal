use bitflags::bitflags;
use engage::gamedata::{Gamedata, accessory::*, assettable::*, unit::{Unit, UnitAccessory, GodUnit}};
use unity::il2cpp::object::Array;
use super::data::{job::Mount, SEARCH_LIST};

const ASSET_CONDITIONS: [&str; 22] = ["私服", "竜化", "情報", "拠点", "会話", "デモ", "クラスチェンジ中", "踊り", "砲台", "エンゲージ開始", "エンゲージ中", "エンゲージ技", "エンゲージ合体技", "メイン", "サブ", "残像", "弾丸", "竜石", "EID_チキ", "チキ", "AID_異形兵", "AID_一般兵"];

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ConditionFlags: i32 {
        const None = 0;
        const CausalClothes = 1 << 0;
        const Transform = 1 << 1;
        const UnitInfo = 1 << 2;
        const Hub = 1 << 3;
        const Talk = 1 << 4;
        const Demo = 1 << 5;
        const ClassChange = 1 << 6;
        const Dance = 1 << 7;
        const Ballista = 1 << 8;
        const Engaging = 1 << 9;
        const Engaged = 1 << 10;
        const EngageAttack = 1 << 11;
        const EngageComboAttack = 1 << 12;
        const EngageAttackComboMain = 1 << 13;
        const EngageAttackComboSub = 1 << 14;
        const Vision = 1 << 15;
        const Bullet = 1 << 16;
        const DragonStone = 1 << 17;
        const TikiEngage = 1 << 18;
        const Tiki = 1 << 19;
        const Corrupted = 1 << 20;
        const Generic = 1 << 21;
        const Male = 1 << 28;
        const Female = 1 << 29;
        const AllyDarkEmblem = 1 << 30;
        const Transforming = 1 << 31;
        const DismountMask = (1 << 6 ) | (1 << 7 ) | (1 << 8) | ( 1 << 9 ) | (1 << 10 ) | ( 1 << 15 );
        const TransformedMask = ( 1 << 18 ) | ( 1 << 19) | 2;
    }
}

impl ConditionFlags {
    pub fn get_from_conditions(conditions: &Array<&Il2CppString>) -> ConditionFlags {
        let mut out: i32 = 0;
        let sf = AssetTableStaticFields::get();
        conditions.iter().for_each(|con|{
            let con_str = con.to_string();
            if let Some(pos) = ASSET_CONDITIONS.iter().position(|&m| m == con_str) {
                out |= 1 << (pos as i32);
            }
        });
        for x in 0..ASSET_CONDITIONS.len(){
            let index = AssetTableStaticFields::get_condition_index(ASSET_CONDITIONS[x]);
            if index > 0 && sf.condition_flags.bits.get(index) { out |= 1 << (x as i32); }
        }
        ConditionFlags::from_bits(out).unwrap()
    }
}

pub fn add_god_unit_engage_conditions(god_unit: &GodUnit, flags :&mut ConditionFlags) {
    let gid = god_unit.data.main_data.gid.to_string();
    if gid.contains("チキ") { 
        flags.set(ConditionFlags::TikiEngage, true);
    }
    if gid.contains("相手") {  AssetTable::add_condition_key( gid.replace("GID_相手", "EID_") ); }
}

pub fn remove_accessory_conditions(acc: &UnitAccessory) {

    if acc.index < 1 { return; }
    if let Some(acc) = AccessoryData::try_index_get_mut(acc.index) {
        let sf = AssetTableStaticFields::get();
        let index = AssetTableStaticFields::get_condition_index(acc.aid);
        if index > 0 { sf.condition_flags.bits.set(index, false); }
        if !acc.asset.is_null() {
            let index2 = AssetTableStaticFields::get_condition_index(acc.asset);
            if index2 > 0 {  sf.condition_flags.bits.set(index2, false);  }
        }
    }
}

pub fn remove_condition<'a>(key: impl Into<&'a Il2CppString>) {
    let sf = AssetTableStaticFields::get();
    let index = AssetTableStaticFields::get_condition_index(key.into());
    if index > 0 { 
        if sf.condition_flags.bits.get(index) { sf.condition_flags.bits.set(index, false);  }
    }
}

pub fn add_condition<'a>(key: impl Into<&'a Il2CppString>) {
    let sf = AssetTableStaticFields::get();
    let index = AssetTableStaticFields::get_condition_index(key.into());

    if index > 0 { 
        if !sf.condition_flags.bits.get(index) { sf.condition_flags.bits.set(index, true);  }
    }
}

pub fn set_gender_conditions(condition_unit: &Unit, flags: &mut ConditionFlags) {
    remove_condition("女装");
    remove_condition("男装");
    flags.set(ConditionFlags::Male, false);
    flags.set(ConditionFlags::Female, false);
    if SEARCH_LIST.get().unwrap().job.iter().find(|j| j.job_hash == condition_unit.job.parent.hash).is_some_and(|j| j.mound == Mount::Pegasus){
        add_condition("女装");
        AssetTable::add_condition_key("女装");
        flags.set(ConditionFlags::Female, true);
        return;
    }

    let gender = crate::assets::unit_dress_gender(condition_unit);
    if gender == 1{
        add_condition("男装");
        AssetTable::add_condition_key("男装");
        flags.set(ConditionFlags::Male, true);
        flags.set(ConditionFlags::Female, false);
    }
    else if gender == 2 {
        add_condition("女装");
        AssetTable::add_condition_key("女装");
        flags.set(ConditionFlags::Female, true);
        flags.set(ConditionFlags::Male, false);
    }
}
