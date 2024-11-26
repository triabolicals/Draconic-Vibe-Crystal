use super::*;
use engage::{
    mess::Mess,
    random::Random,
};
use std::fs::File;
use std::io::Write;
use super::super::skill::SKILL_POOL;

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
    pub max_priority: i8,
    pub min_priority: i8,
    pub in_use : bool,
    pub eirika_twin_skill: bool,
    pub skill_used: bool,
    pub randomized_index: i32,
}
impl SynchoSkill {
    pub fn new(skill_index: i32, priority: i8, eirika: bool) -> Self {
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
    pub inherit_cost: Vec<(i32, u16)>,
    pub chaos_inherit_list: Vec<(i32, i32, bool)>,
    pub randomized: bool, 
    pub sync_list_size: i32, // Size before added
}

impl SynchoList {
    // For the three houses gambits to force them to be 4 separate skills instead of one 4-level skill
    pub fn add_to_non_upgrade(&mut self, sid: &str, is_not_randomized: bool){
        if let Some(skill) = SkillData::get(sid) {
            let skill_index = skill.parent.index;
            let sid = skill.sid.to_string();
            if skill.get_flag() & 1 == 1 && sid != "SID_無し" {  return; } 
            if self.sync_list.iter_mut().find(|x| x.index == skill_index).is_none() {
                self.sync_list.push(SynchoSkill::new(skill_index, 0, false));
            }
            if self.non_random_skills.iter().find(|&&x| x == skill_index).is_none() && is_not_randomized {
                self.non_random_skills.push(skill_index ); // Skills not do not get randomized to another skill but other skills can randomized to
            }
        }
    }
    pub fn add_by_sid(&mut self, sid: &str, is_syncho: bool, is_not_randomized: bool){
        if let Some(skill) = SkillData::get(sid) { 
            let sid = skill.sid.to_string();
            if skill.get_flag() & 1 == 1 && sid != "SID_無し" {  return; }  
            let index = skill.parent.index;
            self.add_list(skill, is_syncho);
            if self.non_random_skills.iter().find(|&&x| x == index).is_none() && is_not_randomized {
                self.non_random_skills.push(index); // Skills not do not get randomized to another skill but other skills can randomized to
            }
         }
    }
    pub fn add_by_index(&mut self, skill_index: i32, is_syncho: bool, is_not_randomized: bool){
        if let Some(skill) = SkillData::try_index_get(skill_index)  { 
            let sid = skill.sid.to_string();
            if skill.get_flag() & 1 == 1 && sid != "SID_無し" {  return; } 
            self.add_list(skill, is_syncho);
            if !self.non_random_skills.iter().find(|&&x| x == skill_index).is_none() && is_not_randomized {
                self.non_random_skills.push(skill_index); // Skills not do not get randomized to another skill but other skills can randomized to
            }
         }
    }
    pub fn add_inherit(&mut self, skill_index: i32, priority: i8, in_sync: bool, in_inherit: bool, chaos_only: bool, is_eirika: bool) {
        if let Some(found) = self.chaos_list.iter_mut().find(|x| x.index == skill_index) {
            if found.max_priority < priority { found.max_priority = priority; }
            if priority < found.min_priority { found.min_priority = priority; }
        }
        else { self.chaos_list.push(SynchoSkill::new(skill_index, priority, is_eirika))  }
        if chaos_only { return; }
        if in_sync {
            if let Some(found) = self.sync_list.iter_mut().find(|x| x.index == skill_index) {
                if found.max_priority < priority { found.max_priority = priority; }
                if priority < found.min_priority { found.min_priority = priority; }
            }
            else {  self.sync_list.push(SynchoSkill::new(skill_index, priority, is_eirika)); }
        }
        if in_inherit {
            if let Some(found) = self.inherit_list.iter_mut().find(|x| x.index == skill_index) {
                if found.max_priority < priority { found.max_priority = priority; }
                if priority < found.min_priority { found.min_priority = priority; }
            }
            else { self.inherit_list.push(SynchoSkill::new(skill_index, priority, is_eirika)); }
        }
    }
    pub fn add_list(&mut self, skill: &SkillData, is_syncho: bool) {
        // ignore "None" "Night and Day", "Friendly Riviary"
        let can_inherit = skill.get_inheritance_cost() != 0;
        if can_inherit {
            if !self.inherit_cost.iter().any(|c| c.0 == skill.parent.index) {
                self.inherit_cost.push( (skill.parent.index, skill.get_inheritance_cost()));
            }
            if !self.chaos_inherit_list.iter().any(|c| c.0 == skill.parent.index) {
                self.chaos_inherit_list.push((skill.parent.index, -1, false));
            }
        }
        let only_chaos = !can_inherit && !is_syncho;
        //if !can_inherit && !is_syncho { return; }   
        let sid = skill.sid.to_string();
        if skill.get_flag() & 1 == 1 && sid != "SID_無し" {  return; }    // cannot be hidden
        if sid == "SID_オルタネイト" || sid == "SID_切磋琢磨"  { return; }
        if sid == "SID_異界の力" { // if book of worlds
            self.add_inherit(skill.parent.index, 0, is_syncho, can_inherit, only_chaos, false);
            return;
        }
        if let Some(s_index) = EIRIKA_TWIN_SKILLS.iter().position(|&skill| skill == sid) {
            let skill_index = if s_index < 6 { skill.parent.index } else { skill.parent.index - 3 };
            let priority = if s_index < 6 { 1 } else { 2 } as i8;
            self.add_inherit(skill_index, priority, is_syncho, can_inherit, only_chaos, true);
            return;
        }
        let priority = skill.get_priority() as i8;
        let skill_index = if priority >= 1 { skill.parent.index - ( (priority as i32) - 1) } else { skill.parent.index };
        self.add_inherit(skill_index, priority, is_syncho, can_inherit, only_chaos, false);
    }
    pub fn reset(&mut self) {
        self.sync_list.iter_mut().for_each(|x| x.reset());
        self.inherit_list.iter_mut().for_each(|x| x.reset());
        self.chaos_list.iter_mut().for_each(|x| x.reset());

        self.sync_list[0].in_use = true;
        self.sync_list[1].in_use = true;
        self.randomized = false;
        self.reset_skill_cost();
        if self.sync_list_size as usize > self.sync_list.len() { self.sync_list.drain(self.sync_list_size as usize..); }
        self.chaos_inherit_list.iter_mut().for_each(|x| x.1 = -1);  
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
            if let Some(found) = i_list.iter().find(|z| z.index == skill_index){     // is inherit skill in sync list
                let random_index = i_list[ found.randomized_index as usize ].index;
                if let Some(found2) = s_list.iter().position(|z| z.index == random_index) {   // is the randomized skill in sync list in inherit list
                    s_list[found2].in_use = true;
                    s_list[x as usize ].randomized_index = found2 as i32;
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
        // Inherit Chaos Mode
        if GameVariableManager::get_number("G_SPCost") == 2 {
            let skill_pool = SKILL_POOL.lock().unwrap();
            let mut available: Vec<i32> = Vec::with_capacity(skill_pool.len());
            skill_pool.iter().for_each(|x| available.push(x.index));
            println!("Skill Pool For Inherit Rando: {}", available.len());
            println!("Inherit Skill Size: {}", self.chaos_inherit_list.len());
            if available.len() < self.chaos_inherit_list.len() {
                let msg = format!("Skill Inherit List exceeds Skill Pool List.\nInherit List Size: {} vs Skill Pool Size: {}\nPlease set non-inheritables to 0 SP.", self.chaos_inherit_list.len(), available.len());
                panic!("{}", msg.as_str());
            }
            self.chaos_inherit_list.iter_mut()
                .for_each(|x|{
                    let index = rng.get_value( available.len() as i32) as usize;
                    x.1 = available[index];
                    if let Some(skill) = SkillData::try_index_get(x.1) {
                        x.2 = skill.get_inheritance_cost() == 0;
                        available.remove(index);
                    }
                }
            );
        }
        self.randomized = true;
    }
    pub fn get_replacement_sid(&mut self, sid: &Il2CppString, is_inherit: bool) -> &'static SkillData {
        if let Some(skill) = SkillData::get(&sid.to_string()) { 
            if is_inherit && GameVariableManager::get_number("G_SPCost") == 2 {
                return if let Some(skill2) = self.inherit_skill_chaos_mode(skill){skill2 }
                    else {  SkillData::get("SID_無し").unwrap() };
            }
            let replacement = self.get_replacement(skill, is_inherit);
            return replacement;
        }
        else {  SkillData::get("SID_無し").unwrap() }
    }
    pub fn get_replacement(&mut self, original_skill: &SkillData, is_inherit: bool) -> &'static SkillData {
        let skill_list = SkillData::get_list().unwrap();
        let o_skill = &skill_list[ original_skill.parent.index as usize];
        let sid = original_skill.sid.to_string();    
        if original_skill.get_flag() & 1 != 0 && sid != "SID_無し" {  return o_skill; }     
    // ignore "Night and Day", "Friendly Riviary"
        if sid == "SID_オルタネイト" || sid == "SID_切磋琢磨" { return o_skill; }
        let mut priority = original_skill.get_priority();
        let skill_index;
        if sid == "SID_異界の力" { priority = 0; }
        if let Some(eirika) = EIRIKA_TWIN_SKILLS.iter().position(|x| *x == sid) {
            skill_index = if eirika < 6 { original_skill.parent.index } else { original_skill.parent.index - 3 };
            priority = if eirika < 6 { 1 } else { 2 };
        }
        else { skill_index = if priority == 0 { original_skill.parent.index } else { original_skill.parent.index - (priority - 1)};  }

        let chaos_mode = GameVariableManager::get_number("G_ChaosMode");
        let mode = if is_inherit { 0 }
            else if chaos_mode & 1 != 0 { 2 }
            else { 1 }; 

        let list = [&mut self.inherit_list, &mut self.sync_list, &mut self.chaos_list];
        let found = list[mode as usize].iter_mut().find(|x| x.index == skill_index);
        if found.is_none() {  return o_skill;  }
        let f = found.unwrap().clone();
        let new_skill_index = list[mode as usize][ f.randomized_index as usize ].index;
        let new_max_priority = list[mode as usize][ f.randomized_index as usize ].max_priority;
        let is_eirika_twin = list[mode as usize][ f.randomized_index as usize ].eirika_twin_skill;
        let new_priority = ( priority as i8 ) - f.min_priority + list[mode as usize][ f.randomized_index as usize ].min_priority;

        if mode >= 1 { list[mode as usize][f.randomized_index as usize ].skill_used = true; }

        if is_eirika_twin { // Replacement skill is an Eirika Twin Skill (Lunar Brace/Solar Brace/Eclipse Brace etc...)
            if new_max_priority <= priority as i8 { // new_max_priority is 2 for Lunar/Solar/Eclipse Brace +
                return &skill_list[ ( new_skill_index + 3 ) as usize]; 
            }
            return &skill_list[ new_skill_index as usize];
        }
        if new_max_priority == 0 || priority == 0 {  return &skill_list[new_skill_index as usize]; }

        let index = if new_max_priority <= new_priority { new_skill_index + (new_max_priority as i32 ) - 1 }
            else { new_skill_index + (new_priority as i32 ) - 1  };
        SkillData::try_index_get(index).unwrap()
    }
    pub fn inherit_skill_chaos_mode(&self, skill: &SkillData) -> Option<&'static SkillData> {
        if let Some(x) = self.chaos_inherit_list.iter().find(|x| x.0 == skill.parent.index) {
            return SkillData::try_index_get(x.1);
        }
        None
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
            let sid = skill_list[ i_list[x].index as usize].sid.to_string();
            writeln!(&mut f, "{}\t{}", i_list[x].index, sid).unwrap();
        }
        for x in 0..i_list.len() {
            let sid = skill_list[ i_list[x].index as usize].sid.to_string();
            let name = Mess::get(skill_list[ i_list[x].index as usize].name.expect(format!("Skill #{}: {} does not have a valid MSID", i_list[x].index, sid).as_str())).to_string();

            let random_index =  i_list[x].randomized_index as usize;
            let sid2 = skill_list[ i_list[random_index].index as usize].sid.to_string();
            let name2 = Mess::get(skill_list[ i_list[random_index].index as usize].name.expect(format!("Skill #{}: {} does not have a valid MSID", i_list[random_index].index, sid2).as_str())).to_string();

            writeln!(&mut f, "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}", x, sid, name, sid2, name2, i_list[x].index,  random_index, i_list[x].max_priority, i_list[random_index].max_priority).unwrap();
        }
        let s_list = &self.sync_list;
        writeln!(&mut f, "\nSync Skills").unwrap();
        writeln!(&mut f, "Index\tSid\tName\tTo Sid\tTo Name\tSkill Index\tRandom Skill Index\tSkill Max Priority\tRandom Skill Max Priority\tUsed").unwrap();
        for x in 0..s_list.len() {
            let sid = skill_list[ s_list[x].index as usize].sid.to_string();
            let name = Mess::get(skill_list[ s_list[x].index as usize].name.expect(format!("Skill #{}: {} does not have a valid MSID", s_list[x].index, sid).as_str())).to_string();
            if s_list[x].randomized_index == -1 { continue; }
            let random_index =  s_list[x].randomized_index as usize;
            let sid2 = skill_list[ s_list[random_index].index as usize].sid.to_string();
            let name2 = Mess::get(skill_list[ s_list[random_index].index as usize].name.expect(format!("Skill #{}: {} does not have a valid MSID", s_list[random_index].index, sid2).as_str())).to_string();

            writeln!(&mut f, "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}", x, sid, name, sid2, name2, s_list[x].index, random_index, s_list[x].max_priority, s_list[random_index].max_priority, s_list[random_index].skill_used).unwrap();
        }
        let c_list = &self.chaos_list;
        writeln!(&mut f, "\nChaos Skills").unwrap();
        writeln!(&mut f, "Index\tSid\tName\tTo Sid\tTo Name\tSkill Index\tRandom Skill Index\tSkill Max Priority\tRandom Skill Max Priority").unwrap();
        for x in 0..c_list.len() {
            let sid = skill_list[ c_list[x].index as usize].sid.to_string();
            let name = Mess::get(skill_list[ c_list[x].index as usize].name.expect(format!("Skill #{}: {} does not have a valid MSID", c_list[x].index, sid).as_str())).to_string();
            if c_list[x].randomized_index == -1 { continue; }
            let random_index =  c_list[x].randomized_index as usize;
            let sid2 = skill_list[ c_list[random_index].index as usize].sid.to_string();
            let name2 = Mess::get(skill_list[ c_list[random_index].index as usize].name.expect(format!("Skill #{}: {} does not have a valid MSID", c_list[random_index].index, sid2).as_str())).to_string();

            writeln!(&mut f, "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}", x, sid, name, sid2, name2, c_list[x].index, random_index, c_list[x].max_priority, c_list[random_index].max_priority).unwrap();
        }
    }
    pub fn get_sync_list_size(&mut self) { 
        println!("Syncho Skill List Size: {}", self.sync_list.len());
        self.sync_list_size = self.sync_list.len() as i32; 
    }
    pub fn randomized_skill_cost(&self, rng: &Random) {
        if GameVariableManager::get_number("G_SPCost") != 0 { return; }
        let pool_size = self.inherit_cost.len() as i32;
        // Make all 0
        self.inherit_cost.iter().for_each(|x|{
            if let Some(skill) = SkillData::try_index_get_mut(x.0) {
                skill.set_inherit_cost(0);
            }
        });
        self.inherit_cost.iter().for_each(|x|{
            if let Some(skill) = SkillData::try_index_get_mut(x.0) {
                if skill.get_inheritance_cost() == 0 {
                    let mut current_index = x.0;
                    let mut current_priority = skill.get_priority();
                    let mut current_cost = 100 + 50 * rng.get_value(59);  // Base
                    skill.set_inherit_cost(current_cost as u16);
                    if current_priority > 0 {
                        current_index += 1;
                        let mut next_skill =  SkillData::try_index_get_mut( current_index).unwrap();
                        let mut max_priority = current_priority;
                    // Getting Max_priority     
                        while next_skill.get_priority() > max_priority {
                            max_priority = next_skill.get_priority();
                            current_index += 1;
                            next_skill = SkillData::try_index_get_mut( current_index ).unwrap();
                        }
                        let dsp =  7000 / ( 50 * max_priority );
                        current_index = x.0 + 1;
                        next_skill =  SkillData::try_index_get_mut( current_index).unwrap();
                        while next_skill.get_priority() > current_priority {
                            current_priority = next_skill.get_priority();
                            current_index += 1;
                            current_cost +=  50 * (rng.get_value(dsp) + 2 );
                            next_skill.set_inherit_cost( current_cost as u16 );
                            next_skill = SkillData::try_index_get_mut( current_index ).unwrap();
                        }
                    }
                }
            }
        });
        if GameVariableManager::get_number("G_SPCost") != 2 { return; }
        self.chaos_inherit_list.iter().for_each(|x|{
            if let Some(skill) = SkillData::try_index_get_mut(x.1) {
                if skill.get_inheritance_cost() == 0 {
                    let mut current_cost = 500  + 150 * rng.get_value(30) ;  // Base
                    skill.set_inherit_cost(current_cost as u16);
                }
            }
        });
    }
    pub fn reset_skill_cost(&self) {
        self.inherit_cost.iter().for_each(|x|{
            if let Some(skill) = SkillData::try_index_get_mut(x.0) {
                skill.set_inherit_cost(x.1);
            }
        });
        self.chaos_inherit_list.iter().for_each(|x|{
            if let Some(skill) = SkillData::try_index_get_mut(x.1) {
                if x.2 { skill.set_inherit_cost(0);
            }
        }
        });
    }
}