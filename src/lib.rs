use std::{mem, io};
use libc::*;
use num_cpus;

// get CPU info
use procfs::CpuInfo;
pub struct CPU {
    pub model: String,
    pub physical_cores: usize,   // cores on chip
    pub execution_units: usize,  // execution units (physical_cores * # of threads/core)
    pub threads_per_core: usize, // threads
    pub sockets: usize,
}
impl CPU {
    pub fn new() -> CPU {
        let mut cmod = "No CPU Info Available".to_owned();
        let mut exu = 0;
        let mut socket_cores = 0;
        let mut siblings = 1;

        let cpu = CpuInfo::new();
        match cpu {
            Err(_) => println!("No CPU available"),
            Ok(cpu) => {
                match cpu.model_name(0) {
                    None => println!("No CPU model info available"),
                    Some(model_id) => {
                        cmod = model_id.to_string();
                    }
                }
                match cpu.get_info(0) {
                    None => println!("No additional info available."),
                    Some(details) => {
                        // println!("{:#?}", details);
                        // physical cores
                        let physical_cores = details.get(&"cpu cores").unwrap();
                        socket_cores = physical_cores.parse().unwrap_or(0);
                        // execution units with hyperthreading
                        let exec_cores = details.get(&"siblings").unwrap();
                        siblings = exec_cores.parse().unwrap_or(0);
                    }
                }
                exu = cpu.num_cores();
            }
        }
        CPU {
            model: cmod.to_string(),
            execution_units: exu,
            physical_cores: socket_cores,
            threads_per_core: siblings / socket_cores,
            sockets: exu / siblings,
        }
    }
}

/// Bind CPU ids
pub fn bind_cpu_ids(mut start:usize, mut end:usize) {
    let cores_num = num_cpus::get_physical() * 2;
    if start > end || start >= cores_num {
        start = 0;
    }
    if end >= cores_num {
        end = cores_num-1;
    }
    
    unsafe {
        let mut cpu_set: cpu_set_t = mem::zeroed();
        CPU_ZERO(&mut cpu_set);
        for coreid in start..=end as usize {
            // println!("coreid: {:?}",coreid);
            CPU_SET(coreid, &mut cpu_set);
        }
        let ret = sched_setaffinity(0, std::mem::size_of::<cpu_set_t>(), &cpu_set as *const cpu_set_t);
        assert_eq!(
            ret,
            0,
            "sched_setaffinity is expected to return 0, was {}: {:?}",
            ret,
            io::Error::last_os_error()
        );
    }
    if check_rustlog() {
        println!("bind_core - pid:{} cpus:{:?}~{:?}", get_pid(), start, end);
    }
}

/// Returns a list of CPU ids
pub fn get_cpu_ids(mut mark: &str) -> Vec<usize> {
    let mut cpus = Vec::new();
    let mut cpu_set: cpu_set_t;
    let r = unsafe {
        cpu_set = mem::zeroed::<cpu_set_t>();
        sched_getaffinity(0, std::mem::size_of::<cpu_set_t>(), &mut cpu_set)
    };

    if r == 0 {
        for c in 0..CPU_SETSIZE as usize {
            if unsafe { CPU_ISSET(c, &cpu_set) } {
                cpus.push(c);
            }
        }
    }
    if check_rustlog() {
        if mark.is_empty() {
            mark = "get_core"
        }
        println!("{} - pid:{} cpus:{} {:?}", mark, get_pid(), cpus.len(), cpus);
    }
    return cpus;
}

/// Returns current process_id
fn get_pid() -> u32 {
    let tid = unsafe { syscall(SYS_gettid) } as u32;
    return tid;
}

/// Returns the environment variable `RUST_LOG` value
fn check_rustlog() -> bool {
    if let Ok(used) = std::env::var("RUST_LOG") {
        if used.is_empty() {
            false
        } else {
            match used.to_uppercase().as_str() {
                "TRACE" | "DEBUG" => true,
                _ => match used.parse::<usize>() {
                    Ok(num) => num != 0,
                    Err(_) => false,
                },
            }
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cores_ext() {
        let cpu = CPU::new();
        let start = cpu.execution_units - cpu.physical_cores*cpu.sockets;
        let end = cpu.execution_units;
        bind_cpu_ids(start, end);

        let cpus = get_cpu_ids("");
        println!("cores:{} {:?}", cpus.len(), cpus);
    }
}
