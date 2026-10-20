use engage::{
    system::collections::generic::IList_1Methods,
    app::{
        IStructDataArray_1Methods,
        GameSound,
        IStructBase, IUnitInfoParamManager,
        Pad, UnitInfoParamManager,
        unit::*, unitstatus::*, unitstatussetter::*, singletonclass_1::*,
        BattleInfo_Flags,
        ICapabilityBase_1Methods, IJobDataMethods, IPersonDataMethods,
        keyhelpdata::*,
    },
    unity_engine::{animator::*, gameobject::*, component::*},
    List_1Ext,
    Mess,
    combat::{Character, Kaneko},
    nn::hid::NpadButton,
    tm_pro::{ITMP_TextMethods, TextMeshProUGUI},
    unity_engine::ITransformMethods,
};
use unity::{Cast, Class, IlNull, OptionalMethod};
use crate::utils::to_opt;

pub fn add_unit_status_key_help_data() {
    [("KHID_クラスチェンジ", 2)].into_iter().for_each(|(khid, button)|{
        if let Some(key) = to_opt(KeyHelpData::get(khid.into())){
            if !key.base().base().iter().any(|v| v.get_button_index() == button) {
                let x_key = KeyHelpData::new();
                x_key.set_button_index(button);
                x_key.set_mid("MID_CONFIG_OVERALLINFORMATION_TOGGLE");
                key.add(x_key);
            }
        }
    });
}
pub fn unit_status_tick_method_edit() {
    if let Some(method) = Class::lookup("App", "UnitStatus").raw_mut().get_method_from_name("Tick", 0) {
        method.method_ptr = unit_status_tick as _;
    }
}
pub fn toggle_parameter_change(setter: UnitStatusSetter, unit: Unit, increase: bool) {
    let v = get_unit_status_mode(setter);
    let new = (if increase { v + 1 } else { v + 2 }) % 3;
    change_parameter_mode(setter, unit, new, false);
}
pub fn change_parameter_mode(setter: UnitStatusSetter, unit: Unit, state: u8, job: bool){
    let param_root = setter.get_transform();
    let unit_index = unit.get_person().index() as i16;
    if !job {
        if (get_unit_status_mode(setter) == state) && (get_unit_status_person_index(setter) == unit_index) { return; }
    }
    set_unit_status_person_index(setter, unit_index);
    set_unit_status_mode(setter, state);
    if let Some(parameter) = to_opt(Kaneko::find_in_children(param_root, "Parameter2")){
        let text = parameter.get_child(0).get_game_object().get_component_in_children_3::<TextMeshProUGUI>();
        if !text.is_null(){
            match state {
                1 => {
                    let mid = if job { "MID_SORTIE_SKILL_CATEGORY_JOB" } else { "MID_SORTIE_SKILL_CATEGORY_PERSON"};
                    text.set_text_2(format!("{} ({})", Mess::get("MID_GAMESTART_GROWMODE_SELECT_TITLE"), Mess::get(mid)), true);
                }
                2 => {
                    text.set_text_2(format!("{} ({})", Mess::get("MID_SYS_Capability"), Mess::get("MID_SYS_MAX")), true);
                }
                _ => { text.set_text_2(Mess::get("MID_SYS_Capability"), true); }
            }
        }
    }
    if let Some(title) = to_opt(Kaneko::find_in_children(param_root, "Parameter3"))
        .and_then(|p|to_opt(Kaneko::find_in_children(p, "Title")))
        .and_then(|t| to_opt(t.get_game_object()))
        .and_then(|go| to_opt(go.get_component::<TextMeshProUGUI>()))
    {
        let mid = if state == 1 { "MID_MENU_ACHIEVEMENT_COUNT_TOTAL" } else { "MID_SYS_Rating" };
        title.set_text_2(Mess::get(mid), true);
    }
    if let Some(hp_slash) = to_opt(Kaneko::find_in_children(param_root, "MaxValue")).map(|t| t.get_child(0)) {
        hp_slash.get_game_object().set_active(state == 0);
    }
    setter.m_now_hp().get_game_object().set_active(state == 0);
    match state {
        1 => {
            let mut total = 0;
            let mut total_other = 0;
            setter.m_capability().iter().for_each(|v| {
                let cap_index = v.m_capability_type().value;
                let grow_t = unit.get_capability_grow(cap_index, false);
                let diff =
                    if job { unit.get_job().get_diff_grow().get(cap_index) as i32 } else {
                        let personal = unit.get_person().get_grow().get(cap_index) as i32;
                        grow_t - personal
                    };
                set_param_value(v.into(), grow_t, diff, false);
                total += grow_t;
                total_other += diff;
            });
            set_param_value(setter.m_rating(), total, total_other, false);
        }
        2 => {
            let mut total = 0;
            let mut total_other = 0;
            let limit = unit.get_person().get_limit();
            let job = unit.get_job();
            setter.m_capability().iter().for_each(|v| {
                let cap_index = v.m_capability_type().value;
                let limit = limit.get(cap_index) as i32;
                let grow_t = job.get_limit().get(cap_index) as i32 + limit;
                let maxed = unit.get_capability(cap_index, false) >= grow_t;
                set_param_value(v.into(), grow_t, limit, maxed);
                total += grow_t;
                total_other += limit;
            });
            set_param_value(setter.m_rating(), total, total_other, false);
        }
        _ => {
            if !job { setter.set_data(unit, BattleInfo_Flags { value: 0 }); }
            setter.m_capability().iter().for_each(|v| { set_plus_minus_status_up(v.m_god_up_balloon(), true); });
            set_plus_minus_status_up(setter.m_rating().m_god_up_balloon(), true);
        }
    }
}
fn set_plus_minus_status_up(go: GameObject, plus: bool) {
    if let Some(plus_text) = to_opt(engage::app::Ut::find_child_component_2::<TextMeshProUGUI>(go, "+")){
        plus_text.set_text_2(if plus { "+" } else { "-" }, true);
    }
}
pub extern "C" fn unit_status_tick(_this: UnitStatus, _: OptionalMethod) {
    let go = UnitStatus::get_game_object();
    if !go.is_null() {
        let animator = go.get_component::<Animator>();
        let mut state = animator.get_current_animator_state_info(0);
        if state.is_name("Closed") {
            let n_time = state.get_normalized_time();
            if n_time.is_nan() || n_time < 1.0 { go.set_active(false); }
        }
        if !animator.get_bool("isOpening") { return; }
        let status = go.get_component::<UnitStatusSetter>();
        let param = UnitInfoParamManager::get_instance();
        if param.is_null() || status.is_null() || param.m_calc_unit().is_null() { return; }
        let unit = param.m_calc_unit();
        if unit.is_null() || unit.get_person().is_null() { return; }
        let current_person = status.m_prev_unit().get_person().index() as i16;
        if current_person != get_unit_status_person_index(status){
            if !status.m_prev_unit().is_null() { change_parameter_mode(status, status.m_prev_unit(), 0, false); }
            else { change_parameter_mode(status, unit, 0, false); }
            set_unit_status_person_index(status, current_person);
        }
        else {
            let left = Pad::is_trigger(NpadButton::zl());
            let right = Pad::is_trigger(NpadButton::zr());
            if left || right {
                if !status.m_prev_unit().is_null() { toggle_parameter_change(status, status.m_prev_unit(), right); }
                else { toggle_parameter_change(status, unit, right); }
                GameSound::post_event("Decide", Character::null());
            }
        }
    }
}
fn set_param_value(value: UnitStatusSetter_ValueParam, v: i32, diff: i32, is_limit: bool) {
    UnitStatusSetter::set_arrow_sprite(value.m_setter(), value.m_up_arrow(), diff);
    UnitStatusSetter::set_text_color(value.m_value(), 0, false);
    value.m_value().set_text_2(v.to_string(), true);
    value.m_god_up_balloon().set_active(diff != 0);
    if diff != 0 {
        if diff < 0 { value.set_balloon(diff.abs(), UnitStatusSetter_RingType::dark_god()); }
        else { value.set_balloon(diff.abs(), UnitStatusSetter_RingType::none()); }
        set_plus_minus_status_up(value.m_god_up_balloon(), diff > 0);
        UnitStatusSetter::set_text_color(value.m_god_up_value(), 0, false);
    }
    UnitStatusSetter::set_text_color(value.m_value(), 0, is_limit);
    value.m_god_buff_image().get_game_object().set_active(false);
}
pub fn set_unit_status_mode(this: UnitStatusSetter, value: u8) { unity::field_set_value_at_offset(this, 0x1ba, value); }
pub fn get_unit_status_mode(this: UnitStatusSetter) -> u8 { unity::field_get_value_at_offset(this, 0x1ba) }
pub fn set_unit_status_person_index(this: UnitStatusSetter, value: i16) { unity::field_set_value_at_offset(this, 0x1bc, value); }
fn get_unit_status_person_index(this: UnitStatusSetter) -> i16 { unity::field_get_value_at_offset(this, 0x1bc) }