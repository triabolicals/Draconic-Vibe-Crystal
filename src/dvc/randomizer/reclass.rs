use super::*;
use engage::{
    app::{
        classchangejobmenu::*,
        classchange::*, basicmenuitem::*,
        classchangejobmenucontent::*, basicmenu::*,
        GameIcon, Gender, AssetTable_Modes, AssetTable_Result,
        IBitField32,
    },
    tm_pro::ITMP_TextMethods,
    unity_engine::ui::IImageMethods,
    combat::CharacterAppearance
};
use engage::app::{ClassChangeRoot, IClassChangeRoot};
use crate::{
    utils::override_vtable,
    dvc::{asset::print_asset_table_result, DVC}
};
use crate::dvc::unit::learn_skill::get_job_learn_skill_level;

fn get_unit_cap_mode(this: ClassChangeJobMenu) -> u8 { unity::field_get_value_at_offset(this, 0xc4) }
fn set_is_grow_mode(this: ClassChangeJobMenu, mode: u8) { unity::field_set_value_at_offset(this, 0xc4, mode); }

#[skyline::hook(offset=0x1ea3760)]
pub fn class_change_job_menu_set_unit_info_all(this: ClassChangeJobMenu, optional_method: OptionalMethod) {
    set_random_reclass();
    call_original!(this, optional_method);
    if let Some(item) = this.get_menu_item(this.m_select_index()).try_cast::<ClassChangeJobMenu_ClassChangeJobMenuItem>(){
        set_unit_job_grow_status_setters(this, item.get_job_data(), get_unit_cap_mode(this));
    }
}
pub fn class_change_vtable_edit() {
    override_vtable("App", "ClassChangeJobMenu.ClassChangeJobMenuItem", "BuildAttribute", no_reclass_build_attr as _);
    override_vtable("App", "ClassChangeJobMenu.ClassChangeJobMenuItem", "OnSelect", class_change_job_menu_item_on_selected as _);
    override_vtable("App", "ClassChangeJobMenu.ClassChangeJobMenuItem", "XCall", class_change_job_menu_item_x_call as _);
    override_vtable("App", "ClassChangeJobMenu", "AfterBuild", class_change_menu_after_build as _);
    override_vtable("App", "ClassChangeJobMenu", "CustomCall", class_change_custom_call as _);
    override_vtable("App", "UnitSelectSubMenu.ClassChangeMenuItem", "ACall", class_change_unit_select_a_call as _);
}
fn class_change_job_menu_item_x_call(this: ClassChangeJobMenu_ClassChangeJobMenuItem, _: OptionalMethod) -> BasicMenu_Result {
    if let Some(menu) = this.m_menu().try_cast::<ClassChangeJobMenu>() {
        let mode = get_unit_cap_mode(menu);
        if mode == 2 {
            let unit = ClassChangeJobMenu::get_unit();
            unsafe { set_unit_info_before(menu.m_root(), ClassChangeJobMenu::get_unit(), None) };
            menu.set_unit_info_after(unit, this.get_job_data());
        }
        set_unit_job_grow_status_setters(menu, this.get_job_data(), (mode + 1) % 3);
        set_is_grow_mode(menu, (mode + 1) % 3);
        BasicMenu_Result::se_cursor()
    }
    else { BasicMenu_Result::pass() }
}
fn class_change_menu_after_build(this: ClassChangeJobMenu, _: OptionalMethod) {
    unsafe { ClassChangeJobMenu::after_build(this) };
    let unit = ClassChangeJobMenu::get_unit();
    let idx = unit.get_person().index() as i16;
    if unity::field_get_value_at_offset::<i16>(this, 0xc6) != idx { unity::field_set_value_at_offset::<i16>(this, 0xc6, idx); }
    if let Some((content, menu_item)) = this.m_menu_content().try_cast().zip(this.get_menu_item(this.m_select_index()).try_cast::<ClassChangeJobMenu_ClassChangeJobMenuItem>()) {
        set_menu_content_for_learn_skill(content, menu_item.m_job_data());
    }
}
fn class_change_custom_call(this: ClassChangeJobMenu, _: OptionalMethod) -> BasicMenu_Result {
    let result = unsafe { ClassChangeJobMenu::custom_call(this) };
    let idx = ClassChangeJobMenu::get_unit().get_person().index() as i16;
    if unity::field_get_value_at_offset::<i16>(this, 0xc6) != idx {
        unity::field_set_value_at_offset::<i16>(this, 0xc6, idx);
        if let Some((content, menu_item)) = this.m_menu_content().try_cast().zip( this.get_menu_item(this.m_select_index()).try_cast::<ClassChangeJobMenu_ClassChangeJobMenuItem>()){
            set_menu_content_for_learn_skill(content, menu_item.m_job_data());
        }
    }
    result
}

