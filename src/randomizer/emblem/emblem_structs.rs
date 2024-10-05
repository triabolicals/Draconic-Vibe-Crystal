use super::*;
use engage::{
    mess::Mess,
    random::Random,
};
use std::fs::File;
use std::io::Write;

pub struct EngageAttackIndex {
    pub index_1: i32,
    pub index_2: i32,
    pub in_use: bool,
    pub linked_use: bool,
}
impl EngageAttackIndex {
    pub fn new(value_1: i32, value_2: i32) -> Self { Self { index_1: value_1, index_2: value_2, in_use:false, linked_use: false,}}

}
#[derive(Clone, Copy)]
pub struct SynchoSkill {
    pub index: i32,
    pub max_priority: i32,
    pub min_priority: i32,
    pub randomized_index: i32,
    pub in_use : bool,
    pub eirika_twin_skill: bool,
    pub skill_used: bool,
}
impl SynchoSkill {
    pub fn new(skill_index: i32, priority: i32, eirika: bool) -> Self {
        Self { index: skill_index, max_priority: priority, min_priority: priority, in_use: false, randomized_index: 0, eirika_twin_skill: eirika, skill_used: false} 
    }
    pub fn reset(&mut self) {
        self.in_use = false;
        self.randomized_index = 0;
        self.skill_used = false;
    }
}

