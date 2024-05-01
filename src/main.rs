use std::{thread, time::Duration};

use clap::Parser;
use nvml_wrapper::{enum_wrappers::device::TemperatureSensor, Device, Nvml};
use serde_derive::Serialize;
use sysinfo::System;

#[derive(Serialize)]
struct CoreInfo {
  usage: f32,
  frequency: u64,
}

#[derive(Serialize)]
struct CpuInfo {
  name: String,
  usage: f32,
  frequency: u64,
  cores: Vec<CoreInfo>,
}

#[derive(Serialize)]
struct MemoryInfo {
  ram_used: u64,
  ram_total: u64,
  swap_used: u64,
  swap_total: u64,
}


#[derive(Serialize)]
struct GpuInfo {
  name: String,
  usage: f32,
  decoder: f32,
  memory: f32,
  temperature: u32,
}

#[derive(Serialize)]
struct HwInfo {
  cpu: CpuInfo,
  memory: MemoryInfo,
  gpus: Vec<GpuInfo>,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
  #[arg(short, long, default_value_t = 1000, help = "Poll interval in milliseconds")]
  poll_interval: u64,
}

fn main() {
  let args = Args::parse();
  let poll_interval = Duration::from_millis(args.poll_interval);

  let mut sys = System::new_all();

  let nvml = Nvml::init().unwrap();
  
  let gpus: Vec<Device> = {
    let count = nvml.device_count().unwrap();

    (0..count).map(|i| nvml.device_by_index(i).unwrap()).collect()
  };
  
  loop {
    sys.refresh_cpu();
    sys.refresh_memory();

    let cores: Vec<CoreInfo> = sys.cpus().iter().map(|cpu| CoreInfo {
      usage: cpu.cpu_usage(),
      frequency: cpu.frequency(),
    }).collect();

    let info = HwInfo {
      cpu: CpuInfo {
        name: sys.cpus()[0].brand().trim().to_owned(),
        usage: sys.global_cpu_info().cpu_usage(),
        frequency: sys.global_cpu_info().frequency(),
        cores,
      },
      memory: MemoryInfo {
        ram_used: sys.used_memory(),
        ram_total: sys.total_memory(),
        swap_used: sys.used_swap(),
        swap_total: sys.total_swap(),
      },
      gpus: gpus.iter().map(|gpu| GpuInfo {
        name: gpu.name().unwrap(),
        usage: gpu.utilization_rates().unwrap().gpu as f32,
        decoder: gpu.decoder_utilization().unwrap().utilization as f32,
        memory: {
          let mem = gpu.memory_info().unwrap();
          mem.used as f32 / mem.total as f32
        },
        temperature: gpu.temperature(TemperatureSensor::Gpu).unwrap(),
      }).collect(),
    };

    println!("{}", serde_json::to_string(&info).unwrap());

    thread::sleep(poll_interval);
  }
}
