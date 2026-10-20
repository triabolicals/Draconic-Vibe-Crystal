use bitflags::bitflags;
use engage::app::{
    unitinfo::*, unititem::*,
    helpitemunititem::*, helpitembase::*,
    itemmenudetailsetter::*, helpparamsetter::*,
    Random_2
};
use engage::tm_pro::ITMP_TextMethods;
use crate::utils::{clamp_value, override_vtable, override_vtable_by_slot};
use super::*;

pub struct DefaultWeaponData {
    pub hash: i32,
    pub weapon_stats: WeaponStats,
    flag: i32,
    attr: u8,
    kind: i32,
    equip: SkillsList,
    give: SkillsList,
}
bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct RandomWeaponSkills: i32 {
        const Brave = 1;
        const ArtDamage = 2;
        const BulletDamage = 4;
        const Smash = 8;
        const NoFollowUp = 16;
        const Liberation = 32;
        const ChainAttack = 64;
        const DoubleExp = 128;
        const MoneyDrop = 256;
        const Poison = 512;
        const Inverted = 1024;
        const ReverseInteraction = 2048;
        const Usable = 4096;
    }
}
impl RandomWeaponSkills {
    const SIDS: [&'static str; 10] = [
        "SID_２回行動", // Brave
        "SID_気功", // Arts
        "SID_弾丸攻撃力", // Bullet
        "SID_スマッシュ", // Smash
        "SID_追撃不可", // No Follow Up,
        "SID_撃破時エンゲージカウント＋１",   // EngageMeter+1
        "SID_チェインアタック許可",   // Chain Attack
        "SID_経験値２倍",    // Double EXP
        "SID_お金入手_500G",
        "SID_毒",
    ];
    pub const DESCRIPTION: [&'static str; 15] = [
        "If user initiates combat, attacks\ntwice.",
        "Attack is affected by Str and Mag.",
        "Attack is affected by Dex.",
        "Smashes foes.\nCannot strike first if initiating combat.",
        "Cannot follow up.",
        "Fills engage meter by 1 when user\ndefeats a foe.",
        "Allow unit to chain attack.",
        "Doubles user's earned experience.",
        "May obtain 500G when unit defeats\na foe. Trigger %=Lck.",
        "Poisons hit foes.",
        "Deals magical damage.",
        "Inverts weapon interactions.",
        "Disappears when consumed as an item,\nbut recovers user's HP.",
        "Deals physical damage.",
        ""
    ];
    pub fn get_description(self, item_data: ItemData) -> &'static str {
        let bits = self.bits();
        for i in 0..13 {
            if bits & (1 << i) != 0 {
                return
                if i == Self::SIDS.len() {
                    if item_data.get_attr() == ItemData_Attrs::physical() { Self::DESCRIPTION[i+3] } else { Self::DESCRIPTION[i] }
                }
                else { Self::DESCRIPTION[i] };
            }
        }
        Self::DESCRIPTION[14]
    }
    pub fn random_effect(item_data: ItemData, rng: Random_2) -> Option<Self> {
        const SID_COUNT: i32 = RandomWeaponSkills::SIDS.len() as i32;
        if !item_data.is_weapon() { None }
        else {
            let flag = item_data.get_flag().m_value();
            if flag & 128 != 0 || rng.get_value_2(2) == 1 { None }
            else {
                let equip_skill = item_data.get_equip_skills();
                let give = item_data.get_give_skills();
                let mut avail: Vec<i32> = vec![];
                for i in 0..Self::SIDS.len() - 1 {
                    if i == 3 && item_data.get_range_i() > 1 { continue; }
                    if equip_skill.find(Self::SIDS[i]).is_null() { avail.push(i as i32); }
                }
                if let Some(last) = Self::SIDS.last() {
                    if give.find(*last).is_null() { avail.push(SID_COUNT - 1); }
                }
                if flag & 65536 == 0 { avail.push(SID_COUNT); }
                if flag & 2048 == 0 { avail.push(SID_COUNT + 1); }
                if flag & 4 == 0 { avail.push(SID_COUNT + 2); }
                if let Some(&v) = avail.get_random_element(rng) { Self::from_bits(1 << v) } else { None }
            }
        }
    }
    pub fn add_random_skill(self, item_data: ItemData) {
        let flag = item_data.get_flag().m_value();
        let equip = item_data.get_equip_skills();
        let give = item_data.get_give_skills();
        let bits = self.bits();
        for v in 0..13 {
            if bits & (1 << v) != 0 {
                match v {
                    0..9 => { equip.add(Self::SIDS[v as usize], SkillData_Categorys::equip(), 0); }
                    9 => { give.add(Self::SIDS[9], SkillData_Categorys::item(), 0); }
                    10 => {
                        item_data.get_flag().set_m_value(65536|flag);
                        if item_data.get_kind() == ItemData_Kinds::magic() { item_data.set_attr(ItemData_Attrs::physical()); }
                        else { item_data.set_attr(ItemData_Attrs::magic()); }
                    }
                    11 => { item_data.get_flag().set_m_value(2048|flag); }
                    12 => { item_data.get_flag().set_m_value(4|flag); }
                    _ => {}
                }
                return;
            }
        }
    }
}
impl DefaultWeaponData {
    pub fn new(item_data: ItemData) -> Self {
        let flag = item_data.get_flag().m_value();
        let attr = item_data.get_attr().value as u8;
        Self {
            kind: item_data.get_kind().value,
            hash: item_data.hash(),
            flag, attr,
            weapon_stats: WeaponStats::new(item_data),
            equip: SkillsList::from_skill_array(item_data.get_equip_skills()),
            give: SkillsList::from_skill_array(item_data.get_equip_skills()),
        }
    }
    pub fn reset(&self) {
        let item = ItemData::try_get_from_hash(self.hash);
        self.weapon_stats.reset(item);
        item.get_flag().set_m_value(self.flag);
        item.set_kind(ItemData_Kinds{value: self.kind});
        item.set_attr(ItemData_Attrs{value: self.attr as i32});
        self.equip.set_skill_array(item.get_equip_skills());
        self.give.set_skill_array(item.get_equip_skills());
    }
    pub fn randomize(&self, rng: Random_2){
        self.reset();
        let item = ItemData::try_get_from_hash(self.hash);
        if DVCFlags::WeaponStats.get_value() {
            let rank = item.m_weapon_level().value;
            let spread = 2 * (rank + 3);
            item.set_power(clamp_value((rng.get_value_2(spread) - 2*rank) as u8 + self.weapon_stats.might, 0, 100));
            item.set_weight(clamp_value((rng.get_value_2(spread) - 2*rank) as u8 + self.weapon_stats.weight, 0, 100));
            let dcrit = 5*(rng.get_value_2(8+2*rank) - 4) as i16;
            let rn_hit = rng.get_value_2(21) as i16;
            let hit =
                match rn_hit {
                    0..11 => 60 + (rn_hit) * 5,
                    11..20 => { self.weapon_stats.hit + (rn_hit - 15) * 5 }
                    _ => { 255 }
                };
            item.set_hit(hit);
            let dsecure = 5 * (rng.get_value_2(spread) - 2 * rank) as i16;
            let davoid = 5 * (rng.get_value_2(spread) - 2 * rank) as i16;
            item.set_critical(self.weapon_stats.crit + dcrit);
            item.set_avoid(self.weapon_stats.avoid + davoid);
            item.set_secure(self.weapon_stats.secure + dsecure);
        }
    }
}

