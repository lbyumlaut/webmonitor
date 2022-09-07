use actix_web::{web, App, HttpServer};
use sysinfo::{ComponentExt, SystemExt, CpuExt};
use serde::Serialize;
use actix_files as fs;
use std::{env, sync::Mutex};

#[derive(Serialize, Debug, Default)]
struct ProcessorCore {
    name: String,
    utilization: f32,
    frequency: u64,
    temperature: f32,
}

#[derive(Serialize, Debug, Default)]
struct Filesystem {
    mount_point: String,
    available_space: u64,
    total_space: u64,
}

#[derive(Serialize, Debug, Default)]
struct Temperature {
    component_name: String,
    temp: f32,
}

#[derive(Serialize, Debug, Default)]
struct SysInfo {
    processor_cores: Vec<ProcessorCore>,
    free_ram: u64,
    total_ram: u64,
    used_ram: u64,
    disks: Vec<Filesystem>,
    load_average_1m: f64,
    load_average_5m: f64,
    load_average_15m: f64,
    temperatures: Vec<Temperature>,
    system_name: String,
    kernel_version: String,
    os_version: String,
}

async fn sysinfo(s: web::Data<Mutex<sysinfo::System>>) -> web::Json<SysInfo> {    
    
    let mut system = s.lock().unwrap();
    
    system.refresh_all();
    
    let mut sys = SysInfo::default();
    
    
    
    let processors = system.cpus();
    for processor in processors {
        sys.processor_cores.push(ProcessorCore{
            name: processor.name().to_string(),
            utilization: processor.cpu_usage(),
            frequency: std::fs::read_to_string(format!("/sys/devices/system/cpu/{}/cpufreq/scaling_cur_freq", processor.name().to_string())).unwrap_or_default().trim().parse().unwrap_or(0),
            temperature: std::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp").unwrap_or_default().trim().parse().unwrap_or(1.0)/1000.0,
        });
    }
    
    sys.free_ram = system.free_memory();
    sys.total_ram = system.total_memory();
    sys.used_ram = system.used_memory();
    sys.load_average_1m = system.load_average().one;
    sys.load_average_5m = system.load_average().five;
    sys.load_average_15m = system.load_average().fifteen;
    
    for component in system.components() {
        sys.temperatures.push(Temperature { component_name: component.label().to_string(), temp: component.temperature() });
    }

    sys.system_name = system.name().unwrap_or_default();
    sys.os_version = system.long_os_version().unwrap_or_default();
    sys.kernel_version = system.kernel_version().unwrap_or_default();

    // let disks = system.disks();
    // for disk in disks {
    //     sys.disks.push(Disk{ name: disk.name().to_str().unwrap_or_default().to_string(),
    //         mount_point: disk.mount_point().to_str().unwrap_or_default().to_string(),
    //         available_space: disk.available_space(),
    //         total_space: disk.total_space(),
    //         filesystem: String::from_utf8_lossy(disk.file_system()).to_string(),
            
    //     });
    // }
    match fs2::statvfs("/")  {
        Ok(stat) => { 
            sys.disks.push(Filesystem { 
                available_space: stat.available_space(),
                mount_point: "/".to_string(),
                total_space: stat.total_space()})},
        Err(_)=> println!("Error retrieving disk stats!"),
    }

    web::Json(sys)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    let args: Vec<String> = env::args().collect();
    
    let name = std::env::current_exe()
    .expect("Can't get the exec path")
    .file_name()
    .expect("Can't get the exec name")
    .to_string_lossy()
    .into_owned();
    
    if args.len() == 1 {
        println!("Usage: {} {{path to web folder}}", name);
        std::process::exit(exitcode::USAGE);
    }
    
    HttpServer::new(move || {
        let system = actix_web::web::Data::new(Mutex::new(sysinfo::System::new()));
        App::new().service(fs::Files::new("/app", args[1].clone()).show_files_listing().index_file("index.html"))
        .route("/sysinfo", web::get().to(sysinfo)).app_data(system)
    })
    .bind("0.0.0.0:80")?
    .run()
    .await
}

