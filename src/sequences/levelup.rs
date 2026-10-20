use engage::{
    unity_engine::IObject_2Methods,
    app::{
        unit::*, levelupsequnece::*,
        BasicMenuItem,
        CommonRewardMenuItemContent,
        GameMessage, GameSound,
        IJobDataMethods,
        IProcInst, ISkillDataMethods,
        Mess, ResourceManager_2, IStructBase
    },
    combat::Character,
    system::Action_1,
    tm_pro::{ITMP_TextMethods, TextMeshProUGUI},
    unity_engine::{GameObject, IComponentMethods, IGameObjectMethods, IRectTransformMethods, RectTransform, Vector2},
};
use unity::{Cast, IlNull, OptionalMethod};
use crate::{DVCFlags, utils::{clamp_value, override_vtable, to_opt}};
use crate::dvc::unit::learn_skill::get_job_learn_skill_level;
use super::*;

pub fn level_up_sequence_edit(this: ProcInst){
    let descs = this.m_descs();
    if let Some(proc) = this.try_cast::<LevelUpSequnece>() {
        descs.set(0, Proc::call_2(ProcVoidMethod::from_fn(proc, prepare).unwrap()));
        descs.set(13, Proc::call_2(ProcVoidMethod::from_fn(proc, level_up_sequence_learn_job_skill).unwrap()));
    }
}

extern "C" fn level_up_sequence_learn_job_skill(this: LevelUpSequnece, _: OptionalMethod) {
    let unit = this.m_unit();
    let level = unit.get_level();
    if level < get_job_learn_skill_level(unit, unit.get_job()) { return; }
    let old_skill = to_opt(unit.m_learned_job_skill()).map(|v| v.hash()).unwrap_or(0);
    if let Some(skill) = dvc::unit::learn_skill::get_learn_job_skill(this.m_unit(), this.m_unit().get_job()).filter(|skill| skill.hash() != old_skill){
        unit.set_learned_job_skill(skill);
        if !engage::app::GameSkip::is_skip() {
            GameSound::post_event("ItemGet_Important", Character::null());
            let tag = Mess::create_skill_sprite_tag(skill);
            Mess::set_argument_2(0, tag);
            Mess::set_argument_2(1, Mess::get(skill.get_name()));
            let message = Mess::get("MID_MSG_LEARN_SKILL");
            GameMessage::create_key_wait(this, message);
        }
        if DVCFlags::EquipLearnSkills.get_value() { unit.add_to_equip_skill_pool_2(skill); }
    }
}
extern "C" fn prepare(this: LevelUpSequnece, _: OptionalMethod) {
    if this.m_is_class_change() { this.prepare(); }
    else {
        let resource_path = this.res_name_level_up();
        ResourceManager_2::load_global_async::<GameObject>(resource_path, Action_1::null());
        let unit = Unit::new(true);
        unit.clear();
        unit.copy_from(this.m_unit());
        unit.set_m_force(this.m_unit().m_force());
        let level = this.m_level();
        let new_level = this.m_unit().get_level();
        let level_ups = clamp_value(1, new_level - level, this.m_unit().get_job().get_max_level() as i32);
        unit.set_level(this.m_level());
        for _ in 0..level_ups { unit.level_up(2); }
        this.set_m_grow(unit);
    }
}
pub fn vtable_build_common_reward() {
    override_vtable("App", "CommonRewardMenuItemContent", "Build", common_reward_menu_item_content_build as _);
}
fn common_reward_menu_item_content_build(this: CommonRewardMenuItemContent, menu_item: BasicMenuItem, _: OptionalMethod) {
    let obj = this.get_game_object();
    obj.get_components_in_children_3::<RectTransform>(true).iter().for_each(|rect|{
        let name = rect.get_name().to_rust_string();
        if name == "ExpGuage"  { rect.set_size_delta(Vector2{x: 270.0, y: 12.0 }); }
        if name == "Width" { rect.set_size_delta(Vector2{x: 270.0, y: 0.0}); }
        if name == "ExpUpValue" {
            rect.set_size_delta(Vector2{x: 75.0, y: 30.0 });
            let go = rect.get_game_object();
            let text = go.get_component::<TextMeshProUGUI>();
            text.set_enable_auto_sizing(true);
        }
    });
    unsafe { CommonRewardMenuItemContent::build(this, menu_item); }

}