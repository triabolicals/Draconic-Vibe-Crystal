use super::*;
use engage::force::*;
use engage::menu::{BasicMenu, BasicMenuContent, BasicMenuMethods, BasicMenuResult, MenuItem};
use engage::menu::menus::class_change::{ClassChangeJobData, ClassChangeJobMenu, ClassChangeJobMenuItem};
use engage::unitinfo::{UnitInfo, UnitInfoSide};
use engage::unityengine::GameObject;
use crate::randomizer::data::GameData;
use crate::randomizer::DVCFlags;


pub fn update_learn_skills(forced: bool) {
    if forced || !crate::randomizer::RANDOMIZER_STATUS.read().unwrap().learn_skill {
        let force_type = [ForceType::Player, ForceType::Absent, ForceType::Dead, ForceType::Lost, ForceType::Enemy, ForceType::Ally];
        for ff in force_type { Force::get(ff).unwrap().iter().for_each(|unit| unit_update_learn_skill(unit) ); }
        let _ = crate::randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock| lock.learn_skill = true);
        return;
    }
}

pub fn unit_update_learn_skill(unit: &Unit) { 
    if unit.learned_job_skill.is_some() && unit.job.learn_skill.is_some() {
        unit.set_learned_job_skill(None);
        if let Some(skill) = unit_learn_job_skill_hook(unit, unit.job, None) {
            if DVCFlags::EquipLearnSkills.get_value() { unit.add_to_equip_skill_pool(skill); }
            unit.set_learned_job_skill(Some(skill));
        }
    }
    else { unit.try_learn_job_skill(); }
}

pub fn random_job_learn_skill(unit: &Unit, job: &JobData) -> Option<&'static SkillData> {
    let hash = ( unit.person.parent.hash >> 2 ) + ( job.parent.hash >> 2 ) + ( DVCVariables::get_seed() >> 1 );
    let rng = Random::instantiate().unwrap();
    rng.ctor(hash as u32);
    let skill_pool = &GameData::get().skill_pool;
    let available = skill_pool.non_upgrades.clone();
    let len = available.len() as i32;
    let mut count = 0;
    let personal = 
        unit.person.get_common_skills().iter()
            .find(|skill| skill.get_skill().is_some_and(|s| s.flag & 1 == 0 ))
            .map_or(0, |e| e.value & 0xFFF) as i32;
    
    let mut weapon_mask = 0;
    for x in 1..9 { if job.get_max_weapon_level(x) > 0 { weapon_mask |= 1 << x; } }

    let mut new_skill_index: i32;
    loop {  
        new_skill_index = available[ rng.get_value(len ) as usize];
        if new_skill_index == personal  { continue; }
        let skill = SkillData::try_get_hash(new_skill_index).unwrap();
        if skill_pool.job_restrictions.is_valid_for_weapon_mask(skill, weapon_mask) || count > 100 { break;  }
        count += 1;
    }
    SkillData::try_get_hash(new_skill_index)
}


#[skyline::hook(offset=0x01a3c3b0)]
pub fn unit_learn_job_skill_hook(this: &Unit, job: &JobData, _method_info: OptionalMethod) -> Option<&'static SkillData> {
    let result = call_original!(this, job, None);
    if result.is_some() {
        let new_result = get_learn_job_skill(this, job);
        this.set_learned_job_skill(new_result);
        if DVCFlags::EquipLearnSkills.get_value() && new_result.is_some() { if let Some(v) = new_result { this.add_to_equip_skill_pool(v); } }
        new_result
    }
    else { None }
}

#[unity::class("App", "ClassChangeJobMenuContent")]
pub struct ClassChangeJobMenuContent {
    junk: [u8; 0x108],
    pub help_text: &'static mut TextMeshProUGUI,
    game: &'static GameObject,
    pub skill_image: u64,
    pub skill_name: &'static mut TextMeshProUGUI,
    pub skill_level: &'static mut TextMeshProUGUI,
    pub skill_help_text: &'static mut TextMeshProUGUI,
}

