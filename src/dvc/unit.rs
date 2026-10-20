use super::*;
pub use engage::app::{godunit::*, unitai::*, Gender, unitpool::*, godpool::*, unitedit::*, };
use engage::{
    app::{
        Random_2, AIValue_Order, GodGrowthData,
        IGodBond, IGodGrowthDataMethods, IUnitItemMethods,
        Map,
        SkillData,
    },
    GameVariableManager,
    List_1Ext,
    Mess,
};
use engage::app::AIValue;

pub mod ai;
pub mod item;
pub mod util;
pub mod job;
pub mod grow;
pub mod weapon_mask;
pub mod learn_skill;

pub use weapon_mask::*;
pub use util::*;
use crate::{
    gamedata::{EmblemPool, JobDataBase},
    dvc::unit::{grow::adjust_recruitment_stats, item::{add_item_no_dup_by_iid, adjust_unit_items}},
    utils::{clamp_value, to_opt}
};
use crate::dvc::autolevel::try_change_unit_tier_for_scale;
use crate::dvc::unit::item::{add_item_no_dup, random_items_drops};

#[unity::hook("App", "Unit", "CreateImpl2")]
pub fn unit_create_impl_2_hook(this: Unit, method_info: OptionalMethod) {
    call_original!(this, method_info);
    if this.get_person().is_null() || this.get_person().get_job().is_null() { return; }
    if DVCVariables::is_main_menu() || this.get_person().hash() == PersonPool::ILLUSORY_DOUBLE || this.is_vision() {
        return;
    }
    let person = this.get_person();
    if person.get_bmap_size() > 1 { return; }
    set_unit_lil(this, this.get_level(), this.get_internal_level());
    let leader = !this.m_private_skill().find("SID_リーダー").is_null();
    let mut ai_change = false;
    if PersonPool::is_playable(person) { adjust_player_unit(this); }
    else {
        if DVCVariable::UnitRecruitment.get_value() != 0 && PersonPool::get_enemy_player_replacement(this.get_person(), false).is_some(){
            grow::enemy_player_character_replacement(this);
            this.set_selected_weapon_from_original_aptitude(this.m_original_aptitude());
            this.update_weapon_mask();
            adjust_unit_items(this);
            this.update_weapon_mask();
        }
        if !custom_emblem_paralogue_unit(this, true) {
            if DVCFlags::RandomEnable.get_value() { ai_change = adjust_enemy_unit(this, leader); }
            else {
                try_change_unit_tier_for_scale(this);
            }
        }
    }
    ai::adjust_person_unit_ai(this);
    if ai_change { ai::adjust_unitai(this); }
    autolevel::try_autoscale_unit(this);
    post_unit_creation_adjustment(this);
}
pub fn set_unit_lil(unit: Unit, lvl: i32, il: i32) {
    unity::field_set_value_at_offset(unit, 0x94, lvl as i8);
    unity::field_set_value_at_offset(unit, 0x95, il as i8);
}
pub fn get_unit_lil(unit: Unit) -> (i32, i32) {
    (unity::field_get_value_at_offset::<i8>(unit, 0x94) as i32, unity::field_get_value_at_offset::<i8>(unit, 0x95) as i32)
}
pub fn set_enemy_adjustment(unit: Unit, adjustment: i32) {
    if unit.get_person().get_asset_force().value != 0 {
        unit.set_skill_point(0);
        unity::field_set_value_at_offset(unit, 0x1be, adjustment as i8);
        unity::field_set_value_at_offset(unit, 0x1bf, true);
    }
}
pub fn get_enemy_adjustment(unit: Unit) -> Option<i32> {
    if unity::field_get_value_at_offset::<bool>(unit, 0x1bf) && unit.get_person().get_asset_force().value != 0 {
        Some(unity::field_get_value_at_offset::<i8>(unit, 0x1be) as i32)
    }
    else { None }
}
fn custom_emblem_paralogue_unit(unit: Unit, adjust_item: bool) -> bool {
    let person = unit.get_person();
    if let Some(paralogue_god) = EmblemPool::get_paralogue_custom_emblem(person) {
        unit.m_edit().set_m_name(Mess::get(paralogue_god.get_mid()));
        unit.m_edit().set_m_gender(Gender { value: paralogue_god.get_female() as i32 + 1 });
        let arena_pid = paralogue_god.get_gid().to_rust_string().replace("GID_", "PID_闘技場_");
        if let Some(arena_emblem) = to_opt(PersonData::get(arena_pid.as_str().into())) {
            unit.set_m_job(arena_emblem.get_job());
        }
        if adjust_item {
            unit.item_put_off_all();
            let grow = GodGrowthData::try_get_from_god_data(paralogue_god);
            grow.get(0).get_style_engage_items(unit.m_job().get_style()).iter()
                .for_each(|item| {
                    let iid = format!("{}_通常", item.get_iid());
                    let normal = ItemData::get(iid.into());
                    if !normal.is_null() { unit.item_add_3(normal); }
                });
            if item::get_number_of_usable_weapons(unit) == 0 { item::adjust_missing_weapons(unit); }
        }
        true
    }
    else { false }
}