fn class_change_job_menu_item_on_selected(this: ClassChangeJobMenu_ClassChangeJobMenuItem, _: OptionalMethod){
    if let Some(method) = this.get_class().raw().get_method_from_name("OnSelect", 0) {
        let on_select = unsafe { std::mem::transmute::<_, fn(ClassChangeJobMenu_ClassChangeJobMenuItem, &MethodInfo)>(method.method_ptr) };
        on_select(this, method);
    }
    let unit = ClassChangeJobMenu::get_unit();
    outfit_core::update_class_change_person(unit, this.m_job_data().get_job());
    unit.set_job(this.m_job_data().get_job());
    if DVC::get().print_events {
        print_asset_table_result(AssetTable_Result::get_from_unit(AssetTable_Modes::onmap(), unit, CharacterAppearance::conditions()), 1);
    }
    if let Some(menu) = this.m_menu().try_cast::<ClassChangeJobMenu>() {
        let idx = unit.get_person().index() as i16;
        if unity::field_get_value_at_offset::<i16>(menu, 0xc6) != idx { unity::field_set_value_at_offset::<i16>(this, 0xc6, idx); }
        let grow = get_unit_cap_mode(menu);
        if grow != 0 { set_unit_job_grow_status_setters(menu, this.m_job_data(), grow); }
    }
    if let Some(content) = this.m_menu().m_menu_content().try_cast() { set_menu_content_for_learn_skill(content, this.m_job_data()); }
}
pub fn set_menu_content_for_learn_skill(this: ClassChangeJobMenuContent, data: ClassChange_ChangeJobData) {
    let learn_mode = DVCVariable::JobLearn.get_value();
    let unit = ClassChangeJobMenu::get_unit();
    if let Some(skill) = unit::learn_skill::get_learn_job_skill(unit, data.get_job()) {
        if learn_mode != 0 {
            let name = Mess::get(skill.get_name());
            let sprite = GameIcon::try_get_skill(skill.get_icon_label());
            this.m_skill_image().set_sprite(sprite);
            this.m_skill_name().set_text_2(name, true);
            let help = Mess::get(skill.get_help());
            this.m_skill_help_text().set_text_2(help, true);
        }
        if DVCVariable::RandomLearnLevel.get_value() != 0 {
            let level = get_job_learn_skill_level(unit, data.get_job());
            engage::app::Mess::set_argument(0, level);
            let message = Mess::get("MID_SORTIE_CLASSCHANGE_SKILL_LEVEL");
            this.m_skill_get_level().set_text_2(message, true);
        }
    }
}
fn set_unit_job_grow_status_setters(menu: ClassChangeJobMenu, data: ClassChange_ChangeJobData, grow: u8) {
    let root = menu.m_root();
    if !root.is_null() {
        let new_unit = ClassChangeJobMenu::get_unit();
        let before = root.m_unit_info_before();
        let after = root.m_unit_info_after();
        if !before.is_null() { unitstatus::change_parameter_mode(before, new_unit, grow, true); }
        if !after.is_null() {
            if grow != 0 {
                new_unit.class_change(data.get_job(), ItemData::null());
                new_unit.set_selected_weapon(data.get_job_weapon_mask());
                new_unit.item_equip();
                new_unit.update_state();
            }
            unitstatus::change_parameter_mode(after, new_unit, grow, true);
        }
    }
}
fn set_random_reclass() {
    let reclass = DVCVariable::Reclassing.get_value();
    if reclass != 1 && reclass != 3 { return; }
    let unit = ClassChangeJobMenu::get_unit();
    let female = if unit.get_dress_gender().value == 2 { 17 } else { 5 };
    let dvc = DVC::get();
    if reclass == 3 {
        dvc.set_reclass.clear();
        if let Some(job) = GrowthData::get_starting_class(unit) {
            dvc.set_reclass = JobDataBase::get_all_relational(job);
            dvc.set_reclass.push(job.hash());
        }
        let mut set =
        ClassChange::get_job_list_all().iter()
            .filter(|x| x.get_job().get_flag().m_value() & female == 1 && x.get_job().is_low() && !dvc.set_reclass.contains(&x.get_job().hash()))
            .map(|x| x.get_job())
            .collect::<Vec<JobData>>();

        let rng = DVCVariables::create_hash_rng(unit.get_person().hash(), 2);
        let class_lines = if dvc.set_reclass.is_empty() { 3 } else { 2 };
        let mut count = 0;
        while count < class_lines {
            if let Some(job) = set.get_remove(rng) {
                let relational = JobDataBase::get_all_relational(job);
                if relational.len() > 0 {
                    dvc.set_reclass.extend(relational.iter());
                    count += 1;
                }
            }
            else { break; }
        }
    }
    else {
        let current_job = unit.get_job();
        let relational = JobDataBase::get_all_relational(current_job);
        let mut high = vec![];
        let mut low = vec![];
        ClassChange::get_job_list_all().iter()
            .filter(|x| x.get_job().get_flag().m_value() & female == 1 && !relational.contains(&x.get_job().hash()))
            .map(|x| x.get_job())
            .for_each(|x|{
                let hash = x.hash();
                if x.is_high() && !high.contains(&hash) { high.push(hash); }
                else if !x.is_high() && !low.contains(&hash) { low.push(hash); }
            });

        let rng = Random_2::new_2(unit.get_grow_seed());
        dvc.random_reclass[0] = low.get_remove(rng).unwrap_or(0);
        dvc.random_reclass[1] = high.get_remove(rng).unwrap_or(0);
    }
}