#[derive(Clone)]
pub struct WeaponStats {
    pub might: u8,
    pub crit: i16,
    pub hit: i16,
    pub avoid: i16,
    pub secure: i16,
    pub weight: u8,
}

impl WeaponStats {
    pub fn new(item_data: ItemData) -> Self {
        Self {
            might: item_data.get_power(),
            crit: item_data.get_critical(),
            hit: item_data.get_hit(),
            avoid: item_data.get_avoid(),
            secure: item_data.get_secure(),
            weight: item_data.get_weight(),
        }
    }
    pub fn reset(&self, item: ItemData) {
        item.set_power(self.might);
        item.set_critical(self.crit);
        item.set_hit(self.hit);
        item.set_avoid(self.avoid);
        item.set_secure(self.secure);
        item.set_weight(self.weight);
    }
}
pub fn item_help_edit() {
    override_vtable("App", "HelpItemUnitItem", "SetContents", help_item_unit_item_set_contents as _);
    override_vtable("App", "HelpItemEquipItem", "SetContents", help_item_equip_set_contents as _);
    override_vtable_by_slot("App", "ItemMenuDetailSetter", 4, item_data_setter_data_4 as _);
    override_vtable_by_slot("App", "ItemMenuDetailSetter", 5, item_data_setter_data_5 as _);
    override_vtable_by_slot("App", "ItemMenuDetailSetter", 6, item_data_setter_data_6 as _);
    override_vtable_by_slot("App", "ItemMenuDetailSetter", 7, item_data_setter_data_7 as _);
}
fn help_item_set_content(setter: HelpParamSetter, unit: Option<Unit>, index: i32, item: UnitItem) {
    if !DVCFlags::WeaponSkill.get_value() { return; }
    let item_hash;
    if !item.is_null() {
        let item = item.m_item();
        if item.is_null() { return }
        item_hash = item.hash();
    }
    else if let Some(unit) = unit {
        let item = unsafe { get_engage_item_data(unit, index, None) };
        if item.is_null() { return; }
        let item = item.m_item();
        item_hash = item.hash();
    }
    else { return; }
    if let Some(d) = RandomizedGameData::get_read().random_item_skills.get(&item_hash) {
        let item = ItemData::try_get_from_hash(item_hash);
        let mut text = setter.m_contents_text().get_text().to_rust_string() + "\n<color=#00FF00>";
        text += d.get_description(item);
        text += "</color>";
        setter.m_contents_text().set_text_2(text, true);
    }
}
fn item_data_set_data(setter: ItemMenuDetailSetter, item: UnitItem) {
    if !DVCFlags::WeaponSkill.get_value() || item.is_null() || item.m_item().is_null() { return; }
    let item = item.m_item();
    if let Some(d) = RandomizedGameData::get_read().random_item_skills.get(&item.hash()) {
        let mut text = setter.m_contents_text().get_text().to_rust_string() + "\n<color=#00FF00>";
        text += d.get_description(item);
        text += "</color>";
        setter.m_contents_text().set_text_2(text, true);
    }
}

