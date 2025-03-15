use super::*;
use std::sync::OnceLock;
pub static JOB_RESTRICT_SKILLS_LIST: OnceLock<Vec<SkillWeaponRestrictions>> = OnceLock::new();
use engage::force::*;
pub struct SkillWeaponRestrictions {
    pub hash: i32,
    pub mask: i32,
}

pub fn update_learn_skills(forced: bool) {
    println!("LearnSkillKey: {}", GameVariableManager::get_number(DVCVariables::JOB_LEARN_SKILL_KEY));
    if forced || !crate::randomizer::RANDOMIZER_STATUS.read().unwrap().learn_skill {
        let force_type = [ForceType::Player, ForceType::Absent, ForceType::Dead, ForceType::Lost, ForceType::Enemy, ForceType::Ally];
        for ff in force_type { Force::get(ff).unwrap().iter().for_each(|unit| unit_update_learn_skill(unit) ); }
        let _ = crate::randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock| lock.learn_skill = true);
        return;
    }
}


pub fn unit_update_learn_skill(unit: &Unit) { 
    if unit.learned_job_skill.is_some() && unit.job.learn_skill.is_some() {
        unit.set_learn_skill(None);
        crate::randomizer::skill::learn::unit_learn_job_skill_hook(unit, unit.job, None);
        //if let Some(skill) = crate::randomizer::skill::learn::unit_learn_job_skill_hook(unit, unit.job, None) {
        //    println!("{} Learned {}", Mess::get_name(unit.person.pid), Mess::get(skill.name.unwrap()));
       // }
    }
    else { unit.try_learn_job_skill(); }
}

pub struct JobLearnSkillMode;
impl ConfigBasicMenuItemSwitchMethods for JobLearnSkillMode {
    fn init_content(_this: &mut ConfigBasicMenuItem){
        if !DVCVariables::is_main_menu() {
            GameVariableManager::make_entry_norewind(DVCVariables::JOB_LEARN_SKILL_KEY, 0); 
            GameVariableManager::make_entry_norewind("LSkC", GameVariableManager::get_number(DVCVariables::JOB_LEARN_SKILL_KEY)); 
        }
    }
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let is_main = DVCVariables::is_main_menu();
        if is_main && !CONFIG.lock().unwrap().random_skill {
            this.help_text = "Enable skill randomization to enable this setting.".into();
            Self::set_command_text(this, None);
            this.update_text();
            return BasicMenuResult::new();
        }
        let value = if is_main  { CONFIG.lock().unwrap().learn_skill } 
            else { GameVariableManager::get_number("LSkC") };

        let result = ConfigBasicMenuItem::change_key_value_i(value, 0, 3, 1);
        if value != result {
            if is_main {  CONFIG.lock().unwrap().learn_skill = result; } 
            else { GameVariableManager::set_number("LSkC", result); }
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){ 
        let is_main = DVCVariables::is_main_menu();
        if is_main && !CONFIG.lock().unwrap().random_skill {
            this.help_text = "Enable skill randomization to enable this setting.".into();
            this.command_text = "Disabled".into();
            return;
        }
        let value = if is_main {  CONFIG.lock().unwrap().learn_skill }
            else { GameVariableManager::get_number("LSkC") };

        let changed = DVCVariables::changed_setting_text("LSkC", DVCVariables::JOB_LEARN_SKILL_KEY);
        this.command_text = format!("{}{}", changed, 
            match value {
                1 => { "Player"},
                2 => { "Enemy"},
                3 => { "All"},
                _ => { "Default"},
            }
        ).into();
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){ 
        let is_main = DVCVariables::is_main_menu();
        if is_main && !CONFIG.lock().unwrap().random_skill {
            this.help_text = "Enable skill randomization to enable this setting.".into();
            return;
        }
        let value = if DVCVariables::is_main_menu()  { CONFIG.lock().unwrap().learn_skill }  else { GameVariableManager::get_number("LSkC") };
        let changed = if DVCVariables::is_main_menu() { "" }
            else if GameVariableManager::get_number("LSkC") != GameVariableManager::get_number(DVCVariables::JOB_LEARN_SKILL_KEY) { " (A to Confirm)"}
            else { "" };

        this.help_text = format!("{}{}",
            match value {
                1 => { "Playable units will have randomized class skill learnset."},
                2 => { "Enemy units will have randomized class skill learnset."},
                3 => { "All units will have randomized class skill learnset."},
                _ => { "No randomized unit class skill learnset."}
            },
            changed
        ).into();
    }
}


