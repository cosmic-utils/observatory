/* sys_info_v2/proc_info.rs
 *
 * Copyright 2024 Romeo Calota
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use super::{Process, ProcessUsageStats};

#[allow(dead_code)]
pub fn process_hierarchy(processes: &std::collections::HashMap<u32, Process>) -> Option<Process> {
    use std::collections::*;

    let now = std::time::Instant::now();

    let pids = processes.keys().map(|pid| *pid).collect::<BTreeSet<_>>();
    let root_pid = match pids.first() {
        None => return None,
        Some(pid) => *pid,
    };

    let mut root_process = match processes.get(&root_pid).map_or(None, |p| Some(p.clone())) {
        None => return None,
        Some(p) => p,
    };

    let mut process_tree = BTreeMap::new();
    process_tree.insert(root_process.pid, 0_usize);

    let mut children = Vec::with_capacity(pids.len());
    children.push(HashMap::new());

    let mut visited = HashSet::new();
    visited.insert(root_process.pid);

    for pid in pids.iter().skip(1).rev() {
        if visited.contains(pid) {
            continue;
        }

        let process = match processes.get(pid) {
            None => continue,
            Some(p) => p,
        };

        let mut stack = vec![process];
        let mut parent = process.parent;
        while parent != 0 {
            let parent_process = match processes.get(&parent) {
                None => break,
                Some(pp) => pp,
            };

            if visited.contains(&parent_process.pid) {
                let mut index = match process_tree.get(&parent_process.pid) {
                    None => {
                        // TODO: Fully understand if this could happen, and what to do if it does.
                        log::error!(
                            "Process {} has been visited, but it's not in the process_tree?",
                            process.pid
                        );
                        break;
                    }
                    Some(index) => *index,
                };
                while let Some(ancestor) = stack.pop() {
                    let p = ancestor.clone();
                    children[index].insert(p.pid, p);

                    visited.insert(ancestor.pid);

                    index = children.len();
                    process_tree.insert(ancestor.pid, index);
                    children.push(HashMap::new());
                }

                break;
            }

            stack.push(parent_process);
            parent = parent_process.parent;
        }
    }

    fn gather_descendants(
        process: &mut Process,
        process_tree: &BTreeMap<u32, usize>,
        children: &mut Vec<HashMap<u32, Process>>,
    ) {
        let pid = process.pid;

        let index = match process_tree.get(&pid) {
            Some(index) => *index,
            None => return,
        };

        if children[index].is_empty() {
            return;
        }

        std::mem::swap(&mut process.children, &mut children[index]);

        let mut merged_stats = ProcessUsageStats::default();
        for (_, child) in &mut process.children {
            gather_descendants(child, process_tree, children);
            merged_stats.merge(&child.merged_usage_stats);
        }
        process.merged_usage_stats.merge(&merged_stats);
    }

    let process = &mut root_process;
    std::mem::swap(&mut process.children, &mut children[0]);

    let mut merged_stats = ProcessUsageStats::default();
    for (_, child) in &mut process.children {
        gather_descendants(child, &process_tree, &mut children);
        merged_stats.merge(&child.merged_usage_stats);
    }
    process.merged_usage_stats.merge(&merged_stats);

    log::debug!(
        "[{}:{}] Loading process hierarchy took {}ms",
        file!(),
        line!(),
        now.elapsed().as_millis()
    );

    Some(root_process)
}