pub fn class_change_job_menu_item_on_select(this: &mut ClassChangeJobMenuItem, _optional_method: OptionalMethod) {
    this.on_select();
    let old_unit = ClassChangeJobMenu::get_selected_unit_copy();
    this.pad = old_unit.person.parent.hash;
    old_unit.class_change(this.job_data.job);
    UnitInfo::set_unit(UnitInfoSide::Left, None, false, false, false, None);
    UnitInfo::set_unit(UnitInfoSide::Left, Some(old_unit), false, false, false, None);
    if !can_rand() { return; }
    let data = &this.job_data;
    let menu_content = unsafe { std::mem::transmute::<&BasicMenuContent, &ClassChangeJobMenuContent>(this.menu.menu_content) };
    set_menu_content_for_learn_skill(menu_content, data);
    
}
pub fn class_change_job_menu_item_custom_call(this: &mut ClassChangeJobMenuItem, _optional_method: OptionalMethod) -> BasicMenuResult {
    let unit = SortieSelectionUnitManager::get_unit().person.parent.hash;
    if !can_rand() || this.pad == unit { return BasicMenuResult::new(); }
    let data = &this.job_data;
    let menu_content = unsafe { std::mem::transmute::<&BasicMenuContent, &ClassChangeJobMenuContent>(this.menu.menu_content) };
    set_menu_content_for_learn_skill(menu_content, data);
    this.pad = unit;
    BasicMenuResult::new()
}
pub fn class_change_job_menu_after_build(this: &mut BasicMenu<ClassChangeJobMenuItem>, optional_method: OptionalMethod) {
    this.after_build();
    if !can_rand() { return; }
    if let Some(select) = this.get_item(this.select_index) {
        let menu_item_content = unsafe { std::mem::transmute::<&BasicMenuContent, &ClassChangeJobMenuContent>(this.menu_content) };
        set_menu_content_for_learn_skill(menu_item_content, select.job_data);
    }
}

pub fn job_menu_item_x_call(this: &mut ClassChangeJobMenuItem, _method_info: OptionalMethod) -> i32 {
    if !GameVariableManager::exist("G_JobGrowth") { GameVariableManager::make_entry_norewind("G_JobGrowth", 0); }
    let mut menu_item_content = unsafe { std::mem::transmute::<&mut BasicMenuContent, &mut ClassChangeJobMenuContent>(this.menu.menu_content) };
    match GameVariableManager::get_number("G_JobGrowth") {
        1 => {
            let stats = create_job_growth_string(this.job_data.job);
            let name = format!("{} {}", Mess::get(this.job_data.job.name), Mess::get("MID_GAMESTART_GROWMODE_SELECT_TITLE"));
            let final_str = format!("{}\n{}", name, stats).into();
            menu_item_content.help_text.set_text(final_str, true);
        },
        2 => {
            let old_unit = ClassChangeJobMenu::get_selected_unit_copy();
            old_unit.class_change(this.job_data.job);
            let stats = unit_total_growths(old_unit);
            let name = format!("{} {} {}",  Mess::get(old_unit.get_job().name), Mess::get_name(old_unit.person.pid), Mess::get("MID_GAMESTART_GROWMODE_SELECT_TITLE"));
            let final_str = format!("{}\n{}", name, stats).into();
            menu_item_content.help_text.set_text(final_str, true);
        },
        3 => {
            let new_unit = ClassChangeJobMenu::get_selected_unit_copy();
            new_unit.class_change(this.job_data.job);
            let old_unit = ClassChangeJobMenu::get_selected_unit_copy();
            let stats = unit_diff_growths(old_unit, new_unit);
            let name = format!("{}: {} -> {}", Mess::get_name(old_unit.person.pid), Mess::get(old_unit.get_job().name), Mess::get(this.job_data.job.name));
            let final_str = format!("{}\n{}", name, stats).into();
            menu_item_content.help_text.set_text(final_str, true);
        },
        _ => {
            let help_text = Mess::get(this.job_data.job.help);
            menu_item_content.help_text.set_text(help_text, true);
        },
    }
    return 0x80;
}