pub struct LearnSkillConfirm;
impl TwoChoiceDialogMethods for LearnSkillConfirm {
    extern "C" fn on_first_choice(this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> BasicMenuResult {
        GameVariableManager::set_number(DVCVariables::JOB_LEARN_SKILL_KEY, GameVariableManager::get_number("LSkC"));
        let menu = unsafe {  std::mem::transmute::<&mut engage::proc::ProcInst, &mut engage::menu::ConfigMenu<ConfigBasicMenuItem>>(this.parent.parent.menu.proc.parent.as_mut().unwrap()) };
        let index = menu.select_index;
        JobLearnSkillMode::set_help_text(menu.menu_item_list[index as usize], None);
        JobLearnSkillMode::set_command_text(menu.menu_item_list[index as usize], None);
        update_learn_skills(true);
        menu.menu_item_list[index as usize].update_text();
        BasicMenuResult::se_cursor().with_close_this(true)
    }
    extern "C" fn on_second_choice(_this: &mut BasicDialogItemNo, _method_info: OptionalMethod) -> BasicMenuResult { BasicMenuResult::new().with_close_this(true) }
}

pub fn lsk_acall(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    if GameVariableManager::get_number("LSkC") == GameVariableManager::get_number(DVCVariables::JOB_LEARN_SKILL_KEY) { return BasicMenuResult::new();}
    YesNoDialog::bind::<LearnSkillConfirm>(this.menu, "Change Randomization Setting?", "Do it!", "Nah..");
    return BasicMenuResult::new();
}

pub extern "C" fn vibe_learn_skill() -> &'static mut ConfigBasicMenuItem {  
    let skill = ConfigBasicMenuItem::new_switch::<JobLearnSkillMode>("Random Class Learn Skills");
    skill.get_class_mut().get_virtual_method_mut("BuildAttribute").map(|method| method.method_ptr = crate::menus::buildattr::skill_gauge_build_attr as _);
    skill.get_class_mut().get_virtual_method_mut("ACall").map(|method| method.method_ptr = lsk_acall as _);
    skill
}


pub fn initialize_job_skill_restrictions() {
    JOB_RESTRICT_SKILLS_LIST.get_or_init(||{
        let mut list: Vec<SkillWeaponRestrictions> = Vec::new();
        include_str!("restrict.txt").lines()
            .into_iter()
            .for_each(|line|{
                let new_line: Vec<_> = line.split_whitespace().collect();
                if let Some(skill) = SkillData::get(new_line[0]) {
                    let hash1 = skill.parent.hash;
                    let mut mask1 = 0;
                    for x in 1..new_line.len() {
                        if let Ok(number) = new_line[x].parse::<i32>() { mask1 |= 1 << number; }
                    }
                    if mask1 > 0 { list.push( SkillWeaponRestrictions { hash: hash1, mask: mask1}); }
                }
            }
        );
        SkillData::get_list_mut().unwrap().iter_mut().for_each(|skill|{
            let weapon_restrict = skill.get_weapon_prohibit().value;
            if weapon_restrict != 0 && skill.get_flag() & 63 == 0 {
                let hash1 =  skill.parent.hash;
                if !list.iter().any(|s| s.hash == hash1) {
                    list.push( SkillWeaponRestrictions { hash: hash1, mask: 1024 - weapon_restrict }); 
                }
            }
        });
        println!("{} skills in the restrict skills list.", list.len());
        list
    });
}

