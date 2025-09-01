use bitflags::bitflags;
use engage::gamedata::{Gamedata, accessory::*, assettable::*, unit::{Unit, UnitAccessory, GodUnit}};
use unity::il2cpp::object::Array;

const ASSET_CONDITIONS: [&str; 25] = [
    "私服", "竜化", "情報", "拠点", "会話", "デモ", "クラスチェンジ中", "踊り", "砲台", "エンゲージ開始",
    "エンゲージ中", "エンゲージ技", "エンゲージ合体技", "メイン", "サブ", "残像", "弾丸", "竜石", "EID_チキ",
    "チキ", "AID_異形兵", "AID_一般兵", "拳", "RandTrans", "協力エンゲージ技",
];

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
        const Fist = 1 << 22;
        const RandomTransform = 1 << 23;
        const HousesUnite = 1 << 24;
        const Male = 1 << 28;
        const Female = 1 << 29;
        const AllyDarkEmblem = 1 << 30;
        const Transforming = 1 << 31;
        const DismountMask = 4229056;
        const NoShield = 2047;
        const EngageOutfit = 31744;
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
    // println!("GodUnit: {}", god_unit.data.mid);
}
pub fn remove_god_unit_engage_conditions(god_unit: &GodUnit, flags :&mut ConditionFlags) {
    let gid = god_unit.data.main_data.gid.to_string();
    if gid.contains("チキ") || flags.contains(ConditionFlags::Tiki) {
        flags.set(ConditionFlags::TikiEngage, true);
        flags.set(ConditionFlags::Engaged, true);
    }
   if !flags.contains(ConditionFlags::Tiki) && !flags.contains(ConditionFlags::TikiEngage) {
        if gid.contains("相手") {
            remove_condition( gid.replace("GID_相手", "EID_"));
            remove_condition("エンゲージ中");
        }
        else {
            remove_condition( gid.replace("GID_", "EID_"));
            remove_condition("エンゲージ中");
        }
    }


    // println!("GodUnit: {}", god_unit.data.mid);
}
pub fn remove_accessory_conditions(acc: &UnitAccessory) {
    if acc.index < 1 { return; }
    if let Some(acc) = AccessoryData::try_index_get_mut(acc.index) {
        let sf = AssetTableStaticFields::get();
        let index = AssetTableStaticFields::get_condition_index(acc.aid);
        if index > 0 { sf.condition_flags.bits.set(index, false); }
        if !acc.asset.is_null() {
            let index2 = AssetTableStaticFields::get_condition_index(acc.asset);
            if index2 > 0 {
                sf.condition_flags.bits.set(index2, false);
            }
        }
    }
}

pub fn remove_condition<'a>(key: impl Into<&'static Il2CppString>) {
    let sf = AssetTableStaticFields::get();
    let index = AssetTableStaticFields::get_condition_index(key.into());
    if index > 0 { 
        if sf.condition_flags.bits.get(index) { sf.condition_flags.bits.set(index, false);  }
    }
}

pub fn add_condition<'a>(key: impl Into<&'static Il2CppString>) {
    let sf = AssetTableStaticFields::get();
    let index = AssetTableStaticFields::get_condition_index(key.into());

    if index > 0 {
        if !sf.condition_flags.bits.get(index) {
            sf.condition_flags.bits.set(index, true);
        }
    }
}

pub fn set_gender_conditions(condition_unit: &Unit, flags: &mut ConditionFlags) {
    remove_condition("女装");
    remove_condition("男装");
    remove_condition("JID_邪竜ノ子");
    remove_condition("JID_M000_神竜ノ子");
    flags.set(ConditionFlags::Male, false);
    flags.set(ConditionFlags::Female, false);
    // let search_list = SEARCH_LIST.get().unwrap();
    // let jid = condition_unit.job.jid.to_string();
    if condition_unit.person.get_job().is_some_and(|j| j.parent.hash != condition_unit.job.parent.hash) {

    }

    /*
    if flags.contains(ConditionFlags::Generic) && search_list.job.iter().find(|j|
        (j.unique && j.gender_flag & 3 == 2) ||
        j.job_hash == condition_unit.job.parent.hash).is_some_and(|j| j.mount == Mount::Pegasus)
    {
        add_condition("女装");
        AssetTable::add_condition_key("女装");
        flags.set(ConditionFlags::Female, true);
        flags.set(ConditionFlags::Male, false);
        return;
    }
    if flags.contains(ConditionFlags::Generic) && search_list.job.iter().find(|j|
        j.unique && j.gender_flag & 1 == 1).is_some()
    {
        add_condition("男装");
        AssetTable::add_condition_key("男装");
        flags.set(ConditionFlags::Female, false);
        flags.set(ConditionFlags::Male, true);
        return;
    }
    */
    let gender = crate::assets::unit_dress_gender(condition_unit);
    // println!("Dress Gender for {}: {}", Mess::get_name(condition_unit.person.pid), gender);
    if gender == 1 { // Male
        add_condition("男装");
        AssetTable::add_condition_key("男装");
        flags.set(ConditionFlags::Male, true);
        flags.set(ConditionFlags::Female, false);
    }
    else if gender == 2 {   //Female
        add_condition("女装");
        AssetTable::add_condition_key("女装");
        flags.set(ConditionFlags::Female, true);
        flags.set(ConditionFlags::Male, false);
    }
}
