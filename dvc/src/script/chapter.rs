use engage::{
    gamedata::Gamedata, map::image::MapImage, mess::Mess, random::Random,
    gamemessage::GameMessage, gameuserdata::GameUserData, gamevariable::GameVariableManager,
    script::*, util::get_instance,
};
use unity::{prelude::*, il2cpp::object::Array};
use crate::{randomizer::Randomizer, config::DVCFlags, DVCVariables, enums::{MPIDS, PIDS, RINGS}};

pub extern "C" fn install_script_edits(script: &EventScript) {
    let chapter = GameUserData::get_chapter().cid.to_string();
    if chapter == "CID_M026" { script.register_action("味方キャラを再配置", m026_phase_2_positions); }
    if chapter == "CID_M022" {
        script.register_action("ユニット会話_ソロ時", ring_talk_1);
        script.register_action("ユニット会話_シンクロ中", ring_talk_2);
        script.register_action("Dialog", ring_dialog_up_dialog);
    }
}
pub extern "C" fn m026_phase_2_positions(_args: &Il2CppArray<&DynValue>, _method_info: OptionalMethod) {
    let map_image = get_instance::<MapImage>();
    let args = Array::<&DynValue>::new_from_element_class(DynValue::class(), 3).unwrap();
    for x in 0..3 {
        args[x] = DynValue::new_number(0.0);
    }
    if DVCFlags::RandomDeploySpot.get_value() {
        println!("Phase 2 Deployment Random");
        let mut possible_pos = vec![];
        for z in 0..31 {
            for x in 0..31 {
                if z >= 13 && z < 18 && x >= 13 && x < 18 { continue; } // Big Dragon Position
                if map_image.unit.get_unit(x, z).is_none() {
                    args[0].assign_number(x as f64);
                    args[1].assign_number(z as f64);
                    let terrain_cost = ScriptMap::terrain_get_move_cost(args);
                    if terrain_cost.get_string().is_some_and(|cost| cost.str_contains("COST_平地")) {
                        possible_pos.push((x, z));
                    }
                }
            }
        }
        let rng = Random::get_game();
        for z in 17..23 {
            for x in 1..29 {
                if let Some(unit) = map_image.unit.get_unit(x, z) {
                    args[0] = DynValue::new_string(unit.person.pid);
                    if let Some(pos) = possible_pos.get_random_element(rng) {
                        args[1].assign_number(pos.0 as f64);
                        args[2].assign_number(pos.1 as f64);
                        ScriptUnit::unit_set_pos(args);
                    }
                }
            }
        }
    }
    else {
        let mut pos_list = [
            ( 11, 12 ), ( 19, 12 ), ( 12, 11 ), ( 18, 11 ), ( 11, 10 ), ( 12, 10 ), ( 14, 10 ), ( 16, 10 ), ( 18, 10 ), ( 19, 10 ),
            ( 13, 9  ), ( 15, 9  ), ( 17, 9  ), ( 14, 8 ), ( 16, 8 ), ( 13, 7), ( 17, 7 ), ( 14, 6 ), ( 16, 6 ), ( 15, 5 ),
            ( 10, 11 ), ( 10, 10 ), ( 9, 11 ), ( 20, 11 ), ( 20, 10 ), ( 21, 11 ), ( 9, 12 ), ( 21, 12 ), ( 11, 9 ),
            ( 19, 9 ), ( 10, 12 ), ( 20, 12 ), ( 13, 10 ), ( 17, 10 ), ( 14, 9 ), ( 16, 9 ), ( 15, 8 ), ( 14, 7 ),
            ( 16, 7 ), ( 15, 6 ),
        ].iter();
        for z in 17..23 {
            for x in 1..29 {
                if let Some((unit, pos)) = map_image.unit.get_unit(x, z).zip(pos_list.next()) {
                    args[0] = DynValue::new_string(unit.person.pid);
                    args[1].assign_number(pos.0 as f64);
                    args[2].assign_number(pos.1 as f64);
                    ScriptUnit::unit_set_pos(args);
                }
            }
        }
    }
}