pub fn no_reclass_build_attr(this: ClassChangeJobMenu_ClassChangeJobMenuItem, _: OptionalMethod) -> BasicMenuItem_Attribute {
    if !DVCFlags::RandomEnable.get_value() { return this.get_attribute(); }
    let unit = ClassChangeJobMenu::get_unit();
    let current_job = unit.get_job();
    let item_job = this.get_job_data().get_job();
    let job_flags = item_job.get_flag().m_value();
    let data = this.get_job_data();
    if DVCConfig::get().debug { data.set_is_enough_item(true); }
    let item_mask = data.get_job_weapon_mask().m_value();
    let unit_mask = unit.m_aptitude().m_value();
    let mask_condition = (item_mask & unit_mask) == item_mask;
    let item_condition = data.get_is_enough_level() && data.get_is_enough_item();
    let gender = unit.get_dress_gender();
    let single_job = (DVCVariables::get_single_class(false, gender == Gender::female()).is_some() && DVCVariable::ClassMode.get_value() == 2)
        || (DVCVariables::get_single_class_gender(false, gender).is_some() && DVCVariable::ClassMode.get_value() == 5);

    let mut attribute;
    match DVCVariable::Reclassing.get_value() {
        1 => {
            let relational = JobDataBase::get_all_relational(current_job);
            let hash = item_job.hash();
            return
            if relational.contains(&hash) || hash == current_job.hash() {
                if mask_condition && item_condition { BasicMenuItem_Attribute::enable() } else { BasicMenuItem_Attribute::disable() }
            }
            else {
                if DVC::get().random_reclass.contains(&hash) {
                    if item_condition { BasicMenuItem_Attribute::enable() } else { BasicMenuItem_Attribute::disable() }
                }
                else { BasicMenuItem_Attribute::hide() }
            };
        }
        2 => {
            if single_class_can_build_attribute(current_job, item_job) {
                data.set_is_default_job(true);
                attribute = BasicMenuItem_Attribute { value: (mask_condition && item_condition) as i32 };
            }
            else { attribute = BasicMenuItem_Attribute::hide(); }
        }
        3 => {
            let hash = item_job.hash();
            attribute =
                if DVC::get().set_reclass.contains(&hash) {
                    if mask_condition && item_condition { BasicMenuItem_Attribute::enable() }
                    else { BasicMenuItem_Attribute::disable() }
                }
                else { BasicMenuItem_Attribute::hide() }
        }
        _ => {
            attribute = if mask_condition && item_condition { BasicMenuItem_Attribute::enable() } else { BasicMenuItem_Attribute::disable() };
            if single_job {
                if !single_class_can_build_attribute(current_job, item_job) { attribute = BasicMenuItem_Attribute::hide(); }
            }
        }
    }
    for dlc in ["SID_弾丸装備", "SID_エンチャント"] {
        if !item_job.get_mask_skill().find(dlc).is_null() && !DVCVariables::has_dlc(){
            attribute = BasicMenuItem_Attribute::hide();
        }
    }
    if (gender == Gender::male() && (job_flags & 20 == 4)) || (gender == Gender::female() && job_flags & 20 == 16) || job_flags & 32 != 0 || job_flags & 3 == 0  {
        attribute = BasicMenuItem_Attribute::hide();
    }
    if attribute.value == 1 { this.m_job_data().set_is_gender(true); }
    IClassChangeJobMenu_ClassChangeJobMenuItem::set_m_attribute(this, attribute);
    attribute
}
fn single_class_can_build_attribute(current_job: JobData, item_job: JobData) -> bool {
    let hash = item_job.hash();
    let jid = item_job.get_jid().to_string().trim_end_matches("_E").to_string();
    let unit_jid = current_job.get_jid().to_string().trim_end_matches("_E").to_string().trim_end_matches("下級").to_string();
    if jid.contains(&unit_jid) || ( jid.contains(&unit_jid) && jid.contains("下級") ) || (jid.contains("メリュジーヌ") && unit_jid.contains("メリュジーヌ")) {
        true
    }
    else if current_job.is_high() {
       JobData::get_list().iter()
            .filter(|x| x.is_low() && x.has_high_jobs())
            .filter(|x| x.get_high_jobs().iter().any(|h| h.hash() == current_job.hash()))
            .any(|x| x.get_high_jobs().iter().any(|h| h.hash() == hash) || x.hash() == hash)
    }
    else if current_job.is_low() && current_job.has_high_jobs() { current_job.get_high_jobs().iter().any(|x| x.hash() == hash) }
    else { false }
}
fn class_change_unit_select_a_call(this: BasicMenuItem, optional_method: OptionalMethod) -> BasicMenu_Result {
    set_random_reclass();
    unsafe { unit_select_sub_menu_cc_a_call(this, optional_method) }
}
#[skyline::from_offset(0x1b56e20)]
fn unit_select_sub_menu_cc_a_call(this: BasicMenuItem, optional_method: OptionalMethod) -> BasicMenu_Result;

#[skyline::from_offset(0x1ea47d0)]
fn set_unit_info_before(this: ClassChangeRoot, unit: Unit, optional_method: OptionalMethod);