pub fn adjust_player_unit(this: Unit) {
    let class_mode = DVCVariable::ClassMode.get_value();
    let random_inventory = DVCVariable::UnitInventory.get_value() & 1 != 0;
    let random_class = class_mode > 0;
    let changed_recruit_order = DVCVariable::UnitRecruitment.get_value() != 0;
    let adjust_items = changed_recruit_order || random_class || random_inventory;
    if !adjust_items && !random_class && !changed_recruit_order { return; }
    if random_class || changed_recruit_order {
        let reclass_mode = ReclassType::get_from_settings(true);
        job::unit_reclass(this, reclass_mode);
    }
    if let Some(old_person) = PersonPool::get_replacement_2(this.get_person(), true) {
        let person = this.get_person();
        let il = if person.get_internal_level() == 0 { person.get_job().get_internal_level() } else { person.get_internal_level() } as i32;
        set_unit_lil(this, person.get_level() as i32, il);
        if person.hash() != old_person.hash() {
            if let Some(items) = to_opt(person.get_items()) {
                items.iter().for_each(|iid|{
                    if let Some(item) = to_opt(ItemData::get(iid)) {
                        let index = this.get_item_index_2(item);
                        if index != -1 { this.item_put_off(index, true); }
                    }
                });
            }
            if let Some(items) = to_opt(old_person.get_items()) {
                items.iter().for_each(|iid|{
                    if let Some(item) = to_opt(ItemData::get(iid)) { add_item_no_dup(this, item); }
                });
            }
        }
        let sp = old_person.get_skill_point();
        this.set_skill_point(if sp < 100 { 1000 } else { sp });
    }
    if adjust_items { adjust_unit_items(this); }
}
fn enemy_rate_check(value: i32, leader: bool) -> bool { value == 10 && leader || (value > 10 && Random_2::get_system().get_value_2(100) < value)}
pub fn adjust_enemy_unit(this: Unit, leader: bool) -> bool {
    let mut adjust_items = false;
    let initial_class = this.get_job().hash();
    let mut change_ai = false;
    if JobDataBase::is_monster_class(this) || this.get_person().get_bmap_size() > 1 { return false; }
    let chapter = DVCVariables::get_chapter_index();
    let usables = item::get_number_of_usable_weapons(this);
    if DVCVariables::is_main_chapter_complete(4) {
        if enemy_rate_check(DVCVariable::EnemyRevivalStone.get_value(), leader){
            let stone_count = this.get_hp_stock_count() + 1;
            this.set_m_hp_stock_count(stone_count as u8);
            this.set_m_hp_stock_count_max(stone_count as u8);
        }
        if !leader && chapter != 11 && chapter != 22 && this.get_god_unit().is_null() && this.get_person().get_engage_skill().is_null() {
            if enemy_rate_check(DVCVariable::EnemyEmblemGauge.get_value(), leader){
                if let Some(god) = EmblemPool::get_random_opp_emblem() {
                    if let Some(g_unit) = to_opt(GodPool::create(god)) {
                        g_unit.set_m_is_escaping(true);
                        if !this.try_connect_god_unit(g_unit).is_null() {
                            this.get_ai().set_sequence(AIValue_Order::attack(), "AI_AT_EngageAttack");
                            this.get_ai().set_value(AIValue_Order::attack(), 0, 1);
                            this.get_ai().set_value(AIValue_Order::attack(), 1, 1);
                            let map_completed = DVCVariables::chapter_number_complete(false) - 6;
                            let bond_level = clamp_value(map_completed, 1, 20);
                            if let Some(god_bond) = to_opt(g_unit.get_bond(this)) { god_bond.set_m_level(bond_level as u8); }
                            change_ai = true;
                        }
                    }
                }
            }
        }
        if ((this.get_person().get_asset_force().value == 0 && DVCVariable::ClassMode.get_value() == 1) || enemy_rate_check(DVCVariable::EnemyJobGauge.get_value(), leader)) && !this.get_person().get_jid().is_null() {
            adjust_items |= job::unit_reclass(this, ReclassType::Enemy); 
        }
        else { try_change_unit_tier_for_scale(this); }
        if enemy_rate_check(DVCVariable::EnemySkillGauge.get_value(), leader){
            let skill = GameData::get_random_skill_dispos(0, Random_2::get_system());
            change_ai = true;
            this.add_private_skill_2(skill);
        }
        if adjust_items {
            adjust_unit_items(this);
            if usables > 0 { item::adjust_missing_weapons(this); }
        }
    }
    if DVCVariables::is_main_chapter_complete(2) && enemy_rate_check(DVCVariable::EnemyItemDropGauge.get_value(), leader){
        let has_drop = item::has_drops(this);
        if DVCVariable::UnitInventory.get_value() & 2 != 0 && has_drop {
            for x in 0..8 {
                let item = this.get_item(x);
                if item.is_null() || !item.get_is_drop() { continue; }
                let use_type = item.get_use_type().value;
                if use_type != 24 && use_type != 23 {
                    item.ctor_3(GameData::get_random_item(RandomItemReplacementType::Drop, false));
                    item.set_is_drop(true);
                    this.item_equip();
                }
            }
        }
        random_items_drops(this);
    }
    this.item_equip();
    this.get_job().hash() != initial_class || change_ai
}
pub fn post_unit_creation_adjustment(unit: Unit) {
    let hash = unit.get_person().hash();
    if PersonPool::is_playable(unit.get_person()) && DVCVariables::is_random_map() && DVCVariables::is_main_chapter_complete(4) {
        let completed = DVCVariables::chapter_number_complete(false) * 150 + 300;
        unit.set_skill_point(completed);
    }
    match hash {
        479136587 => { add_item_no_dup_by_iid(unit, "IID_レスト"); }   // E001 Nil Restore
        1425493619 => { // E005 Elixir
            let elixir = ItemData::get("IID_特効薬".into());
            for _ in 0..3 { unit.item_add_3(elixir); };
        }
        PersonPool::LUEUR => {
            let edit = unit.m_edit();
            edit.set_m_name(GameVariableManager::get_string(DVCVariables::LUEUR_NAME));
            edit.set_m_gender(Gender{value: DVCVariable::LueurGender.get_value()});
        }
        PersonPool::VEYLE => {
            add_item_no_dup_by_iid(unit, "IID_ミセリコルデ");
            add_item_no_dup_by_iid(unit, "IID_オヴスキュリテ");
            item::magic_dagger_weapon_change(unit.get_job());
        }
        PersonPool::NEL => {
            add_item_no_dup_by_iid(unit, "IID_邪竜石");
            add_item_no_dup_by_iid(unit, "IID_邪竜石_騎馬特効")
        }
        PersonPool::RAFALE => {
            add_item_no_dup_by_iid(unit, "IID_邪竜石");
            add_item_no_dup_by_iid(unit, "IID_邪竜石_騎馬特効")
        }
        PersonPool::M002_LUMERA => {
            if DVCVariable::EnemyJobGauge.get_value() >= 10 || DVCVariable::UnitRecruitment.get_value() != 0 {
                let hero_unit = UnitPool::get_hero(false);
                let hero_job = hero_unit.get_job();
                if let Some(job_kind) = GameData::get().job_db.get_by_hash(hero_job.hash()).map(|v| v.primary_kind).filter(|v| *v != 0){
                    DVCVariable::LiberationKind.init_var(job_kind, true);
                    item::change_liberation_type();
                    let mut stats: [i32; 9] = [0; 9];
                    for x in 0..9 { stats[x] = unit.get_capability(x as i32, true); }
                    if hash != LUEUR_CLASS[0] {
                        if hero_job.has_high_jobs() {
                            unit.set_m_job(hero_job.get_high_jobs().get(0));
                            unit.m_selected_weapon_mask().set(hero_unit.get_weapon_mask().m_value());
                        }
                        else {
                            unit.set_m_job(hero_job);
                            unit.set_level(unit.get_level() + unit.get_internal_level());
                        }
                        unit.update_weapon_mask();
                        let base = unit.get_base_capability();
                        for x in 0..9 {
                            let current = unit.get_capability(x as i32, true);
                            if stats[x] < current {
                                let dbase = current - stats[x];
                                unit.set_base_capability(x as i32, base.get(x as i32) as i32 - dbase);
                            }
                        }
                    }
                }
                else { DVCVariable::LiberationKind.set_value(1); }
            }
        }
        _ => {
            if PersonPool::get().get_recruitment_index(unit.get_person()).is_some_and(|v| v > 40) {
                let items = unit.get_person().get_items();
                if !items.is_null() {
                    items.iter()
                        .flat_map(|iid| to_opt(ItemData::get(iid)))
                        .for_each(|item|{ if item.get_kind().value == 9 || item.get_flag().m_value() & 3 != 0 { add_item_no_dup(unit, item); } });
                }
            }
        }
    }
    if let Some(old_person) = PersonPool::get_replacement_2(unit.get_person(), true) {
        let hash = old_person.hash();
        unit.set_skill_point(old_person.get_skill_point());
        match hash {
            PersonPool::LUEUR => {
                if unit.get_person().hash() != PersonPool::LUEUR {
                    adjust_recruitment_stats(unit, 1).iter().enumerate().for_each(|(i, s)|{ unit.set_base_capability(i as i32, *s as i32); });
                    let name = unit.get_name();
                    unit.m_edit().set_m_name(name);
                    unit.m_edit().set_m_gender(unit.get_person().get_gender());
                }
                adjust_unit_items(unit);
                unit.set_skill_point(old_person.get_skill_point());
                add_item_no_dup_by_iid(unit, "IID_傷薬");
            }
            PersonPool::VANDRE => {
                if DVCVariables::get_chapter_index() == 1 {
                    unit.item_put_off_all();
                    let mask = unit.get_weapon_mask().get_value();
                    item::add_generic_weapons(unit);
                    if mask & 128 != 0 { add_item_no_dup_by_iid(unit, "IID_ライブ"); }
                    if DVCVariable::UnitInventory.get_value() & 1 != 0 { item::adjust_missing_weapons(unit); }
                    else { add_item_no_dup_by_iid(unit, if unit.get_capability(0, true) >= 45 { "IID_特効薬" } else { "IID_傷薬" }); }
                }
            }
            PersonPool::CLANNE|PersonPool::FRAMME => {
                if DVCVariables::get_chapter_index() == 1 {
                    unit.item_put_off_all();
                    if DVCVariable::UnitInventory.get_value() & 1 != 0 { item::adjust_missing_weapons(unit); }
                }
            }
            PersonPool::YUNAKA => add_item_no_dup_by_iid(unit, "IID_リライブ"),
            PersonPool::SEADALL => {
                if DVCVariables::is_random_map() {
                    unit.set_m_extra_hp_stock_count(1);
                    unit.set_m_extra_hp_stock_count_max(1);
                }
            }
            _ => {}
        }
    }
    unit.set_hp(unit.get_capability(0, true));
    if PersonPool::is_playable(unit.get_person()) {
        GrowthData::set_starting_class(unit);
        GrowthData::adaptive_grow(unit.get_person());
    }
    unit.item_equip();
}
pub fn enemy_check_soar(unit: Unit, dispos: bool){
    let (x, z) = if dispos { (unit.get_dispos_x(), unit.get_dispos_z()) } else { (unit.get_x(), unit.get_z()) };
    if !Map::can_enter_terrain(unit, x, z) {
        unit.add_private_skill_2(SkillData::get("SID_天駆_飛行".into()));
        unit.set_base_capability(10, -3);
    }
}