pub fn help_item_unit_item_set_contents(this: HelpItemUnitItem, setter: HelpParamSetter, _: OptionalMethod) {
    unsafe { HelpItemUnitItem::set_contents(this, setter); }
    let unit = if this.m_is_temp_unit() { to_opt(this.m_temp_unit()) } else { to_opt(UnitInfo::get_unit(UnitInfo_Side::left())) };
    help_item_set_content(setter, unit, this.m_index(), this.m_unit_item());
}
pub fn help_item_equip_set_contents(this: HelpItemUnitItem, setter: HelpParamSetter, optional_method: OptionalMethod) {
    unsafe { help_item_equip_item_set_contents_original(this, setter, optional_method); }
    let unit = if this.m_is_temp_unit() { to_opt(this.m_temp_unit()) } else { to_opt(UnitInfo::get_unit(UnitInfo_Side::left())) };
    help_item_set_content(setter, unit, this.m_index(), this.m_unit_item());
}
pub fn item_data_setter_data_4(this: ItemMenuDetailSetter, selected: UnitItem, _: OptionalMethod) {
    unsafe { ItemMenuDetailSetter::set_data(this, selected) };
    item_data_set_data(this, selected)

}
pub fn item_data_setter_data_5(this: ItemMenuDetailSetter, unit: Unit, selected: UnitItem, _: OptionalMethod){
    unsafe { ItemMenuDetailSetter::set_data_2(this, unit, selected) };
    item_data_set_data(this, selected)
}
pub fn item_data_setter_data_6(this: ItemMenuDetailSetter, base: UnitItem, target: UnitItem, unit: Unit, _: OptionalMethod){
    unsafe { ItemMenuDetailSetter::set_data_3(this, base, target, unit) };
    item_data_set_data(this, target);
}
pub fn item_data_setter_data_7(this: ItemMenuDetailSetter, base: UnitItem, kind: ItemMenuDetailSetter_Kind, value: i32, _: OptionalMethod){
    unsafe { ItemMenuDetailSetter::set_data_4(this, base, kind, value) };
    item_data_set_data(this, base);
}
#[skyline::from_offset(0x297d160)]
fn help_item_equip_item_set_contents_original(this: HelpItemUnitItem, setter: HelpParamSetter, optional_method: OptionalMethod);

#[skyline::from_offset(0x2912810)]
fn get_engage_item_data(unit: Unit, index: i32, optional_method: OptionalMethod) -> UnitItem;