extern "C" fn ring_talk_2(args: &Il2CppArray<&DynValue>, _method_info: OptionalMethod) {
    if let Some(pid) = args.try_get_string(0) {
        let args = Array::<&DynValue>::new_from_element_class(DynValue::class(), 1).unwrap();
        args[0] = DynValue::new_string(pid);
        let pid_str = pid.to_string();
        GameVariableManager::set_number("TalkPID", 0);
        if let Some(pos) = PIDS.iter().position(|x| *x == pid_str) {
            let talk_key = format!("{}2", MPIDS[pos].replace("MPID", "MID_TK"));
            args[0] = DynValue::new_string(talk_key.into());
        }
        else if let Some(p) = engage::gamedata::PersonData::get(pid_str.as_str()) {
            GameVariableManager::set_number("TalkPID", p.parent.hash);
            if p.flag.value & 128 != 0 { args[0] = DynValue::new_string("MID_TK_Lueur2".into()); }
            else if let Some(name) = p.name.as_ref().
                and_then(|name| PIDS.iter().map(|mpid| mpid.trim_start_matches("MPID_").to_string())
                    .find(|mpid| name.str_contains(mpid)))
            {
                args[0] = DynValue::new_string(format!("MID_TK_{}2", name).into());
            }
            else {
                let sel = Random::get_game().get_value(40) + 1;
                let talk_key = format!("{}2", MPIDS[sel as usize].replace("MPID", "MID_TK"));
                args[0] = DynValue::new_string(talk_key.into());
            }
        }
        ScriptSystem::talk(args);
    }
    GameVariableManager::set_number("TalkPID", 0);
}
extern "C" fn ring_talk_1(args: &Il2CppArray<&DynValue>, _method_info: OptionalMethod) {
    if let Some(pid) = args.try_get_string(0) {
        let args = Array::<&DynValue>::new_from_element_class(DynValue::class(), 1).unwrap();
        args[0] = DynValue::new_string(pid);
        GameVariableManager::set_number("TalkPID", 0);
        let pid_str = pid.to_string();
        if !PIDS.iter().any(|x| *x == pid_str){
            if let Some(p) = engage::gamedata::PersonData::get(pid_str.as_str()).filter(|p| p.flag.value & 128 == 0){
                GameVariableManager::set_number("TalkPID", p.parent.hash);
                if let Some(name) = p.name.as_ref()
                    .and_then(|name| PIDS.iter().map(|mpid| mpid.trim_start_matches("MPID_").to_string())
                        .find(|mpid| name.str_contains(mpid)))
                {
                    args[0] = DynValue::new_string(format!("MID_TK_{}1", name).into());
                    ScriptSystem::talk(args);
                    return;
                }
            }
            let sel = Random::get_game().get_value(40) + 1;
            let talk_key = format!("{}2", MPIDS[sel as usize].replace("MPID", "MID_TK"));
            args[0] = DynValue::new_string(talk_key.into());
        }
        ScriptSystem::talk(args);
    }
    GameVariableManager::set_number("TalkPID", 0);
}
extern "C" fn ring_dialog_up_dialog(args: &Il2CppArray<&DynValue>, _method_info: OptionalMethod) {
    let v = DVCFlags::GodNames.get_value();
    if DVCVariables::is_changed_recruitment_order(true) || v {
        if let Some(mid) = args.try_get_string(0)
            .filter(|v| v.str_contains("MID_TUT_NAVI_M022_GET_")).map(|v| v.to_string())
        {
            println!("Message: {}", mid.as_str());
            let mut message = Mess::get("MID_TUT_NAVI_M022_GET_Siglud").to_string();
            DVCFlags::GodNames.set_value(false);
            let sigurd = Mess::get("MGID_Ring_Siglud").to_string();
            DVCFlags::GodNames.set_value(v);
            let mid2 = mid.trim_start_matches("MID_TUT_NAVI_M022_GET_");
            if let Some(god) = RINGS.iter().position(|x| *x == mid2).and_then(|n| DVCVariables::get_god_from_index(n as i32, false)) {
                if let Some(person) = crate::randomizer::names::get_emblem_person(god.mid).and_then(|m| m.name.as_ref())
                {
                    if let Some(alias) = MPIDS.iter().find(|mid| person.str_contains(*mid))
                        .map(|mid| Mess::get(mid.replace("MPID_", "MPID_alias_")))
                    {
                        message = message.replace(sigurd.as_str(), alias.to_string().as_str());
                    }
                    else {
                        let name = Mess::get(*person);
                        message = message.replace(sigurd.as_str(), name.to_string().as_str());
                    }
                }
                else if let Some(ring) = god.ring_name.map(|r| Mess::get(r)){
                    message = message.replace(sigurd.as_str(), ring.to_string().as_str());
                }
                else {
                    let god_name = god.mid.to_string();
                    message = message.replace(sigurd.as_str(), god_name.as_str());
                }
                let sequence = ScriptUtil::get_sequence();
                GameMessage::create_key_wait(sequence, message.as_str());
                return;
            }
        }
    }
    ScriptSystem::dialog(args)
}