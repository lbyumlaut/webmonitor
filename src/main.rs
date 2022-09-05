use actix_web::{web, App, HttpRequest, HttpServer};
use sysinfo::{SystemExt, ProcessorExt, DiskExt};
use serde::Serialize;
use actix_files as fs;
use std::env;

#[derive(Serialize, Debug, Default)]
struct ProcessorCore {
    name: String,
    utilization: f32
}

#[derive(Serialize, Debug, Default)]
struct Disk {
    name: String,
    mount_point: String,
    available_space: u64,
    total_space: u64,
    filesystem: String,
}

#[derive(Serialize, Debug, Default)]
struct SysInfo {
    processor_cores: Vec<ProcessorCore>,
    free_ram: u64,
    total_ram: u64,
    used_ram: u64,
    disks: Vec<Disk>,
}

async fn sysinfo(_req: HttpRequest) -> web::Json<SysInfo> {    
    let mut system = sysinfo::System::new();
    system.refresh_all();

    let mut sys = SysInfo::default();
    
    let processors = system.get_processor_list();
    for processor in processors {
        sys.processor_cores.push(ProcessorCore{name: processor.get_name().to_string(), utilization: processor.get_cpu_usage()});
    }

    sys.free_ram = system.get_free_memory();
    sys.total_ram = system.get_total_memory();
    sys.used_ram = system.get_used_memory();

    
    let disks = system.get_disks();
    for disk in disks {
        sys.disks.push(Disk{ name: disk.get_name().to_str().unwrap_or_default().to_string(),
                             mount_point: disk.get_mount_point().to_str().unwrap_or_default().to_string(),
                             available_space: disk.get_available_space(),
                             total_space: disk.get_total_space(),
                             filesystem: String::from_utf8_lossy(disk.get_file_system()).to_string(),

            });
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
    let path_for_page = args[1].clone();

    HttpServer::new(move|| {
        App::new().service(fs::Files::new("/app", &path_for_page).show_files_listing().index_file("index.html"))
        .route("/sysinfo", web::get().to(sysinfo))
    })
    .bind("127.0.0.1:80")?
    .run()
    .await
}
