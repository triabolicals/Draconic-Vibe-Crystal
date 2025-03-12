use super::*;
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
    pub sync_rando: Vec<(i32, i32)>,
}

const EIRIKA_HASH: [i32; 12] = [ 1166279381, 1203307432, 244739392, 446418448, 933063973, -1323396701, 	1137740356, -1874837901, 919405771, -213541829, -1311625676, 1981791378];
const NONE_SID: i32 = 359194254;
const FRIENDLY_RIVARLRY: i32 = 1238512915;
const NIGHT_DAY: i32 = 924387794;
const BOOK_OF_WORLDS: i32 = 106021179;

impl SynchoList {
    // For the three houses gambits to force them to be 4 separate skills instead of one 4-level skill
    pub fn add_to_non_upgrade(&mut self, sid: &str, is_not_randomized: bool){
        if let Some(skill) = SkillData::get(sid) {
            let skill_index = skill.parent.index;
            if ( skill.get_flag() & 1 == 1 || skill.help.is_none() ) && skill.parent.hash != NONE_SID {  return; } 
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
            if (skill.get_flag() & 1 == 1 || skill.help.is_none() ) && skill.parent.hash != NONE_SID {  return; }  
            let index = skill.parent.index;
            self.add_list(skill, is_syncho);
            if self.non_random_skills.iter().find(|&&x| x == index).is_none() && is_not_randomized {
                self.non_random_skills.push(index); // Skills not do not get randomized to another skill but other skills can randomized to
            }
         }
    }
    pub fn add_by_index(&mut self, skill_index: i32, is_syncho: bool, is_not_randomized: bool){
        if let Some(skill) = SkillData::try_index_get(skill_index)  { 
            if (skill.get_flag() & 1 == 1 || skill.help.is_none() ) && skill.parent.hash != NONE_SID {  return; } 
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
        let hash = skill.parent.hash;
        if skill.help.is_none() && skill.parent.index > 2 {  return; }
        if skill.flag & 1 != 0  && hash != NONE_SID {  return; }    // cannot be hidden
        if hash == FRIENDLY_RIVARLRY || hash == NIGHT_DAY   { return; }

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
        let sid = skill.sid.to_string();

        if hash == BOOK_OF_WORLDS { // if book of worlds
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
        self.sync_rando.clear();
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
            if self.non_random_skills.iter().find(|&&y| y == s_list[x as usize].index ).is_some() { //Prevent the non-randomized from being pick to replaced
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
        if GameVariableManager::get_number(DVCVariables::SP_KEY) == 2 {
            let skill_pool = SKILL_POOL.lock().unwrap();
            let mut available: Vec<i32> = Vec::with_capacity(skill_pool.len());
            skill_pool.iter().for_each(|x| available.push(x.index));
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
        if let Some(skill) = SkillData::get(sid) { 
            if is_inherit && GameVariableManager::get_number(DVCVariables::SP_KEY) == 2 {
                return if let Some(skill2) = self.inherit_skill_chaos_mode(skill){ skill2 }
                    else {  SkillData::try_index_get(0).unwrap() }
            }
            else { self.get_replacement(skill, is_inherit) }
        }
        else {  SkillData::try_index_get(0).unwrap() }
    }
    pub fn get_replacement(&mut self, original_skill: &SkillData, is_inherit: bool) -> &'static SkillData {
        let o_index = original_skill.parent.index;
        if !is_inherit {
            if let Some(new) = self.sync_rando.iter().find(|x| x.0 == o_index) {
                return SkillData::try_index_get(new.1).unwrap();
            }
        }
        let o_skill = SkillData::try_index_get(o_index).unwrap();  
        let hash = original_skill.parent.hash;
        if ( original_skill.get_flag() & 1 != 0 && hash != NONE_SID ) ||  hash == FRIENDLY_RIVARLRY || hash == NIGHT_DAY { 
            if !is_inherit { self.sync_rando.push( (o_index, o_index)); }
            return o_skill;
        }     // Hidden and not None
       // if hash == 1238512915 || hash == "SID_切磋琢磨" { return o_skill; } // ignore "Night and Day", "Friendly Riviary"

        let mut priority = original_skill.get_priority();
        let skill_index;
        if hash == BOOK_OF_WORLDS { priority = 0; }
        if let Some(eirika) = EIRIKA_HASH.iter().position(|&x| x == hash) {
            skill_index = if eirika < 6 { o_index } else { o_index- 3 };
            priority = if eirika < 6 { 1 } else { 2 };
        }
        else { 
            skill_index = if priority == 0 { o_index } else { o_index - (priority - 1)}; 
        }

        let chaos_mode = GameVariableManager::get_number(DVCVariables::EMBLEM_SKILL_CHAOS_KEY);
        let mode = if is_inherit { 0 }
            else if chaos_mode & 1 != 0 { 2 }
            else { 1 }; 

        let list = if is_inherit { &mut self.inherit_list }
            else if chaos_mode & 1 != 0 { &mut self.chaos_list }
            else { &mut self.sync_list }; 
        let index;
        let randomized_index;
        if let Some(found) = list.iter().find(|x| x.index == skill_index) {
            randomized_index =  found.randomized_index as usize;
            let new_skill_index = list[ randomized_index ].index;
            let new_max_priority = list[ randomized_index].max_priority;
            let is_eirika_twin = list[ randomized_index ].eirika_twin_skill;
            let new_priority = ( priority as i8 ) - found.min_priority + list[ randomized_index ].min_priority;

            if is_eirika_twin { // Replacement skill is an Eirika Twin Skill (Lunar Brace/Solar Brace/Eclipse Brace etc...)
                index = new_skill_index + if new_max_priority <= priority as i8 { 3 } else { 0 }; // new_max_priority is 2 for Lunar/Solar/Eclipse Brace +
            }
            else if new_max_priority == 0 || priority == 0 { index = new_skill_index; }
            else {
                index = if new_max_priority <= new_priority { new_skill_index + (new_max_priority as i32 ) - 1 } else { new_skill_index + (new_priority as i32 ) - 1  };
            }
        }
        else { 
            if !is_inherit { self.sync_rando.push( (o_index, o_index)); }
            return o_skill;
        }
        if mode >= 1 { list[ randomized_index ].skill_used = true; }    // removes already used sync skills for extra sync skills
        let skill = SkillData::try_index_get(index).unwrap();
        if skill.help.is_none() || skill.flag & 1 != 0 {   // In case if the index is incorrect, search for skill that matches priority and has help text
            let priority = skill.priority;
            let mut n_index = skill.parent.index;
            let mut count = 0;
            while count < 10 {
                n_index -= 1;
                count += 1;
                if index < 1 { break; }
                let correct_skill = SkillData::try_index_get(n_index).unwrap();
                if correct_skill.help.is_some() && correct_skill.priority == priority && correct_skill.flag & 1 == 0 {
                    if !is_inherit { self.sync_rando.push( (o_index, n_index)); }
                    return correct_skill;
                }
            }
            count = 0;
            while count < 10 {
                n_index += 1;
                count += 1;
                if index >= SkillData::get_count() - 1 { break; }
                let correct_skill = SkillData::try_index_get(n_index).unwrap();
                if correct_skill.help.is_some() && correct_skill.priority == priority && correct_skill.flag & 1 == 0 {
                    if !is_inherit { self.sync_rando.push( (o_index, n_index)); }
                    return correct_skill;
                }
            }
        }
        if !is_inherit { 
            // println!("{} -> {}", Mess::get(original_skill.name.unwrap()), Mess::get(skill.name.unwrap()));
            self.sync_rando.push( (o_index, index)); 
        }
        return skill;
    }
    pub fn inherit_skill_chaos_mode(&self, skill: &SkillData) -> Option<&'static SkillData> {
        if let Some(x) = self.chaos_inherit_list.iter().find(|x| x.0 == skill.parent.index) {
            return SkillData::try_index_get(x.1);
        }
        None
    }

    pub fn get_non_randomized_skill(&mut self) -> Vec<&mut SynchoSkill> {
        let chaos_mode = GameVariableManager::get_number(DVCVariables::EMBLEM_SKILL_CHAOS_KEY);
        let list = if chaos_mode & 1 != 0 { &mut self.chaos_list } else { &mut self.sync_list };
        let out: Vec<_> = list.iter_mut().filter(|x| !x.skill_used ).collect();
        out
    }
    pub fn get_sync_list_size(&mut self) { 
        self.sync_list_size = self.sync_list.len() as i32; 
    }
    pub fn randomized_skill_cost(&self, rng: &Random) {
        if GameVariableManager::get_number(DVCVariables::SP_KEY) == 0 { return; }
        // Make all 0
        self.inherit_cost.iter().for_each(|x|{
            if let Some(skill) = SkillData::try_index_get_mut(x.0) { skill.set_inherit_cost(0); }
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
        if GameVariableManager::get_number(DVCVariables::SP_KEY) != 2 { return; }
        self.chaos_inherit_list.iter().for_each(|x|{
            if let Some(skill) = SkillData::try_index_get_mut(x.1) {
                if skill.get_inheritance_cost() == 0 {
                    let current_cost = 500  + 150 * rng.get_value(30) ;  // Base
                    skill.set_inherit_cost(current_cost as u16);
                }
            }
        });
    }
    pub fn reset_skill_cost(&self) {
        self.inherit_cost.iter()
            .for_each(|x|{
                if let Some(skill) = SkillData::try_index_get_mut(x.0) {
                    skill.set_inherit_cost(x.1);
                }
            }
        );
        self.chaos_inherit_list.iter()
            .for_each(|x|{
                if let Some(skill) = SkillData::try_index_get_mut(x.1) {
                    if x.2 { skill.set_inherit_cost(0); }
                }
            }
        );
    }
}