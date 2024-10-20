use super::*;

#[unity::class("App", "MapInspectors")]
pub struct MapInspectors {
    parent: [u8; 0x10],
    pub inspectors: &'static mut List<MapInspector>,
    pub kind_inspectors: &'static mut Array<&'static mut List<MapInspector>>,
}

#[unity::class("App", "MapInspector")]
pub struct MapInspector {
    pub kind: i32,
    __: i32,
    pub  m_condition: &'static DynValue,
    pub function: &'static DynValue,
    pub arg: &'static Array<&'static DynValue>,
    pub var1: i32,
    pub var2: i32,
    pub var3: i32,
    pub var4: i32,
    pub var5: i32,
    pub var6: i32,
}
impl MapInspectors {
    fn get_instance() -> &'static mut MapInspectors {
        let idk = get_generic_class!(SingletonClass<MapInspectors>).unwrap();
        let pointer = unsafe { &*(idk.rgctx_data as *const Il2CppRGCTXData as *const u8 as *const [&'static MethodInfo; 6]) };
        let get_instance =
            unsafe { std::mem::transmute::<_, extern "C" fn(OptionalMethod) -> &'static mut MapInspectors>(pointer[5].method_ptr) };
        get_instance(Some(&pointer[5]))
    }
}

pub fn change_g_pid_lueur() {
    if !GameVariableManager::exist("G_R_PID_リュール") { return; }
    let replacement_pid = GameVariableManager::get_string("G_R_PID_リュール");
    if unsafe { crate::utils::is_null_empty(replacement_pid, None) } { return; }
    println!("Replacing Lueur with replacement"); 
    if crate::utils::str_contains(replacement_pid, "PID_") {
        if replacement_pid.get_string().unwrap() != "PID_リュール" {
            let dyn_value = unsafe { crate::message::dynvalue_new_string(replacement_pid, None) };
            crate::message::set_script_variable("g_pid_lueur", dyn_value);
            println!("Lueur PID was replaced for Chapter 22"); 
        }
    }
}

pub fn adjust_map_inspectors() {
    println!("Adjust Map Inspectors");
    if GameUserData::get_chapter().cid.get_string().unwrap() == "CID_M022" {
        GameVariableManager::make_entry("VeyleRecruitment", 0);

        // Change PID
        if GameVariableManager::get_number("G_Random_Recruitment") != 0 || crate::utils::lueur_on_map() { change_g_pid_lueur(); }

    }
    if GameVariableManager::get_number("G_EmblemDeployMode") == 2  {
        emblem_selection_menu_enable(false);
        unsafe { remove_all_rings(0, None); }
    }
    else if GameVariableManager::get_number("G_EmblemDeployMode") == 1 {
        unsafe { remove_all_rings(0, None); }
        let emblem_list = get_emblem_list();
        if emblem_list.len() == 0 { return; }
        let mut emblem_count = emblem_list.len();
        let mut set_emblems: [bool; 20] = [false; 20];
        let player_force = Force::get(ForceType::Player).unwrap();
        let max_player = player_force.get_count();
        if emblem_count > max_player as usize { emblem_count = max_player as usize; }
        let mut current_emblem_count = 0;
        let mut force_iter = Force::iter(player_force);
        let rng = Random::get_game();
        while let Some(unit) = force_iter.next() {
            let mut value = rng.get_value(emblem_list.len() as i32) as usize;
            while set_emblems[value] == true { value = rng.get_value(emblem_list.len() as i32) as usize;  }
            let god_unit = GodPool::try_get_gid(emblem_list[value], true).unwrap();
            unit.set_god_unit(god_unit);
            current_emblem_count += 1;
            set_emblems[value] = true;
            if current_emblem_count == emblem_count { break; } 
        }
    }
    else { Patch::in_text(0x01d77028).bytes(&[0xc0, 0x00, 0x00, 0x36]).unwrap();}

    if crate::utils::lueur_on_map() && GameVariableManager::get_number("G_DeploymentMode") == 3 { return; } // if alear is on map don't change anything 
    let inspectors = MapInspectors::get_instance();
    for x in 0..inspectors.inspectors.len() {
        let kind = inspectors.inspectors[x].kind;
        if kind == 9 {
            if inspectors.inspectors[x].var6 == 1 { inspectors.inspectors[x].var6 = -1; }
        }
        if kind == 18 || kind == 19 {
            if inspectors.inspectors[x].var1 == 1 { inspectors.inspectors[x].var1 = -1; } //new_person_index; }
        }
    }
    for x in 0..inspectors.kind_inspectors[9].len() {
        if inspectors.kind_inspectors[9].items[x as usize].var6 == 1 { inspectors.kind_inspectors[9].items[x as usize].var6 = -1; }
    }
    for x in 0..inspectors.kind_inspectors[18].len() {
        if inspectors.kind_inspectors[18].items[x as usize].var1 == 1 { inspectors.kind_inspectors[18].items[x as usize].var1 = -1; }
    }
    for x in 0..inspectors.kind_inspectors[19].len() {
        if inspectors.kind_inspectors[19].items[x as usize].var1 == 1 { inspectors.kind_inspectors[19].items[x as usize].var1 = -1; }
    }
}