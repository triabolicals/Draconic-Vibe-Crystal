use engage::{
    app::{
        mapinspectors::*, mapinspector::*, pokeinspector::*, eachinspector::*,
        unitinspector::*, personinspector::*, interruptinspector::*,
        ScriptUtil, EachInspector, EventScript, 
        ISingletonClass_1Methods, IStructBase, IStructData_1Methods
    },
    List_1Ext,
    moon_sharp::interpreter::{dynvalue::*, script::*,  ITableMethods},
};
use crate::{DVCVariable, DVCVariables, gamedata::PersonPool, utils::il2str};
use unity::{Cast, OptionalMethod, system::string::IIl2CppStringMethods};

fn person_index_convert(person_index: i32) -> Option<i32> {
    if DVCVariable::UnitRecruitment.get_value()  == 0 || person_index < 1 { return None; }
    if let Some(person) = crate::utils::to_opt(engage::app::PersonData::try_get_2(person_index)){
        if let Some(person) = PersonPool::get_replacement_2(person, false){ return Some(person.index()); }
    }
    None
}

pub fn adjust_person_map_inspectors() {
    if DVCVariables::get_chapter_index() == 26 {
        let script = EventScript::get_instance().m_global_table();
        script.set("temp_x_start", DynValue::new_number(1.0));
        script.set("temp_x_end", DynValue::new_number(29.0));
        script.set("temp_z_start", DynValue::new_number(17.0));
        script.set("temp_z_end", DynValue::new_number(23.0));
        return;
    }
    let inspectors = MapInspectors::get_instance();
    let free_deploy = DVCVariable::UnitDeployment.get_value() == 3;
    inspectors.m_inspectors().iter().for_each(|inspector|{ adjust_inspector(inspector, free_deploy); });
}

