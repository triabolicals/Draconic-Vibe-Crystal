use std::collections::HashMap;
use engage::{
    app::{
        IGodDataMethods,
        DisposData,
        IDisposDataMethods,
        IStructDataArray_1Methods,
        IPersonDataMethods
    },
    List_1Ext,
};
use engage::app::{IJobData, IJobDataMethods};
use unity::Cast;
use crate::{gamedata::{EmblemPool, PersonPool}, utils::{il2str, to_opt}, DVCVariable, DVCVariables};
use crate::enums::EMBLEM_GIDS;
// RecommendLevel, Emblem Level, Generic Level, Special Generic Level
pub const PARALOGUE_LEVELS: [u8; 48] = [
    34, 40, 15, 20,
    25, 30, 10, 20,
    30, 35, 12, 15,
    28, 35, 8, 13,
    30, 35, 11, 28,
    28, 35, 10, 15,
    19, 25, 1, 2,
    19, 28, 2, 4,
    23, 30, 4, 6,
    25, 32, 6, 10,
    27, 32, 6, 8,
    28, 35, 7, 9,
];
pub fn change_map_dispos() {
    let dispos_list = DisposData::get_list();
    dispos_list.base().iter()
        .flat_map(|group| group.base().base().iter())
        .filter(|v| !v.is_null() && !v.get_pid().is_null())
            .for_each(|dispos| {
                if let Some(replacement) = to_opt(dispos.get_person()).and_then(|person| PersonPool::get_replacement_2(person, false)) {
                    dispos.set_pid(replacement.get_pid());
                }
                if let Some(replacement) = il2str(dispos.get_gid())
                    .and_then(|gid| EMBLEM_GIDS.iter().position(|&v| *v == gid))
                    .and_then(|i| EmblemPool::get_replacement_god_3(i as i32, false))
                {
                    dispos.set_gid(replacement.get_gid());
                }
        });
    let chapter = DVCVariables::get_chapter_index();
    if chapter >= 33 && chapter < 45 {
        let mut level_map: HashMap<i32, i32> = HashMap::new();
        dispos_list.base().iter()
            .flat_map(|group| group.base().base().iter())
            .filter(|v| !v.is_null() && !v.get_pid().is_null() && v.get_force_type().value != 0)
            .for_each(|v|{
                if let Some(person) = to_opt(v.get_person()){
                    let job = person.get_job();
                    let level = person.get_level() as i32;
                    if job.is_high() || (job.is_low() && job.has_high_jobs()) {
                        if let Some(count) = level_map.get_mut(&level) { *count += 1; }
                        else { level_map.insert(level, 1); }
                    }
                }
            });
        if let Some(mean) = level_map.into_iter().max_by(|a, b|a.1.cmp(&b.1)) {
            DVCVariable::EmblemParalogueGenericLevel.set_value(mean.0);
        }
    }
}