pub fn set_menu_content_for_learn_skill(this: &ClassChangeJobMenuContent, data: &ClassChangeJobData) {
    if let Some(skill) = get_learn_job_skill(SortieSelectionUnitManager::get_unit(), data.job) {
        this.skill_name.set_text(Mess::get(skill.name.unwrap()), false);
        if let Some(label) = skill.icon_label {
            if let Some(sprite) = GameIcon::try_get_skill(label) {
                unsafe { info_utils_try_set_sprite(this.skill_image, sprite, None) };
            }
        }
        this.skill_help_text.set_text(Mess::get(skill.help.unwrap()), false);
    }
}

#[unity::hook("App", "ClassChangeJobMenuContent", "SetJobDetails")]
pub fn class_change_job_menu_content_hook(this: &mut ClassChangeJobMenuContent, data: &ClassChangeJobData, _method_info: OptionalMethod) {
    call_original!(this, data, None);
    if !can_rand() { return; }
    set_menu_content_for_learn_skill(this, data);
}

pub fn get_learn_job_skill(unit: &Unit, job: &JobData) -> Option<&'static SkillData> {
    let job_learn_mode = DVCVariables::JobLearnMode.get_value();
    if job.learn_skill.is_none() { None }
    else if !can_rand() { job.learn_skill.and_then(|sid| SkillData::get(sid)) }
    else if job_learn_mode == 2 { random_job_learn_skill(unit, job) }
    else if job_learn_mode == 1 { SkillData::try_get_hash(GameVariableManager::get_number(format!("G_L_{}", job.jid).as_str())) }
    else { job.learn_skill.and_then(|sid| SkillData::get(sid)) }
}


#[skyline::from_offset(0x0290f730)]
pub fn info_utils_try_set_sprite(image: u64, spr: &Sprite, method_info: OptionalMethod);

fn create_job_growth_string(job: &JobData) -> String {
    let mut out = "".to_string();
    let diff = job.get_diff_grow();
    let mut count = 0;
    let stat_order = [0, 1, 6, 2, 3, 4, 5, 7, 8];
    for x in stat_order {
        let stat = if diff[x] != 0 {  format!("{}: {}%", crate::utils::get_stat_label(x), diff[x]) }
        else { format!("{}: -", crate::utils::get_stat_label(x))};
        out =
            if count == 0 { stat }
            else if count == 4 { format!("{}\n{}", out, stat) }
            else { format!("{}, {}", out, stat) };
        count += 1;
        //}
    }
    return out;
}

fn unit_total_growths(unit: &Unit) -> String {
    let mut out = "".to_string();
    let mut count = 0;
    let stat_order = [0, 1, 6, 2, 3, 4, 5, 7, 8];
    for x in stat_order {
        let stat = format!("{}: {}%", crate::utils::get_stat_label(x as usize), unit.get_capability_grow(x, false));
        out =
            if count == 0 { stat }
            else if count == 4 { format!("{}\n{}", out, stat) }
            else { format!("{}, {}", out, stat) };
        count += 1;
        //}
    }
    return out;
}

fn unit_diff_growths(before: &Unit, after: &Unit) -> String {
    let mut out = "".to_string();
    let mut count = 0;
    let stat_order = [0, 1, 6, 2, 3, 4, 5, 7, 8];
    for x in stat_order {
        let value = after.get_capability_grow(x, false) - before.get_capability_grow(x, false);
        let stat = format!("{}: {}%", crate::utils::get_stat_label(x as usize), value);
        out =
            if count == 0 { stat }
            else if count == 4 { format!("{}\n{}", out, stat) }
            else { format!("{}, {}", out, stat) };
        count += 1;
        //}
    }
    return out;

}