pub struct SynchoList {
    pub sync_list: Vec<SynchoSkill>,
    pub inherit_list: Vec<SynchoSkill>,
    pub chaos_list: Vec<SynchoSkill>,
    pub non_random_skills: Vec<i32>,
    pub randomized: bool, 
    pub sync_list_size: i32, // Size before added
}
impl SynchoList {
    // For the three houses gambits to force them to be 4 separate skills instead of one 4-level skill
    pub fn add_to_non_upgrade(&mut self, sid: &str, is_not_randomized: bool){
        let skill = SkillData::get(sid);
        if skill.is_none() { return; } 
        let sk = skill.unwrap();
        let skill_index = sk.parent.index;
        let sid = sk.sid.get_string().unwrap();
        if sk.get_flag() & 1 == 1 && sid != "SID_無し" {  return; } 
        let found = self.sync_list.iter_mut().find(|x| x.index == skill_index);
        if found.is_none() {  self.sync_list.push(SynchoSkill::new(skill_index, 0, false)); }
        if self.non_random_skills.iter().find(|&&x| x == skill_index).is_none() && is_not_randomized {
            self.non_random_skills.push(skill_index ); // Skills not do not get randomized to another skill but other skills can randomized to
        }
    }
    pub fn add_by_sid(&mut self, sid: &str, is_syncho: bool, is_not_randomized: bool){
        let skill = SkillData::get(sid);
        if skill.is_some() { 
            let sk = skill.unwrap();
            let sid = sk.sid.get_string().unwrap();
            if sk.get_flag() & 1 == 1 && sid != "SID_無し" {  return; }  
            let index = sk.parent.index;
            self.add_list(skill.unwrap(), is_syncho);
            if self.non_random_skills.iter().find(|&&x| x == index).is_none() && is_not_randomized {
                self.non_random_skills.push(index); // Skills not do not get randomized to another skill but other skills can randomized to
            }
         }
    }
    pub fn add_by_index(&mut self, skill_index: i32, is_syncho: bool, is_not_randomized: bool){
        let skill = SkillData::try_index_get(skill_index);
        if skill.is_some() { 
            let sk = skill.unwrap();
            let sid = sk.sid.get_string().unwrap();
            if sk.get_flag() & 1 == 1 && sid != "SID_無し" {  return; } 
            self.add_list(sk, is_syncho);
            
            if self.non_random_skills.iter().find(|&&x| x == skill_index).is_none() && is_not_randomized {
                self.non_random_skills.push(skill_index); // Skills not do not get randomized to another skill but other skills can randomized to
            }
         }
    }
    pub fn add_inherit(&mut self, skill_index: i32, priority: i32, in_sync: bool, in_inherit: bool, chaos_only: bool, is_eirika: bool) {
        let found = self.chaos_list.iter_mut().find(|x| x.index == skill_index);
        if found.is_none() {  self.chaos_list.push(SynchoSkill::new(skill_index, priority, is_eirika)); }
        else {
            let skill = found.unwrap();
            if skill.max_priority < priority { skill.max_priority = priority; }
            if priority < skill.min_priority { skill.min_priority = priority; }
        }
        if chaos_only { return; }
        if in_sync {
            let found = self.sync_list.iter_mut().find(|x| x.index == skill_index);
            if found.is_none() {  self.sync_list.push(SynchoSkill::new(skill_index, priority, is_eirika)); }
            else {
                let skill = found.unwrap();
                if skill.max_priority < priority { skill.max_priority = priority; }
                if priority < skill.min_priority { skill.min_priority = priority; }
            }
        }
        if in_inherit {
            let found = self.inherit_list.iter_mut().find(|x| x.index == skill_index);
            if found.is_none() {  self.inherit_list.push(SynchoSkill::new(skill_index, priority, is_eirika));  }
            else {
                let skill = found.unwrap();
                if skill.max_priority < priority { skill.max_priority = priority; }
                if priority < skill.min_priority { skill.min_priority = priority; }
            }
        }
    }
    pub fn add_list(&mut self, skill: &SkillData, is_syncho: bool) {
        // ignore "None" "Night and Day", "Friendly Riviary"
        let can_inherit = skill.get_inheritance_cost() != 0;
        let only_chaos = !can_inherit && !is_syncho;
        //if !can_inherit && !is_syncho { return; }   
        let sid = skill.sid.get_string().unwrap();
        if skill.get_flag() & 1 == 1 && sid != "SID_無し" {  return; }    // cannot be hidden
        if sid == "SID_オルタネイト" || sid == "SID_切磋琢磨"  { return; }
        if sid == "SID_異界の力" { // if book of worlds
            self.add_inherit(skill.parent.index, 0, is_syncho, can_inherit, only_chaos, false);
            return;
        }
        for x in 0..EIRIKA_TWIN_SKILLS.len() {  //Eirika Skills
            if EIRIKA_TWIN_SKILLS[x] == sid {
                let skill_index = if x < 6 { skill.parent.index } else { skill.parent.index - 3 };
                let priority = if x < 6 { 1 } else { 2 };
                self.add_inherit(skill_index, priority, is_syncho, can_inherit, only_chaos, true);
                return;
            }
        }
        let priority = skill.get_priority();
        let skill_index = if priority >= 1 { skill.parent.index - (priority - 1) } else { skill.parent.index };
        self.add_inherit(skill_index, priority, is_syncho, can_inherit, only_chaos, false);
    }
    pub fn reset(&mut self) {
        self.sync_list.iter_mut().for_each(|x| x.reset());
        self.inherit_list.iter_mut().for_each(|x| x.reset());
        self.chaos_list.iter_mut().for_each(|x| x.reset());
        self.sync_list[0].in_use = true;
        self.sync_list[1].in_use = true;
        self.randomized = false;
        if self.sync_list_size as usize > self.sync_list.len() { self.sync_list.drain(self.sync_list_size as usize..); }
    }
    pub fn randomized(&mut self, rng: &Random) {
        if self.randomized { return; }
        let i_size = self.inherit_list.len() as i32;
        let i_list = &mut self.inherit_list;
        let mut value;
        for x in 0..i_size {     //inherit skill -> inherit skill
            value = rng.get_value( i_size );
            let dp = i_list[x as usize].max_priority - i_list[x as usize].min_priority;
            let mut dp2 = i_list[value as usize].max_priority - i_list[value as usize].min_priority;
            let mut count = 0;
            while i_list[ value as usize ].in_use || ( dp != dp2 ) {
                if count == 200 { break; }
                value = rng.get_value( i_size );
                dp2 = i_list[value as usize].max_priority - i_list[value as usize].min_priority;
                count += 1;
            }
            if count == 200 { value = x as i32; }
            i_list[value as usize].in_use = true;
            i_list[x as usize ].randomized_index = value;
        }
        let i_list = &mut self.inherit_list;
        let s_list = &mut self.sync_list;
        let size = s_list.len() as i32;
        for x in 2..size {
            s_list[x as usize].randomized_index = -1;
            let skill_index = s_list[x as usize].index;
            let found = i_list.iter().find(|z| z.index == skill_index);     // is inherit skill in sync list
            if found.is_some() {
                let random_index = i_list[ found.unwrap().randomized_index as usize ].index;
                let found2 = s_list.iter().position(|z| z.index == random_index);    // is the randomized skill in sync list in inherit list
                if found2.is_some() {
                    s_list[found2.unwrap()].in_use = true;
                    s_list[x as usize ].randomized_index = found2.unwrap() as i32;
                }
            }
        }
        value =  rng.get_value( size - 5) + 2;  //Gambit
        while s_list[ value as usize].max_priority != 0 || s_list[ value as usize].in_use { value =  rng.get_value( size - 5) + 2; }
        s_list[0].randomized_index = value;  s_list[ value as usize].in_use = true; s_list[0].skill_used = true;
        //None
        while s_list[ value as usize].max_priority != 0 || s_list[ value as usize].in_use { value =  rng.get_value( size - 5) + 2; }
        s_list[1].randomized_index = value; s_list[ value as usize].in_use = true; s_list[1].skill_used = true;
        for x in 2..size {
            if s_list[x as usize].randomized_index != -1 { continue; }
            if self.non_random_skills.iter().find(|&&y| y == s_list[x as usize].index ).is_some() { 
                s_list[x as usize].randomized_index = x as i32;
                continue; 
            }
            let dp = s_list[x as usize].max_priority;
            let mut count = 0;
            value = rng.get_value( size - 2 ) + 2;
            let mut dp2 = s_list[value as usize].max_priority;
           
            while s_list[ value as usize ].in_use || ( dp != dp2 ) || value < 2 {
                if count >= 100 && !s_list[ value as usize ].in_use { break; }
                if count == 200 { break; }
                value = rng.get_value( size - 2 ) + 2;
                dp2 = s_list[value as usize].max_priority ;
                count += 1;
            }
            if count >= 200 { value = x as i32; }
            s_list[value as usize].in_use = true;
            s_list[x as usize ].randomized_index = value;
        }
        for x in 0..size { 
            if s_list[x as usize ].randomized_index == -1 { s_list[x as usize ].randomized_index = x; }
        }
        let c_size = self.chaos_list.len() as i32;
        let c_list = &mut self.chaos_list;
        // Chaos
        for x in 0..c_size {
            let dp = c_list[x as usize].max_priority;
            let mut count = 0;
            value = rng.get_value( c_size );
            let mut dp2 = c_list[value as usize].max_priority;
            while c_list[ value as usize ].in_use || ( dp != dp2 ) {
                if count >= 100 && !c_list[ value as usize ].in_use { break; }
                if count == 200 { break; }
                value = rng.get_value( c_size );
                dp2 = c_list[value as usize].max_priority ;
                count += 1;
            }
            if count >= 200 { value = x as i32; }
            c_list[value as usize].in_use = true;
            c_list[x as usize ].randomized_index = value;
        }
        self.randomized = true;
    }
    pub fn get_replacement_sid(&mut self, sid: &Il2CppString, is_inherit: bool) -> &'static SkillData {
        let skill = SkillData::get(&sid.get_string().unwrap());
        if skill.is_some() {
            let sk = skill.unwrap();
            let replacement = self.get_replacement(sk, is_inherit);
            return replacement;
        }
        return SkillData::get("SID_無し").unwrap();
    }
    pub fn get_replacement(&mut self, original_skill: &SkillData, is_inherit: bool) -> &'static SkillData {
        let skill_list = SkillData::get_list().unwrap();
        let o_skill = &skill_list[ original_skill.parent.index as usize];
        let sid = original_skill.sid.get_string().unwrap();    
        if original_skill.get_flag() & 1 != 0 && sid != "SID_無し" {  return o_skill; }     // ignore "Night and Day", "Friendly Riviary"
        if sid == "SID_オルタネイト" || sid == "SID_切磋琢磨" { return o_skill; }
        let mut priority = original_skill.get_priority();
        let skill_index;
        if sid == "SID_異界の力" { priority = 0; }
        let is_eirika = EIRIKA_TWIN_SKILLS.iter().position(|x| *x == sid);
        if is_eirika.is_some() {
            let x = is_eirika.unwrap();
            skill_index = if x < 6 { original_skill.parent.index } else { original_skill.parent.index - 3 };
            priority = if x < 6 { 1 } else { 2 };
        }
        else { skill_index = if priority == 0 { original_skill.parent.index } else { original_skill.parent.index - (priority - 1)};  }

        let chaos_mode = GameVariableManager::get_number("G_ChaosMode");
        let mode = if chaos_mode == 3 || ( !is_inherit && chaos_mode == 1 ) { 2 }
                    else if is_inherit { 0 } else { 1 };
        let list = [&mut self.inherit_list, &mut self.sync_list, &mut self.chaos_list];
        let found = list[mode as usize].iter_mut().find(|x| x.index == skill_index);
        if found.is_none() { return o_skill; }
        let f = found.unwrap().clone();
        let new_skill_index = list[mode as usize][ f.randomized_index as usize ].index;
        let new_max_priority = list[mode as usize][ f.randomized_index as usize ].max_priority;
        let is_eirika_twin = list[mode as usize][ f.randomized_index as usize ].eirika_twin_skill;
        let new_priority = priority - f.min_priority + list[mode as usize][ f.randomized_index as usize ].min_priority;

        if mode >= 1 { list[mode as usize][f.randomized_index as usize ].skill_used = true; }

        if is_eirika_twin { // Replacement skill is an Eirika Twin Skill (Lunar Brace/Solar Brace/Eclipse Brace etc...)
            if new_max_priority <= priority { // new_max_priority is 2 for Lunar/Solar/Eclipse Brace +
                return &skill_list[ ( new_skill_index + 3 ) as usize]; 
            }
            return &skill_list[ new_skill_index as usize];
        }
        if new_max_priority == 0 || priority == 0 {  return &skill_list[new_skill_index as usize]; }
        if new_max_priority <= new_priority { return &skill_list[ (new_skill_index + new_max_priority - 1) as usize ]; }
        return &skill_list[ (new_skill_index + new_priority - 1  ) as usize];
    }
    pub fn get_non_randomized_skill(&self) -> Vec<SynchoSkill> {
        let chaos_mode = GameVariableManager::get_number("G_ChaosMode");
        let list = if chaos_mode == 3 || chaos_mode == 1 { &self.chaos_list } else { &self.sync_list };
        let out: Vec<SynchoSkill> = list.iter().filter(|&x| !x.skill_used ).copied().collect();
        out
    }
    pub fn print_inherit_list(&self) {
        let skill_list = SkillData::get_list().unwrap();
        let filename = format!("sd:/Draconic Vibe Crystal/Emblem Skills List.txt");
        let file = File::options().create(true).write(true).truncate(true).open(filename);
        if file.is_err() { println!("Cannot create output file"); return; }
        let mut f = file.unwrap();
        let i_list = &self.inherit_list;
        writeln!(&mut f, "Inherit Skills").unwrap();
        writeln!(&mut f, "Index\tSid\tName\tTo Sid\tTo Name\tSkill Index\tRandom Skill Index\tSkill Max Priority\tRandom Skill Max Priority").unwrap();
        for x in 0..i_list.len() {
            let sid = skill_list[ i_list[x].index as usize].sid.get_string().unwrap();
            writeln!(&mut f, "{}\t{}", i_list[x].index, sid).unwrap();
        }
        for x in 0..i_list.len() {
            let sid = skill_list[ i_list[x].index as usize].sid.get_string().unwrap();
            let name = Mess::get(skill_list[ i_list[x].index as usize].name.expect(format!("Skill #{}: {} does not have a valid MSID", i_list[x].index, sid).as_str())).get_string().unwrap();

            let random_index =  i_list[x].randomized_index as usize;
            let sid2 = skill_list[ i_list[random_index].index as usize].sid.get_string().unwrap();
            let name2 = Mess::get(skill_list[ i_list[random_index].index as usize].name.expect(format!("Skill #{}: {} does not have a valid MSID", i_list[random_index].index, sid2).as_str())).get_string().unwrap();

            writeln!(&mut f, "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}", x, sid, name, sid2, name2, i_list[x].index,  random_index, i_list[x].max_priority, i_list[random_index].max_priority).unwrap();
        }
        let s_list = &self.sync_list;
        writeln!(&mut f, "\nSync Skills").unwrap();
        writeln!(&mut f, "Index\tSid\tName\tTo Sid\tTo Name\tSkill Index\tRandom Skill Index\tSkill Max Priority\tRandom Skill Max Priority\tUsed").unwrap();
        for x in 0..s_list.len() {
            let sid = skill_list[ s_list[x].index as usize].sid.get_string().unwrap();
            let name = Mess::get(skill_list[ s_list[x].index as usize].name.expect(format!("Skill #{}: {} does not have a valid MSID", s_list[x].index, sid).as_str())).get_string().unwrap();
            if s_list[x].randomized_index == -1 { continue; }
            let random_index =  s_list[x].randomized_index as usize;
            let sid2 = skill_list[ s_list[random_index].index as usize].sid.get_string().unwrap();
            let name2 = Mess::get(skill_list[ s_list[random_index].index as usize].name.expect(format!("Skill #{}: {} does not have a valid MSID", s_list[random_index].index, sid2).as_str())).get_string().unwrap();

            writeln!(&mut f, "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}", x, sid, name, sid2, name2, s_list[x].index, random_index, s_list[x].max_priority, s_list[random_index].max_priority, s_list[random_index].skill_used).unwrap();
        }
        let c_list = &self.chaos_list;
        writeln!(&mut f, "\nChaos Skills").unwrap();
        writeln!(&mut f, "Index\tSid\tName\tTo Sid\tTo Name\tSkill Index\tRandom Skill Index\tSkill Max Priority\tRandom Skill Max Priority").unwrap();
        for x in 0..c_list.len() {
            let sid = skill_list[ c_list[x].index as usize].sid.get_string().unwrap();
            let name = Mess::get(skill_list[ c_list[x].index as usize].name.expect(format!("Skill #{}: {} does not have a valid MSID", c_list[x].index, sid).as_str())).get_string().unwrap();
            if c_list[x].randomized_index == -1 { continue; }
            let random_index =  c_list[x].randomized_index as usize;
            let sid2 = skill_list[ c_list[random_index].index as usize].sid.get_string().unwrap();
            let name2 = Mess::get(skill_list[ c_list[random_index].index as usize].name.expect(format!("Skill #{}: {} does not have a valid MSID", c_list[random_index].index, sid2).as_str())).get_string().unwrap();

            writeln!(&mut f, "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}", x, sid, name, sid2, name2, c_list[x].index, random_index, c_list[x].max_priority, c_list[random_index].max_priority).unwrap();
        }
    }
    pub fn get_sync_list_size(&mut self) { 
        println!("Syncho Skill List Size: {}", self.sync_list.len());
        self.sync_list_size = self.sync_list.len() as i32; 
    }
}