fn adjust_inspector(inspector: MapInspector, free_deploy: bool) {
    let kind = inspector.get_kind();
    let chapter_index = DVCVariables::get_chapter_index();
    if !inspector.get_function().is_null() {
        let func_name = ScriptUtil::get_func_name(inspector.get_function());
        if let Some(r) = il2str(func_name){
            let hash =  func_name.get_hash_code();
            /*
            println!("Function Name: {} [{}], Kind: {} ({})", r,  hash, inspector.get_class().raw().get_name(), kind.value);
            if kind.value == 1 {
                println!("Turn Min/Max {} / {} Force; {}",
                         unity::field_get_value_at_offset::<i32>(inspector, 0x30),
                         unity::field_get_value_at_offset::<i32>(inspector, 0x34),
                         unity::field_get_value_at_offset::<i32>(inspector, 0x38),
                );
            }
             */
            if chapter_index == 53 && -348564208 == hash {
                let func = EventScript::get_instance().m_global_table().get("EngageAttackCheck");
                if !func.is_null() { inspector.set_condition(func); }
                return;
            }
        }
    }
    if free_deploy {
        if kind == MapInspector_Kind::escape() {
            if let Some(poke) = inspector.try_cast::<PokeInspector>() {
                let person =  poke.m_person();
                if person == 1 { poke.set_m_person(-1); }
                else if let Some(p) = person_index_convert(person){ poke.set_m_person(p); }
            }
            return;
        }
        if kind == MapInspector_Kind::fixed() {
            if let Some(unit) = inspector.try_cast::<UnitInspector>(){
                let person =  unit.m_person();
                if person == 1 { unit.set_m_person(-1); }
                else if let Some(p) = person_index_convert(person){ unit.set_m_person(p); }
            }
            return;
        }
        if kind == MapInspector_Kind::talk() {
            if let Some(talk) = inspector.try_cast::<EachInspector>(){
                if talk.m_from_person() == 1 { talk.set_m_from_person(-1); }
                if let Some(to) = person_index_convert(talk.m_to_person()) { talk.set_m_to_person(to); }
            }
            return;
        }
    }
    let kind = MapInspectorKind::from(kind);
    match kind {
        MapInspectorKind::UnitCommandPrepare | MapInspectorKind::TargetSelect | MapInspectorKind::EngageAfter | MapInspectorKind::EngageBefore|
        MapInspectorKind::Pickup => {
            let inspector = unsafe { inspector.cast::<PersonInspector>() };
            if let Some(person) = person_index_convert(inspector.m_person()) { inspector.set_m_person(person); }
        }
        MapInspectorKind::Die | MapInspectorKind::ReviveBefore | MapInspectorKind::ReviveAfter | MapInspectorKind::Fixed => {
            let inspector = unsafe { inspector.cast::<UnitInspector>() };
            if let Some(person) = person_index_convert(inspector.m_person()) { inspector.set_m_person(person); }
        }
        MapInspectorKind::BattleTalk | MapInspectorKind::BattleAfter | MapInspectorKind::BattleBefore | MapInspectorKind::Talk => {
            let inspector = unsafe { inspector.cast::<EachInspector>() };
            if let Some(person) = person_index_convert(inspector.m_from_person()) { inspector.set_m_from_person(person); }
            if let Some(person) = person_index_convert(inspector.m_to_person()) { inspector.set_m_to_person(person); }
        }
        MapInspectorKind::UnitCommandInterrupt => {
            let inspector = unsafe { inspector.cast::<InterruptInspector>() };
            if let Some(person) = person_index_convert(inspector.m_person()) { inspector.set_m_person(person); }
        }
        MapInspectorKind::Escape | MapInspectorKind::Breakdown => {
            let inspector = unsafe { inspector.cast::<PokeInspector>() };
            if let Some(person) = person_index_convert(inspector.m_person()) { inspector.set_m_person(person); }
        }
        _ => {}
    }

}
pub enum MapInspectorKind {
    None = 0,
    Turn = 1,   //TurnCommon
    TurnAfter = 2,  //TurnCommon
    TurnEnd = 3,    //TurnCommon
    Area = 4,   //Area
    Tbox = 5,   //Poke
    Door = 6,   //Poke
    Torch = 7,  //Poke
    Visit = 8,  //Poke
    Escape = 9, //Poke
    Destroy = 10,   //Poke
    Breakdown = 11, //Poke
    BreakdownEnemy = 12,    //Poke
    Waypoint = 13,  //Poke
    Command = 14,   //Poke
    Die = 15,   //Unit
    ReviveBefore = 16,  //Unit
    ReviveAfter = 17,   //Unit
    Fixed = 18, //Unit
    Talk = 19,  //Each
    BattleBefore = 20,  //Each
    BattleTalk = 21,    //Each
    BattleAfter = 22,   //Each
    Pickup = 23,    //Person
    TargetSelect = 24,  //Person
    UnitCommandPrepare = 25,    //Person
    UnitCommandInterrupt = 26,  //Interrupt
    EngageBefore = 27,  //Person
    EngageAfter = 28,   //Person
    Cannon = 29,    //Cannon
    HelpSpot = 30,  //Poke
}
impl MapInspectorKind {
    pub fn from(i: MapInspector_Kind) -> Self {
        let i = i.value;
        match i {
            1 => MapInspectorKind::Turn,
            2 => MapInspectorKind::TurnAfter,
            3 => MapInspectorKind::TurnEnd,
            4 => MapInspectorKind::Area,
            5 => MapInspectorKind::Tbox,
            6 => MapInspectorKind::Door,
            7 => MapInspectorKind::Torch,
            8 => MapInspectorKind::Visit,
            9 => MapInspectorKind::Escape,
            10 => MapInspectorKind::Destroy,
            11 => MapInspectorKind::Breakdown,
            12 => Self::BreakdownEnemy,
            13 => Self::Waypoint,
            14 => Self::Command,
            15 => Self::Die,
            16 => Self::ReviveBefore,
            17 => Self::ReviveAfter,
            18 => Self::Fixed,
            19 => Self::Talk,
            20 => Self::BattleBefore,
            21 => Self::BattleTalk,
            22 => Self::BattleAfter,
            23 => Self::Pickup,
            24 => Self::TargetSelect,
            25 => Self::UnitCommandPrepare,
            26 => Self::UnitCommandInterrupt,
            27 => Self::EngageBefore,
            28 => Self::EngageAfter,
            29 => Self::Cannon,
            30 => Self::HelpSpot,
            _ => Self::None,
        }
    }
}

pub fn turn_common_inspector_is_eanble(this: MapInspector, turn: i32, current_force: i32, _optional_method: OptionalMethod) -> bool {
    let min = unity::field_get_value_at_offset::<i32>(this, 0x30);
    let max = unity::field_get_value_at_offset::<i32>(this, 0x34);
    let force = unity::field_get_value_at_offset::<i32>(this, 0x38);
    if (min < 0 || min <= turn) && (max < 0 || turn <= max) && this.is_value(force, current_force) {
        let condition = this.get_condition();
        let cond = this.is_condition();
        if cond && !condition.is_null() {
            if condition.m_type().value == 10 { return EventScript::get_instance().call(condition).get_boolean(); }
        }
        cond
    }
    else { false }
}
pub fn unit_inspector_is_eanble(this: MapInspector, current_person: i32, current_force: i32, _optional_method: OptionalMethod) -> bool {
    let person = unity::field_get_value_at_offset::<i32>(this, 0x30);
    let force = unity::field_get_value_at_offset::<i32>(this, 0x34);
    let is_person =  this.is_value(person, current_person);
    let is_force = this.is_value(force, current_force);
    if is_person && is_force {
        let condition = this.get_condition();
        let cond = this.is_condition();
        if cond && !condition.is_null(){
            if condition.m_type().value == 10 { return EventScript::get_instance().call(condition).get_boolean(); }
        }
        cond
    }
    else { false }
}