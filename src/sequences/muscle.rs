use super::*;
use engage::{
    app::{
        push_up::*,
        sit_up::*,
        squat::*,
        capabilitydefinition::*,
    },
    app::{IUnit, IUnitEnhanceFactorsMethods}
};
use engage::app::{Force, Force_Type, IMuscleExercisePrizeDataMethods, IRandom_2Methods, IStructData_1Methods, Random_2};
use outfit_core::Randomizer;

const STAT: [&'static str; 6] = ["HP", "力", "技", "HP", "Str", "Tec"];
const PRIZE_LEVEL: [&'static str; 4] = ["Normal", "Hard", "Master", "Endless"];
const EXCERCISE: [&'static str; 3] = ["PushUp", "SitUp", "Squat"];

pub fn exercise_prize_method_edits() {
    [("PushUp", "Pushup"), ("SitUp", "Situp"), ("Squat", "Squat")].into_iter().for_each(|i|{
        let namespace = format!("App.{}", i.0);
        let class = format!("Muscle{}Sequence", i.1);
        let class_data = Class::lookup(namespace.as_str(), class.as_str());
        if let Some(method) = class_data.raw_mut().get_method_from_name("GetPrizeBonus", 0){
            method.method_ptr = get_stat_boost_muscle as _;
        }
    });
}

extern "C" fn get_stat_boost_muscle(this: ProcInst, _: OptionalMethod){
    let mut rank_bonus = Il2CppString::null();
    if let Some(push) = this.try_cast::<MusclePushupSequence>() {
        if push.get_select_level() > 2 { return; }
        rank_bonus = push.m_rank_bonus();
    }
    else if let Some(s) = this.try_cast::<MuscleSitupSequence>() {
        if s.get_select_level() > 2 { return; }
        rank_bonus = s.m_rank_bonus();
    }
    else if let Some(s) = this.try_cast::<MuscleSquatSequence>() {
        if s.get_select_level() > 2 { return; }
        rank_bonus = s.m_rank_bonus();
    }
    if !rank_bonus.is_null(){
        get_rank_bonus(this, rank_bonus, DVCFlags::ExerciseStat.get_value());
    }
}
fn get_random_stat_order() -> [i32; 6] {
    let mut stats: Vec<i32> = (0..11).into_iter().collect();
    let rng = DVCVariables::get_rng();
    stats.get_remove(rng);
    let mut out = [0, 1, 2, 0, 1, 2];
    for i in 0..6 {
        if let Some(v) = stats.get_remove(rng){ out[i] = v; }
    }
    out
}
fn get_rank_bonus(this: ProcInst, rank_bonus: Il2CppString, random: bool){
    if rank_bonus.is_null() { return; }
    let str = rank_bonus.to_rust_string();
    if let Some((stat, amount)) = str.split_once("+") {
        if let Some((v, i)) = amount.parse::<i32>().ok().zip(STAT.iter().position(|&s| s == stat).filter(|&s| s < 3)){
            let stat_order = get_random_stat_order();
            let stat = if random { stat_order[i] } else { i as i32 };
            let amount = if stat == 9 { 5 } else { v };
            let hero = engage::app::UnitPool::get_hero(false);
            hero.m_enhance_factors().set_hub_value_2(stat, amount );
            if random && Random_2::get_system().get_value_2(4) == 0 && stat < 9 {
                let s_message = format!("{}/{}", CapabilityDefinition::get_name(stat), CapabilityDefinition::get_name(stat_order[3 + i]));
                engage::app::Mess::set_argument_2(0, s_message);
                hero.m_enhance_factors().set_hub_value_2(stat_order[3 + i], amount);
            }
            else {
                engage::app::Mess::set_argument_2(0, CapabilityDefinition::get_name(stat));
                engage::app::Mess::set_argument(1, amount);
            };
            engage::app::GameMessage::create_key_wait(this, engage::app::Mess::get("MID_Hub_MuscleExercises_StatusUp"));
            engage::app::GameSound::post_event("Status_Up", engage::combat::Character::null());
            if stat == 9 {
                Force::get(Force_Type::absent()).iter()
                    .for_each(|u|{ u.m_enhance_factors().set_hub_value_2(stat, amount); });
            }
        }
    }
}
pub extern "C" fn hub_quick_exercise_get_prize_bonus(this: ProcInst, _: OptionalMethod) {
    let kind = unity::field_get_value_at_offset::<i8>(this, 0x1c);
    let level = unity::field_get_value_at_offset::<i8>(this, 0x1d);
    let prize_id = format!("ID_Prize{}_{}", EXCERCISE[kind as usize], PRIZE_LEVEL[level as usize]);
    let prize = engage::app::MuscleExercisePrizeData::get(prize_id.into());
    get_rank_bonus(this, prize.get_bonus_sss(), DVCFlags::ExerciseStat.get_value());
}