pub fn random_job_learn_skill(unit: &Unit, job: &JobData) -> Option<&'static SkillData> {
    let hash = ( unit.person.parent.hash >> 2 ) + ( job.parent.hash >> 2 ) + ( DVCVariables::get_seed() >> 1 );
    let rng = Random::instantiate().unwrap();
    rng.ctor(hash as u32);
    let available: Vec<_> = SKILL_POOL.lock().unwrap().iter()
        .filter(|&skill|  SkillData::try_index_get(skill.index).is_some_and(|s| !s.can_override_skill() && !s.is_style_skill()))
        .map(|skill| skill.index).collect();
    let len = available.len() as i32;
    let mut count = 0;
    let personal = unit.person.get_common_skills().iter()
        .find(|skill| skill.get_skill().is_some_and(|s| s.get_flag() & 1 == 0 ))
        .map_or(0, |e| e.value & 0xFFF) as i32;

    let mut weapon_mask = 0;
    for x in 1..9 { if job.get_max_weapon_level(x) > 0 { weapon_mask |= 1 << x; } }

    let mut new_skill_index: i32;
    let restriction_list = JOB_RESTRICT_SKILLS_LIST.get().unwrap();
    loop {  
        new_skill_index = available[ rng.get_value(len ) as usize];
    // Skill cannot be personal or current job skill
        if new_skill_index == personal  { continue; }
        let skill = SkillData::try_index_get(new_skill_index).unwrap();
        if count == 50 { break; }
        let hash = skill.parent.hash;
        if let Some(restrict) = restriction_list.iter().find(|x| x.hash == hash) {
            if restrict.mask & weapon_mask != 0 { break; }
            else { count += 1;  }
        }
        else { break; }
    }
    SkillData::try_index_get(new_skill_index)
}


#[skyline::hook(offset=0x01a3c3b0)]
pub fn unit_learn_job_skill_hook(this: &Unit, job: &JobData, _method_info: OptionalMethod) -> Option<&'static SkillData> {
    let result = call_original!(this, job, None);
    let mode = GameVariableManager::get_number(DVCVariables::JOB_LEARN_SKILL_KEY);
    if !can_rand() || !GameVariableManager::get_bool(DVCVariables::SKILL_KEY) { return result; }
    if ( mode & 1 == 0 && this.person.get_asset_force() == 0 ) || ( mode & 2 == 0 && this.person.get_asset_force() != 0 ) || result.is_none() { return result; }
    let new_result = random_job_learn_skill(this, job);
    this.set_learn_skill(new_result);
    new_result
}

#[unity::class("App", "ClassChangeJobMenuContent")]
pub struct ClassChangeJobMenuContent {
    junk: [u8; 0x118],
    pub skill_image: u64,
    pub skill_name: &'static mut TextMeshProUGUI,
    pub skill_level: &'static mut TextMeshProUGUI,
    pub skill_help_text: &'static mut TextMeshProUGUI,
}

#[unity::hook("App", "ClassChangeJobMenuContent", "SetJobDetails")]
pub fn class_change_job_menu_content_hook(this: &mut ClassChangeJobMenuContent, data: &crate::randomizer::job::ChangeJobData, _method_info: OptionalMethod) {
    call_original!(this, data, None);
    if data.job.learn_skill.is_none() || !can_rand() || !GameVariableManager::get_bool(DVCVariables::SKILL_KEY)  { return; }
    if GameVariableManager::get_number(DVCVariables::JOB_LEARN_SKILL_KEY) & 1 == 0 { return; }
    if let Some(skill) = random_job_learn_skill(SortieSelectionUnitManager::get_unit(), data.job) {
        this.skill_name.set_text(Mess::get(skill.name.unwrap()), false);
        if let Some(label) = skill.icon_label {
            if let Some(sprite) = GameIcon::try_get_skill(label) {
                unsafe { info_utils_try_set_sprite(this.skill_image, sprite, None) };
            }
        }
        this.skill_help_text.set_text(Mess::get(skill.help.unwrap()), false);
    }
}

#[skyline::from_offset(0x0290f730)]
pub fn info_utils_try_set_sprite(image: u64, spr: &Sprite, method_info: OptionalMethod); 